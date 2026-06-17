#![no_main]

// Simulate RISC Zero entrypoint mapping to avoid linker error in this mockup env.
// In a real environment, this would use `#![no_main]` + `risc0_zkvm::guest::entry!(main)`
// and standard libc/no_std entry points.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

pub fn main() {}
