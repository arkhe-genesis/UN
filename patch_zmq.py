import re

with open("cathedral_v14/cognitive_substrate.py", "r") as f:
    content = f.read()

import_zmq = """
import os
import zmq
import zmq.asyncio
import json
"""
content = re.sub(r'import asyncio', import_zmq + "import asyncio", content, count=1)

memory_zmq = """class EpisodicMemoryV2:
    \"\"\"
    Memória de longo prazo com indexação HNSW (O(log n)) e Curva de Esquecimento.
    Delega a execução para o Rust Data Plane via ZeroMQ para alta performance.
    \"\"\"
    def __init__(self, dim: int = 64, max_elements: int = 10000, m: int = 64, ef_construction: int = 200):
        self.dim = dim
        self.context = zmq.asyncio.Context()
        self.socket = self.context.socket(zmq.REQ)
        # Em produção conectaríamos no IP correto do Data Plane
        self.socket.connect("tcp://127.0.0.1:5555")

    async def start(self):
        logger.info("Episodic Memory V2 conectado ao Rust Data Plane via ZMQ")

    async def stop(self):
        self.socket.close()
        self.context.term()

    async def store(self, embedding: List[float], metadata: Dict) -> int:
        req = {
            "action": "StoreMemory",
            "embedding": embedding,
            "metadata": metadata
        }
        await self.socket.send_json(req)
        resp = await self.socket.recv_json()
        return resp.get("memory_id", -1)

    async def recall(self, query: List[float], top_k: int = 5, min_similarity: float = 0.7) -> List[Dict]:
        req = {
            "action": "RecallMemory",
            "query": query,
            "top_k": top_k,
            "min_similarity": min_similarity
        }
        await self.socket.send_json(req)
        resp = await self.socket.recv_json()
        return resp.get("results", [])
"""

content = re.sub(r'class EpisodicMemoryV2:.*?class CausalEngine:', memory_zmq + "\n# =============================================================================\n# 3. CAUSAL ENGINE (DoWhy + Counterfactuals)\n# =============================================================================\n\nclass CausalEngine:", content, flags=re.DOTALL)


introspective_zmq = """class IntrospectiveMonitor:
    \"\"\"
    Mede a "saúde cognitiva" do sistema monitorando latência assíncrona,
    erros de inferência e variação de confidence scores.
    Em produção: Solicita self-check Godeliano do Rust Data Plane.
    \"\"\"
    def __init__(self, check_interval_s: float = 5.0, anomaly_window: int = 10):
        self.check_interval = check_interval_s
        self.anomaly_window = anomaly_window
        self._latencies: deque = deque(maxlen=anomaly_window)
        self._errors: deque = deque(maxlen=anomaly_window)
        self._health_score: float = 1.0
        self._task: Optional[asyncio.Task] = None
        self.context = zmq.asyncio.Context()
        self.socket = self.context.socket(zmq.REQ)
        self.socket.connect("tcp://127.0.0.1:5555")
        self.pid = os.getpid()

    async def start(self):
        self._task = asyncio.create_task(self._monitor_loop())

    async def stop(self):
        if self._task:
            self._task.cancel()
            try: await self._task
            except asyncio.CancelledError: pass
        self.socket.close()

    async def _monitor_loop(self):
        \"\"\"Loop de monitoramento que roda paralelo ao loop principal sem bloqueá-lo.\"\"\"
        while True:
            try:
                start = time.monotonic()
                await asyncio.sleep(self.check_interval)
                delay = time.monotonic() - start - self.check_interval
                loop_lag_ms = max(0, (delay / self.check_interval) * 1000)

                self._latencies.append(loop_lag_ms)

                # Check Godeliano no Rust
                req = {
                    "action": "GodelianCheck",
                    "target_pid": self.pid
                }
                await self.socket.send_json(req)
                resp = await self.socket.recv_json()
                rust_healthy = resp.get("healthy", False)

                self._health_score = self._calculate_health()
                if not rust_healthy:
                    self._health_score *= 0.5 # Penalidade forte

                if self._health_score < 0.5:
                    logger.critical("Introspective Monitor: Loop congelado/alucinação! Score: %.2f.", self._health_score)

            except asyncio.CancelledError:
                break

    def record_inference_error(self, error: str):
        self._errors.append(time.time())
        if len(self._errors) > self.anomaly_window:
            logger.error("Introspective Monitor: Picos de erro anormais detectados!")

    def _calculate_health(self) -> float:
        if not self._latencies: return 1.0
        avg_lag = sum(self._latencies) / len(self._latencies)
        lag_penalty = max(0, (avg_lag - 10.0) / 90.0) * 2.0
        recent_errors = sum(1 for t in self._errors if time.time() - t < 60)
        error_penalty = recent_errors * 0.2
        return max(0.0, 1.0 - lag_penalty - error_penalty)
"""

content = re.sub(r'class IntrospectiveMonitor:.*?class EnergyBudgetController:', introspective_zmq + "\n# =============================================================================\n# 6. ENERGY BUDGET CONTROL PLANE\n# =============================================================================\n\nclass EnergyBudgetController:", content, flags=re.DOTALL)


energy_zmq = """class EnergyBudgetController:
    \"\"\"
    Planejamento de energia (Control Plane).
    Delega a execução de DVFS e Sparsity para o Data Plane Rust via ZeroMQ.
    \"\"\"
    def __init__(self, max_watts: float = 20.0, carbon_budget_kwh: float = 1000.0):
        self.max_watts = max_watts
        self.carbon_budget_kwh = carbon_budget_kwh
        self.consumed_kwh = 0.0
        self.current_state = "NORMAL"
        self.context = zmq.asyncio.Context()
        self.socket = self.context.socket(zmq.REQ)
        self.socket.connect("tcp://127.0.0.1:5555")

    async def update_from_gguf_stats(self, tokens_per_sec: float, queue_size: int, cache_hit_rate: float):
        req = {
            "action": "UpdateEnergy",
            "tokens_per_sec": tokens_per_sec,
            "cache_hit_rate": cache_hit_rate
        }
        await self.socket.send_json(req)
        resp = await self.socket.recv_json()

        self.current_state = resp.get("state", "NORMAL")
        estimated_watts = resp.get("estimated_watts", 15.0)

        time_delta_s = 10.0
        self.consumed_kwh += (estimated_watts * time_delta_s) / 3600.0
        remaining = max(0.0, self.carbon_budget_kwh - self.consumed_kwh)

        logger.info("Energy Budget: State=%s, Est. Watts=%.1fW, Budget Remaining=%.2fkWh",
                     self.current_state, estimated_watts, remaining)

    def stop(self):
        self.socket.close()
"""

content = re.sub(r'class EnergyBudgetController:.*?class CognitiveSubstrateOrchestrator:', energy_zmq + "\n# =============================================================================\n# 7. COGNITIVE ORCHESTRATOR (Aglutinador de Todos os Módulos)\n# =============================================================================\n\nclass CognitiveSubstrateOrchestrator:", content, flags=re.DOTALL)

# Adjust orchestrator to await update_from_gguf_stats
content = content.replace("self.energy.update_from_gguf_stats(tokens_per_sec=gguf_tokens / 10.0, queue_size=0, cache_hit_rate=0.5)", "await self.energy.update_from_gguf_stats(tokens_per_sec=gguf_tokens / 10.0, queue_size=0, cache_hit_rate=0.5)")


with open("cathedral_v14/cognitive_substrate.py", "w") as f:
    f.write(content)
