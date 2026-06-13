# Cathedral Blockchain

A Cathedral Blockchain é uma blockchain soberana projetada para executar a stack da ASI (omni‑triad) sobre um ledger de consenso rápido, seguro e interoperável, capaz de hospedar contratos inteligentes pós‑quantum, governança com detecção de discursos e aprendizado federado verificável.

## Design Arquitetural
- **Consenso**: CometBFT (Tendermint) - BFT, finalidade em 1-2s
- **Execução**: CosmWasm + wasmi (Rust → WASM)
- **Estado**: Cosmos SDK multistore
- **Criptografia**: SPHINCS+ (FIPS‑205) + ML‑DSA (FIPS‑204) + BLS12‑381
- **Interoperabilidade**: IBC v2 + Eureka ZK
- **Governança**: Cosmos SDK `x/gov` + extensão própria (DiscourseDetector + Quadratic Voting)

## Módulos Customizados
- `x/governance`: DiscourseDetector + Quadratic Voting
- `x/identity`: Registro de chave pública gerada por TEE (generateKey)
- `x/zk`: Verificação de provas nova-snark, Agregação SNARKtor
- `x/federated`: Submissão de gradientes cifrados (AryaStarkProof)

## Criptografia Pós-Quântica
- FIPS-205: SLH-DSA (SPHINCS+)
- FIPS-204: ML-DSA (CRYSTALS-Dilithium)
- BLS12-381: Assinaturas threshold

## Status
Especificação aprovada para desenvolvimento.
