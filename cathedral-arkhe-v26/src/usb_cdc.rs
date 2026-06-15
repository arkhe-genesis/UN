//! USB-CDC v2 Transport for Cathedral ARKHE v26.1
//! Protocol: [magic: 0xC4FE] [req_id: u32] [opcode: u8] [flags: u8] [len: u16] [payload] [crc32: u32]

use heapless::Vec;
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;

const MAGIC: [u8; 2] = [0xC4, 0xFE];
const CRC32_POLY: u32 = 0xEDB88320;

static RX_BUFFER: Mutex<RefCell<Vec<u8, 4096>>> = Mutex::new(RefCell::new(Vec::new()));
static TX_BUFFER: Mutex<RefCell<Vec<u8, 4096>>> = Mutex::new(RefCell::new(Vec::new()));

pub struct UsbCdcV2Transport;

impl UsbCdcV2Transport {
    /// Send a request frame to the GPU daemon
    pub fn send_request(opcode: u8, req_id: u32, payload: &[u8]) -> Result<(), &'static str> {
        let mut frame = Vec::<u8, 4096>::new();
        frame.extend_from_slice(&MAGIC).map_err(|_| "Buffer full")?;
        frame.extend_from_slice(&req_id.to_le_bytes()).map_err(|_| "Buffer full")?;
        frame.push(opcode);
        frame.push(0); // flags
        frame.extend_from_slice(&(payload.len() as u16).to_le_bytes()).map_err(|_| "Buffer full")?;
        frame.extend_from_slice(payload).map_err(|_| "Buffer full")?;
        let crc = Self::crc32(&frame);
        frame.extend_from_slice(&crc.to_le_bytes()).map_err(|_| "Buffer full")?;

        cortex_m::interrupt::free(|cs| {
            let mut tx = TX_BUFFER.borrow(cs).borrow_mut();
            for &b in frame.iter() {
                tx.push(b).ok();
            }
        });
        Ok(())
    }

    /// Receive a response frame (blocking with timeout)
    pub fn recv_response(timeout_ms: u32) -> Result<(u32, u8, Vec<u8, 4096>), &'static str> {
        let start = Self::get_tick_ms();
        loop {
            let result = cortex_m::interrupt::free(|cs| {
                let mut rx = RX_BUFFER.borrow(cs).borrow_mut();
                if rx.len() < 14 { // minimum frame size
                    return None;
                }
                // Find magic
                let mut magic_pos = None;
                for i in 0..rx.len()-1 {
                    if rx[i] == MAGIC[0] && rx[i+1] == MAGIC[1] {
                        magic_pos = Some(i);
                        break;
                    }
                }
                let pos = magic_pos?;
                if rx.len() < pos + 14 { return None; }

                let req_id = u32::from_le_bytes([rx[pos+2], rx[pos+3], rx[pos+4], rx[pos+5]]);
                let opcode = rx[pos+6];
                let _flags = rx[pos+7];
                let payload_len = u16::from_le_bytes([rx[pos+8], rx[pos+9]]) as usize;

                if rx.len() < pos + 10 + payload_len + 4 { return None; }

                let payload_end = pos + 10 + payload_len;
                let frame_crc = u32::from_le_bytes([rx[payload_end], rx[payload_end+1], rx[payload_end+2], rx[payload_end+3]]);

                let mut payload = Vec::<u8, 4096>::new();
                for i in (pos+10)..payload_end {
                    payload.push(rx[i]).ok()?;
                }

                // Remove processed bytes
                let total_len = payload_end + 4 - pos;
                for _ in 0..total_len {
                    rx.remove(0);
                }

                Some((req_id, opcode, payload))
            });

            if let Some(r) = result {
                return Ok(r);
            }

            if Self::get_tick_ms() - start > timeout_ms {
                return Err("Timeout");
            }
        }
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = !0;
        for &byte in data {
            let mut cur_byte = byte;
            for _ in 0..8 {
                if (crc ^ (cur_byte as u32)) & 1 == 1 {
                    crc = (crc >> 1) ^ CRC32_POLY;
                } else {
                    crc >>= 1;
                }
                cur_byte >>= 1;
            }
        }
        !crc
    }

    fn get_tick_ms() -> u32 {
        // Placeholder: would use DWT or SysTick
        0
    }
}
