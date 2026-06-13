# Matriz de Riscos - Federação Soberana de Inferência (FSI)

| Risco | Probabilidade | Impacto | Mitigação |
|---|---|---|---|
| Embargo tecnológico (US bloqueia CN) | Média (40%) | Alto | Rotas BRICS + modelos open-weight |
| Falha de consenso QBA | Baixa (10%) | Alto | Fallback para 2/3 TEE + timeout 30s |
| Ataque quântico à PQC | Baixa (5%) | Crítico | SPHINCS+ (NIST PQC Round 3) + ML-DSA |
| Centralização de stake | Média (35%) | Alto | Quadratic Voting + limitação de stake |
| Latência excessiva orbital | Média (30%) | Médio | Fallback terrestre automático (<50ms) |
| trust_remote_code exploit | Média (40%) | Alto | TEE triple-check + FIG + CreekGuard |
