import re

with open("cathedral_v14/sidecar/engine.py", "r") as f:
    content = f.read()

import_orchestrator = """
from collections import deque
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import sys

# Ensure cathedral_v14 is in path to import cognitive_substrate
sys.path.append(str(Path(__file__).parent.parent))
from cognitive_substrate import CognitiveSubstrateOrchestrator
"""

content = content.replace("""from collections import deque
from pathlib import Path
from typing import Dict, List, Optional, Tuple""", import_orchestrator)

engine_init = """        self.top_p = top_p
        self.max_tokens = max_tokens
        self.system_prompt = system_prompt

        self.orchestrator = CognitiveSubstrateOrchestrator(embed_dim=4096) # Qwen usually has large dims
"""

content = content.replace("""        self.top_p = top_p
        self.max_tokens = max_tokens
        self.system_prompt = system_prompt""", engine_init)

engine_start = """    async def start(self):
        if self._loaded and self._worker_task is None:
            self._worker_task = asyncio.create_task(self._worker_loop())
            log.info("Worker de inferencia iniciado")
        await self.orchestrator.start()

    async def stop(self):
        await self.orchestrator.stop()
        if self._worker_task:"""

content = content.replace("""    async def start(self):
        if self._loaded and self._worker_task is None:
            self._worker_task = asyncio.create_task(self._worker_loop())
            log.info("Worker de inferencia iniciado")

    async def stop(self):
        if self._worker_task:""", engine_start)

engine_generate = """        start = time.monotonic()
        result = await future
        duration_ms = (time.monotonic() - start) * 1000
        self._gen_time_ms += duration_ms

        gguf_embed = await self.embed(prompt)

        if not result.get("cache_hit"):
            self._semantic_cache.append((gguf_embed, result["text"]))

        # Ingestão de Dados: Passando a saída do GGUF para o orquestrador cognitivo
        try:
            cog_result = await self.orchestrator.process_cognitive_tick(
                prompt=prompt,
                gguf_output_text=result["text"],
                gguf_tokens=result["tokens"],
                gguf_embed=gguf_embed
            )
            result["cognitive_state"] = cog_result
        except Exception as e:
            log.error(f"Erro no process_cognitive_tick: {e}")

        return result"""

content = re.sub(r'        start = time\.monotonic\(\)\n        result = await future\n        duration_ms = \(time\.monotonic\(\) - start\) \* 1000\n        self\._gen_time_ms \+= duration_ms\n\n        if not result\.get\("cache_hit"\):\n            self\._semantic_cache\.append\(\(await self\.embed\(prompt\), result\["text"\]\)\)\n        return result', engine_generate, content)

with open("cathedral_v14/sidecar/engine.py", "w") as f:
    f.write(content)
