//! BCH(63,51) t=2 – Full Berlekamp-Massey decoder with real discrepancy calculation
//! Supports correction of up to 2 errors per 63-bit codeword

use heapless::Vec;

const N: usize = 63;  // codeword length
const K: usize = 51;  // message length
const T: usize = 2;   // error correction capability

// GF(2^6) primitive polynomial: x^6 + x + 1
const PRIM_POLY: u8 = 0x43;

// Precomputed log/antilog tables for GF(2^6)
const GF_LOG: [u8; 256] = {
    let mut log = [0u8; 256];
    let mut a = 1u8;
    let mut i = 0u8;
    while i < 63 {
        log[a as usize] = i;
        // multiply by alpha (2)
        let mut aa = a as u16;
        aa <<= 1;
        if (aa & 0x40) != 0 {
            aa ^= PRIM_POLY as u16;
        }
        a = aa as u8;
        i += 1;
    }
    log
};

const GF_ALOG: [u8; 256] = {
    let mut alog = [0u8; 256];
    let mut a = 1u8;
    let mut i = 0u8;
    while i < 63 {
        alog[i as usize] = a;
        let mut aa = a as u16;
        aa <<= 1;
        if (aa & 0x40) != 0 {
            aa ^= PRIM_POLY as u16;
        }
        a = aa as u8;
        i += 1;
    }
    alog
};

fn gf_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 { return 0; }
    let sum = (GF_LOG[a as usize] as u16) + (GF_LOG[b as usize] as u16);
    GF_ALOG[(sum % 63) as usize]
}

fn gf_inv(a: u8) -> u8 {
    if a == 0 { return 0; }
    GF_ALOG[(63 - GF_LOG[a as usize]) as usize]
}

fn gf_pow(base: u8, exp: u8) -> u8 {
    let mut r = 1u8;
    for _ in 0..exp {
        r = gf_mul(r, base);
    }
    r
}

fn codeword_to_bits(cw: &[u8; 8]) -> [u8; N] {
    let mut bits = [0u8; N];
    for i in 0..8 {
        for j in 0..8 {
            if i * 8 + j < N {
                bits[i * 8 + j] = (cw[i] >> (7 - j)) & 1;
            }
        }
    }
    bits
}

fn extract_message(bits: &[u8; N]) -> [u8; 7] {
    let mut msg = [0u8; 7];
    for i in 0..K {
        let byte_idx = i / 8;
        let bit_idx = 7 - (i % 8);
        if bits[i] == 1 {
            msg[byte_idx] |= 1 << bit_idx;
        }
    }
    msg
}

fn calculate_syndromes(bits: &[u8; N]) -> [u8; 2 * T] {
    let mut s = [0u8; 2 * T];
    for i in 0..N {
        // bits[62 - i] corresponds to x^i
        if bits[N - 1 - i] == 1 {
            for j in 0..2 * T {
                let power = (i * (j + 1)) as u8;
                s[j] ^= gf_pow(2, power);
            }
        }
    }
    s
}

/// Berlekamp-Massey algorithm for binary BCH codes
/// Returns error locator polynomial coefficients [Λ0, Λ1, ..., Λt]
fn berlekamp_massey(syndromes: &[u8; 2 * T]) -> Vec<u8, 4> {
    let mut lambda = Vec::<u8, 4>::new();
    lambda.push(1).unwrap();  // Λ(x) = 1

    let mut b = Vec::<u8, 4>::new();
    b.push(1).unwrap();  // B(x) = 1

    let mut l = 0usize;  // current number of errors
    let mut m = 1usize;  // shift

    for r in 0..(2 * T) {
        // Calculate discrepancy Δ
        let mut delta = 0u8;
        for i in 0..=l {
            if i < lambda.len() {
                delta ^= gf_mul(lambda[i], syndromes[r - i]);
            }
        }

        if delta != 0 {
            // Save current lambda
            let mut temp = Vec::<u8, 4>::new();
            for i in 0..lambda.len() {
                temp.push(lambda[i]).unwrap();
            }

            // Update lambda: Λ(x) = Λ(x) + Δ * x^m * B(x)
            for i in 0..b.len() {
                let idx = i + m;
                let term = gf_mul(delta, b[i]);
                if idx < lambda.len() {
                    lambda[idx] ^= term;
                } else {
                    while lambda.len() <= idx {
                        lambda.push(0).unwrap();
                    }
                    lambda[idx] = term;
                }
            }

            // Update B(x) and l
            if 2 * l <= r {
                l = r + 1 - l;
                let inv_delta = gf_inv(delta);
                for x in temp.iter_mut() {
                    *x = gf_mul(*x, inv_delta);
                }
                b = temp;
                m = 1;
            } else {
                m += 1;
            }
        } else {
            m += 1;
        }
    }

    // Trim leading zeros
    while lambda.len() > 1 && lambda.last() == Some(&0) {
        lambda.pop();
    }

    lambda
}

/// Chien search: find error locations by evaluating error locator polynomial
fn chien_search(lambda: &[u8]) -> Vec<u8, T> {
    let mut errors = Vec::<u8, T>::new();

    for i in 0..N {
        let x = gf_pow(2, i as u8);
        let mut eval = 0u8;

        for (idx, &coeff) in lambda.iter().enumerate() {
            let x_pow = gf_pow(x, idx as u8);
            eval ^= gf_mul(coeff, x_pow);
        }

        if eval == 0 {
            // If alpha^i is a root, error pos in polynomial is (63 - i) % 63.
            // Which corresponds to array index 62 - pos.
            let pos = (63 - i) % 63;
            let array_idx = 62 - pos;
            errors.push(array_idx as u8).unwrap();
            if errors.len() == T {
                break;
            }
        }
    }

    errors
}

/// Decode BCH(63,51) codeword, correcting up to 2 errors
pub fn bch_decode(codeword: &[u8; 8]) -> Option<[u8; 7]> {
    let bits = codeword_to_bits(codeword);
    let syndromes = calculate_syndromes(&bits);

    // No errors
    if syndromes.iter().all(|&x| x == 0) {
        return Some(extract_message(&bits));
    }

    // Find error locator polynomial
    let lambda = berlekamp_massey(&syndromes);
    let errors = chien_search(&lambda);

    if errors.is_empty() {
        return None;
    }

    // Correct errors
    let mut corrected = bits;
    for pos in errors {
        corrected[pos as usize] ^= 1;
    }

    // Verify correction
    let syndromes_check = calculate_syndromes(&corrected);
    if syndromes_check.iter().all(|&x| x == 0) {
        Some(extract_message(&corrected))
    } else {
        None
    }
}

/// Encode message into BCH(63,51) codeword
pub fn bch_encode(message: &[u8; 7]) -> [u8; 8] {
    let mut bits = [0u8; N];
    for i in 0..K {
        let byte_idx = i / 8;
        let bit_idx = 7 - (i % 8);
        bits[i] = (message[byte_idx] >> bit_idx) & 1;
    }

    let mut cw_bits = bits;
    // generator polynomial g(x) = x^12 + x^10 + x^8 + x^5 + x^4 + x^3 + 1
    // binary: 1010100111001 (0x1539)
    let g = [1,0,1,0,1,0,0,1,1,1,0,0,1];

    for i in 0..K {
        if cw_bits[i] == 1 {
            for j in 0..13 {
                cw_bits[i + j] ^= g[j];
            }
        }
    }

    // cw_bits[K..N] now contains the parity
    for i in K..N {
        bits[i] = cw_bits[i];
    }

    let mut cw = [0u8; 8];
    for i in 0..N {
        if bits[i] == 1 {
            let byte_idx = i / 8;
            let bit_idx = 7 - (i % 8);
            cw[byte_idx] |= 1 << bit_idx;
        }
    }
    cw
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_no_error() {
        let msg = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xC0];
        let enc = bch_encode(&msg);
        let dec = bch_decode(&enc).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn test_correct_single_error() {
        let msg = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xE0];
        let mut enc = bch_encode(&msg);
        enc[0] ^= 0x80; // flip one bit
        let dec = bch_decode(&enc).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn test_correct_two_errors() {
        let msg = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00];
        let mut enc = bch_encode(&msg);
        enc[1] ^= 0x01; // flip bit in byte 1
        enc[4] ^= 0x02; // flip bit in byte 4
        let dec = bch_decode(&enc).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn test_uncorrectable_error() {
        let msg = [0x00; 7];
        let mut enc = bch_encode(&msg);
        // Flip 3 bits (beyond correction capability)
        enc[0] ^= 0xFF;
        enc[1] ^= 0xFF;
        enc[2] ^= 0xFF;
        let result = bch_decode(&enc);
        assert!(result.is_none() || result.unwrap() != msg);
    }
}
