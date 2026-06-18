1. **SecretResource + PearPassSecretManager (Prioridade alta) - structs implementation**
   - Write the `SecretResource`, `SecretEntry`, `SecretType`, `SecretAccess` structs and implement the `Resource` trait for `SecretResource` in `cathedral-agent/src/evolution/secret_resource.rs`.
   - Run `cd cathedral-agent && cargo check` to verify the changes compile.

2. **SecretResource + PearPassSecretManager (Prioridade alta) - registry integration**
   - Patch `cathedral-agent/src/evolution/registry.rs` to include `register_secret` and `get_secret` methods in `ResourceRegistry` (note: `ResourceRegistry` is defined in registry.rs that I already wrote).
   - Run `cd cathedral-agent && cargo check` to verify the changes compile.

3. **Melhore o fine_tune_lora_via_qvac com uma interface mais completa - lora structs**
   - Review and write `LoRAConfig`, `LoRAAdapter`, `LoRAMetrics` structs in `cathedral-agent/src/evolution/lora_finetune.rs`. The code was already partially implemented based on the user's snippet.
   - Run `cd cathedral-agent && cargo check` to verify the changes compile.

4. **Melhore o fine_tune_lora_via_qvac com uma interface mais completa - sepl integration**
   - Write `propose_lora` and `commit_lora` implementations in `cathedral-agent/src/evolution/lora_finetune.rs` (which extends `AutogenesisOperator`).
   - Run `cd cathedral-agent && cargo check` to verify the changes compile.

5. **Crie testes unitários para o novo fluxo de inferência híbrida**
   - Write tests for `infer_with_strategy` in `AutogenesisOperator` that cover both QVAC-first and Eve-fallback execution paths in `cathedral-agent/src/evolution/sepl.rs` within a `#[cfg(test)]` block.
   - Run `cd cathedral-agent && cargo test` to verify the tests compile and run.

6. **Adicionar tratamento de erro robusto - retry mechanism**
   - Write `RetryConfig` and `RetryContext` with exponential backoff in `cathedral-agent/src/error_handling/retry.rs`.
   - Run `cd cathedral-agent && cargo check` to verify the changes compile.

7. **Adicionar tratamento de erro robusto - circuit breaker**
   - Write `CircuitBreaker`, `CircuitState`, `CircuitBreakerConfig` in `cathedral-agent/src/error_handling/circuit_breaker.rs`.
   - Run `cd cathedral-agent && cargo check` to verify the changes compile.

8. **Adicionar tratamento de erro robusto - sepl integration**
   - Patch `AutogenesisOperator` (`cathedral-agent/src/evolution/sepl.rs`) to use these robust error-handling mechanisms in `infer_with_strategy` and wrap the Eve cloud fallback with circuit breaking and retries.
   - Run `cd cathedral-agent && cargo check` to verify the changes compile.

9. **Run all tests**
   - Run `cd cathedral-agent && cargo test` to verify the system's integrity.

10. **Pre commit step**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
