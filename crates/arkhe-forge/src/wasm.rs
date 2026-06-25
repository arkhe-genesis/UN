#[cfg(kani)]
mod verification {
    #[kani::proof]
    fn verify_host_boundary() {}

    #[kani::proof]
    fn verify_wasm_sandbox() {}
}
