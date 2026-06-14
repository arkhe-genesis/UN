import pytest
from aiohttp import web
from aiohttp.test_utils import AioHTTPTestCase
import aiohttp

from sidecar.client import GgufSidecarClient, CircuitBreaker

# Mock da API Server para testar o Client e CircuitBreaker
async def mock_generate(request):
    auth = request.headers.get("Authorization")
    if auth != "Bearer test-token":
        return web.json_response({"detail": "Unauthorized"}, status=401)

    body = await request.json()
    prompt = body.get("prompt", "")

    if "timeout" in prompt:
        import asyncio
        await asyncio.sleep(0.5) # Simula timeout

    if "error" in prompt:
        return web.json_response({"detail": "Internal Server Error"}, status=500)

    return web.json_response({
        "text": f"Mock response for: {prompt[:10]}",
        "tokens": 42,
        "cache_hit": False
    })

class ClientIntegrationTestCase(AioHTTPTestCase):
    async def get_application(self):
        app = web.Application()
        app.router.add_post('/v1/generate', mock_generate)
        return app

    async def setUpAsync(self):
        # Explicitly call super() to setup server
        await super().setUpAsync()
        self.server_url = f"http://{self.server.host}:{self.server.port}"
        self.client = GgufSidecarClient({
            "sidecar_url": self.server_url,
            "sidecar_token": "test-token",
            "sidecar_timeout_s": 0.2, # Timeout bem curto para testar
            "circuit_max_failures": 2,
            "circuit_recovery_s": 0.5,
            "sidecar_slow_threshold_s": 0.1
        })

    async def tearDownAsync(self):
        await self.client.close()
        await super().tearDownAsync()

    async def test_successful_generation(self):
        resp = await self.client.generate("Hello world")
        self.assertIn("Mock response", resp["text"])
        self.assertEqual(resp["tokens"], 42)
        self.assertEqual(self.client.circuit._state, "CLOSED")

    async def test_auth_failure(self):
        self.client.token = "wrong-token"
        resp = await self.client.generate("Hello world")
        self.assertIn("FALLBACK", resp["text"])
        self.assertIn("Auth Failed", resp["text"])

    async def test_circuit_breaker_timeout_and_recovery(self):
        # 1. Primeira falha de timeout
        resp1 = await self.client.generate("timeout please", max_retries=0)
        self.assertIn("FALLBACK", resp1["text"])
        self.assertEqual(self.client.circuit._failures, 1)
        self.assertEqual(self.client.circuit._state, "CLOSED")

        # 2. Segunda falha de timeout (atinge max_failures=2)
        resp2 = await self.client.generate("timeout please", max_retries=0)
        self.assertEqual(self.client.circuit._state, "OPEN")

        # 3. Requisição imediata deve ser bloqueada pelo circuit breaker
        resp3 = await self.client.generate("normal request")
        self.assertIn("Circuit breaker OPEN", resp3["text"])

        # 4. Aguarda o tempo de recovery (0.5s)
        import asyncio
        await asyncio.sleep(0.6)

        # 5. Próxima requisição deve passar (HALF_OPEN) e fechar o circuito com sucesso
        resp4 = await self.client.generate("normal request", max_retries=0)
        self.assertNotIn("FALLBACK", resp4["text"])
        self.assertEqual(self.client.circuit._state, "CLOSED")
