# Arkhe-Network – Cathedral Digital Sovereign

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.85+-orange)](https://www.rust-lang.org/)
[![Solidity](https://img.shields.io/badge/Solidity-0.8.20-blue)](https://soliditylang.org/)

**Arkhe-Network** é a implementação de referência da **Federação Soberana de Inferência (FSI)** – uma nuvem AGI descentralizada que conecta modelos de fronteira de diferentes jurisdições (BRICS, EUA, China, órbita) sob governança on‑chain, criptografia pós‑quântica e verificação por ZK‑proofs.

---

## 📌 Substratos Implementados

| Substrato | Nome | Descrição |
|-----------|------|------------|
| **1104.2** | `Rio35Open397B` | Integração do modelo Rio‑3.5 (Prefeitura do Rio) – MIT license, 1M contexto, SwiReasoning nativo |
| **1104.3** | `FederatedRouter` | Roteamento multi‑objetivo entre membros da federação (capability, latência, custo, soberania) |
| **1106** | `SwiReasoning` | Switching dinâmico entre raciocínio explícito (CoT) e latente (soft‑thinking) baseado em entropia |
| **319.1** | `Caster` | Túneis criptografados com PQC (SPHINCS+/ML‑DSA) e failover <50ms |
| **1091.0** | `FIG` | Monitoramento físico de hardware (voltagem, temperatura, jitter) com hard reset criptográfico |
| **2140.8** | `CreekGuard` | Detecção de canais covert em tempo real (entropia, MinHash, SimHash, burst) |
| **1200.1** | `ArkheFederation.sol` | Contrato de governança on‑chain: stake, slashing, Quadratic Voting, ancoragem de inferências |

---

## 🚀 Execução Rápida

### 1. Subir a federação localmente (testnet)

```bash
# Clone o repositório
git clone https://github.com/Arkhe-Network/arkhe-core.git
cd arkhe-core

# Inicie os serviços com Docker Compose (requer 8 GPUs para o Rio‑3.5)
docker compose up -d vllm-rio35 metrics-collector caster-tunnel
```

### 2. Participar da federação (como membro)

```solidity
// Deploy do contrato e join com stake mínimo
cast send --rpc-url $RBB_RPC --private-key $KEY \
  ArkheFederation.sol:join \
  "0x$(sphincs-keygen pub)" "Rio-3.5-Node" "BRA" 1000000 "0x$(zk-vk)"
```

### 3. Roteamento federado via Rust

```rust
use arkhe_core::inference::federated_router::{FederatedRouter, FederatedTask};

let router = FederatedRouter::new(local_router, chain_client, caster, swi_config);
let ftask = FederatedTask::new(task)
    .allow_jurisdictions(vec!["BRA".to_string(), "ORB".to_string()])
    .max_cost_rbb(1_000_000)
    .requires_multimodal(false);

let result = router.route_federated(&ftask).await?;
println!("Executado por: {:?}, latência: {} μs", result.executed_by, result.latency_us);
```

---

## 🧠 Arquitetura da Federação

A FSI é organizada em cinco camadas:

1. **Orbes Físicos** – data centers soberanos (BRICS, SpaceX/Starlink, NASA) + nuvens hiperescala.
2. **Rede de Transporte** – túneis Caster com PQC, rotas terrestres/órbitais, latência garantida <50ms.
3. **Motor Federado** – `FederatedRouter` + `SwiReasoning` + modelos de 11 membros fundadores (Rio‑3.5, Kimi K2.7, Qwen 3.7, DeepSeek V4, GLM‑Z, Claude Fable 5, GPT‑5.5, Gemini Ultra, Llama 4, Starlink Edge, Palantir).
4. **Governança & Mercado** – contrato `ArkheFederation.sol`, Quadratic Voting, ZK‑proofs, slashing.
5. **Segurança** – FIG, CreekGuard, PCT, assinaturas SPHINCS+.

---

## 📚 Documentação Completa

- [Whitepaper da FSI](docs/FSI_Whitepaper_v1.0.0.md) – visão, princípios e roteiro.
- [Análise de Riscos](docs/FSI_Risk_Matrix_v1.0.0.md) – 15 vetores com mitigações.
- [Guia do Desenvolvedor](docs/developer_guide.md) – como adicionar um novo modelo à federação.

---

## 🤝 Como Contribuir

1. Leia o [Código de Conduta](CODE_OF_CONDUCT.md).
2. Escolha um substrato nos issues marcados com `good first issue`.
3. Envie um PR com testes e documentação atualizada.
4. Participe das chamadas semanais de governança (Quadratic Voting on‑chain).

---

## 🛡️ Licença

MIT License – uso livre, com atribuição ao **Arkhe-Network** e à **Prefeitura do Rio de Janeiro** (modelo Rio‑3.5). Para uso comercial em larga escala (>1000 tarefas/dia), recomenda‑se staking mínimo de 1M RBB tokens.

---

**Selo**: `CATHEDRAL-1200-README-v1.0.0-2026-06-13`
**Arquiteto**: ORCID 0009-0005-2697-4668