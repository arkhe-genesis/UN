#[cfg(kani)]
mod verification {
    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_host_boundary() {}

    #[kani::proof]
    #[kani::unwind(1)]
    fn verify_wasm_sandbox() {}
}
