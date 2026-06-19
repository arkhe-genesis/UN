# Cathedral ARKHE — Runbook de Operações

**Versão:** v2.1.0
**Data:** 2026-06-19
**Selo:** CATHEDRAL-ARKHE-8000-RUNBOOK-v2.1.0-2026-06-19

---

## 1. Visão Geral

Este runbook documenta os procedimentos operacionais para o ambiente Cathedral ARKHE, cobrindo monitoramento, alertas, resposta a incidentes, manutenção e recuperação.

### 1.1. Componentes Monitorados

| Componente | Porta | Função |
|------------|-------|--------|
| MCP Server (Headroom) | 8787 | Servidor MCP principal |
| Metrics (Prometheus) | 8788 | Exportador de métricas |
| PostgreSQL (WormGraph) | 5432 | Banco de dados imutável |
| Redis (DLQ + Cache) | 6379 | Fila morta e cache |
| Prometheus | 9090 | Coleta de métricas |
| Grafana | 3000 | Dashboards e visualização |

### 1.2. Ferramentas de Acesso

```bash
# Logs do MCP Server
docker logs -f cathedral-mcp

# Shell interativo no container
docker exec -it cathedral-mcp /bin/sh

# Status do serviço
docker-compose -f docker-compose.alpine.yml ps

# Métricas em tempo real
curl http://localhost:8788/metrics
```

---

## 2. Alertas e Critérios de Severidade

| Severidade | Critério | Exemplo |
|------------|----------|---------|
| **P1 (Crítico)** | Interrupção total do serviço | MCP Server down |
| **P2 (Alto)** | Degradação severa | Circuit breaker aberto, DLQ overflow |
| **P3 (Médio)** | Degradação parcial | Alta latência, erro rate > 5% |
| **P4 (Baixo)** | Impacto mínimo | Poison pill detectado |

---

## 3. Procedimentos de Resposta a Incidentes

### 3.1. P1 — MCP Server Down

**Sintoma:** `up{job="cathedral-mcp"} == 0` dispara alerta.

**Ação Imediata:**
1. Verificar se o container está rodando:
   ```bash
   docker ps | grep cathedral-mcp
   ```
2. Se parado, tentar reiniciar:
   ```bash
   docker-compose -f docker-compose.alpine.yml restart cathedral-mcp
   ```
3. Verificar logs:
   ```bash
   docker logs cathedral-mcp --tail 100
   ```
4. Se persistir, verificar recursos do sistema (memória, CPU, disco):
   ```bash
   docker stats
   df -h
   ```

**Escalonamento:** Se não recuperar em 5 minutos, acionar equipe de plantão.

---

### 3.2. P2 — Circuit Breaker Aberto

**Sintoma:** `circuit_breaker_state{state="open"} == 1`

**Ação Imediata:**
1. Identificar componente afetado:
   ```sql
   SELECT component, state, failure_count, triggered_at
   FROM circuit_breaker_events
   WHERE state = 'open'
   ORDER BY triggered_at DESC
   LIMIT 10;
   ```
2. Verificar logs do componente:
   ```bash
   docker logs cathedral-mcp | grep "circuit.*open" | tail -20
   ```
3. Se falhas forem transitórias, resetar o circuit breaker:
   ```bash
   curl -X POST http://localhost:8787/admin/circuit-breaker/reset?component=<component>
   ```
4. Se falhas persistirem, investigar causa raiz (dependência externa, timeout, etc.).

**Escalonamento:** Se circuito abrir mais de 3 vezes em 1 hora, acionar equipe de engenharia.

---

### 3.3. P2 — DLQ Overflow

**Sintoma:** `dlq_size > 1000` por mais de 5 minutos.

**Ação Imediata:**
1. Verificar tamanho e conteúdo da DLQ:
   ```sql
   SELECT COUNT(*) FROM dlq_messages;
   SELECT component, error_type, COUNT(*)
   FROM dlq_messages
   GROUP BY component, error_type
   ORDER BY COUNT(*) DESC;
   ```
2. Se mensagens forem de um único componente, revisar o componente.
3. Purgar mensagens antigas (mais de 24h):
   ```sql
   SELECT wormgraph.purge_dlq_old_messages(1);
   ```
4. Se necessário, reprocessar mensagens com Last-Effort:
   ```bash
   curl -X POST http://localhost:8787/api/dlq/reprocess
   ```

---

### 3.4. P3 — Erro Rate Elevado (> 5%)

**Sintoma:** `rate(mcp_requests_total{status=~"5.."}[5m]) / rate(mcp_requests_total[5m]) > 0.05`

**Ação Imediata:**
1. Identificar endpoints afetados:
   ```bash
   curl -s http://localhost:8788/metrics | grep mcp_requests_total | grep "5.."
   ```
2. Verificar logs:
   ```bash
   docker logs cathedral-mcp | grep ERROR | tail -50
   ```
3. Revisar configurações de timeout e rate limit.
4. Se necessário, aumentar recursos do container.

---

### 3.5. P4 — Poison Pill Detectado

**Sintoma:** Alerta `poison_pill_detected_total` incrementa.

**Ação Imediata:**
1. Identificar mensagem:
   ```sql
   SELECT id, original_id, error_type, error_message
   FROM dlq_messages
   WHERE poison_pill = true
   ORDER BY enqueued_at DESC
   LIMIT 1;
   ```
2. Analisar padrão de falha.
3. Se for um caso isolado, reconhecer e arquivar:
   ```bash
   curl -X POST http://localhost:8787/api/dlq/acknowledge/<id>
   ```
4. Se for recorrente, escalar para equipe de engenharia.

---

## 4. Manutenção de Banco de Dados (WormGraph)

### 4.1. Compactação e Vacuum

O WormGraph cresce continuamente. Recomenda-se:

```sql
-- Compactar tabelas
VACUUM ANALYZE wormgraph_events;
VACUUM ANALYZE dlq_messages;
VACUUM ANALYZE agent_memory;

-- Reindexar (se necessário)
REINDEX INDEX idx_wormgraph_events_created_at;
```

### 4.2. Purga de Dados Antigos

```sql
-- Remover eventos com mais de 30 dias
DELETE FROM wormgraph_events
WHERE created_at < NOW() - INTERVAL '30 days';

-- Remover mensagens da DLQ com mais de 7 dias
DELETE FROM dlq_messages
WHERE enqueued_at < NOW() - INTERVAL '7 days';
```

### 4.3. Backup

```bash
# Backup completo
docker exec cathedral-postgres pg_dump -U cathedral cathedral > backup_$(date +%Y%m%d_%H%M%S).sql

# Backup incremental (WAL)
docker exec cathedral-postgres pg_basebackup -D /backup -Ft -z -P
```

---

## 5. Gestão de Capacidade

### 5.1. Métricas de Referência

| Métrica | Valor Alvo | Ação se Ultrapassar |
|---------|------------|---------------------|
| CPU MCP Server | < 70% | Aumentar CPU limits |
| Memória MCP Server | < 80% | Aumentar memory limits |
| Conexões PostgreSQL | < 50 | Aumentar pool de conexões |
| DLQ Size | < 500 | Purga automática |
| Espaço em disco | < 80% | Limpar logs antigos |

### 5.2. Escalonamento Horizontal

Se carga ultrapassar limites consistentes:
1. Aumentar réplicas do MCP Server:
   ```yaml
   deploy:
     replicas: 3
   ```
2. Aumentar recursos do PostgreSQL:
   ```yaml
   deploy:
     resources:
       limits:
         memory: 2G
         cpus: '2.0'
   ```

---

## 6. Backup e Recuperação

### 6.1. Estratégia de Backup

| Componente | Frequência | Retenção | Ferramenta |
|------------|------------|----------|------------|
| PostgreSQL | Diária (full) + WAL (15 min) | 30 dias | pg_dump + pg_basebackup |
| Redis | Diária (RDB) | 7 dias | redis-cli --rdb |
| Configurações | Sempre | Indefinida | Git |
| CCR Cache | Não (dados efêmeros) | N/A | N/A |

### 6.2. Recuperação de Desastre

**Recuperação do PostgreSQL:**

```bash
# Parar serviços que dependem do banco
docker stop cathedral-mcp

# Restaurar backup mais recente
docker exec -i cathedral-postgres psql -U cathedral < backup_latest.sql

# Subir novamente
docker start cathedral-mcp
```

**Recuperação do Redis:**

```bash
# Parar Redis
docker stop cathedral-redis

# Copiar backup RDB
cp /backup/redis.rdb /var/lib/docker/volumes/cathedral_redis-data/_data/dump.rdb

# Reiniciar Redis
docker start cathedral-redis
```

---

## 7. Procedimentos de Escalonamento

| Nível | Responsável | Contato |
|-------|-------------|---------|
| Nível 1 | Operador de Plantão | plantao@cathedral.arkhe |
| Nível 2 | Engenheiro Sênior | eng-senior@cathedral.arkhe |
| Nível 3 | Arquiteto de Sistemas | arquiteto@cathedral.arkhe |
| Nível 4 | Comitê de Governança | governance@cathedral.arkhe |

### 7.1. Matriz de Escalonamento por Severidade

| Severidade | Tempo para Escalar | Nível |
|------------|--------------------|-------|
| P1 | Imediato | Nível 2 |
| P2 | 15 minutos | Nível 2 |
| P3 | 1 hora | Nível 1 |
| P4 | 4 horas | Nível 1 |

---

## 8. Checklist de Health Check (Diário)

- [ ] MCP Server está rodando (`docker ps | grep cathedral-mcp`)
- [ ] Métricas estão acessíveis (`curl http://localhost:8788/metrics | head -20`)
- [ ] PostgreSQL está saudável (`docker exec cathedral-postgres pg_isready`)
- [ ] Redis está respondendo (`docker exec cathedral-redis redis-cli ping`)
- [ ] Prometheus está coletando (`curl http://localhost:9090/-/healthy`)
- [ ] Grafana está acessível (`curl http://localhost:3000/api/health`)
- [ ] DLQ Size < 500 (`SELECT COUNT(*) FROM dlq_messages;`)
- [ ] Circuit Breakers fechados (`SELECT COUNT(*) FROM circuit_breaker_events WHERE state = 'open';`)
- [ ] Última execução de backup foi bem-sucedida (`ls -la /backup/ | tail -1`)

---

## 9. Comandos Úteis

```bash
# --- MCP Server ---
# Verificar status
./scripts/alpine-start.sh status

# Reiniciar com grace period
./scripts/alpine-start.sh restart

# Ver logs em tempo real
./scripts/alpine-start.sh logs

# --- Banco de Dados ---
# Conectar ao PostgreSQL
docker exec -it cathedral-postgres psql -U cathedral

# Ver estatísticas de tabelas
SELECT schemaname, tablename, n_live_tup, n_dead_tup
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC;

# --- Redis ---
# Monitorar filas
docker exec -it cathedral-redis redis-cli
> LLEN dlq:queue
> INFO memory

# --- Prometheus ---
# Ver targets
curl http://localhost:9090/api/v1/targets

# --- Grafana ---
# Resetar senha
docker exec -it cathedral-grafana grafana-cli admin reset-admin-password cathedral
```

---

## 10. Manutenção Programada

| Atividade | Frequência | Janela | Responsável |
|-----------|------------|--------|-------------|
| Compactação do Banco | Semanal | Domingo 02:00-04:00 | DBA |
| Purga de Logs | Diária | 03:00 | Operador |
| Atualização de Imagens | Mensal | 1ª Sábado 01:00-06:00 | Engenharia |
| Backup Completo | Diária | 01:00 | Operador |
| Simulação de Desastre | Trimestral | Fora do horário | Engenharia |

---

**Selo:** CATHEDRAL-ARKHE-8000-RUNBOOK-v2.1.0-2026-06-19
**Próxima Revisão:** 2026-07-19
**Aprovado por:** Cathedral ARKHE Operations Core
