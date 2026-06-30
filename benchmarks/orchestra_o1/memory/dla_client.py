import requests
import pickle
import base64

class DLAMemoryClient:
    def __init__(self, api_base: str = "http://localhost:8000"):
        self.api = api_base

    def store_state(self, state_vector: list) -> str:
        """Envia um vetor de estado (ex: embedding da conversa) para o DLA."""
        payload = {"vector": state_vector, "timestamp": None}
        resp = requests.post(f"{self.api}/v1/dla/step", json=payload)
        return resp.json().get("state_id")

    def get_context(self) -> list:
        """Recupera o contexto comprimido atual (média ponderada)."""
        resp = requests.get(f"{self.api}/v1/dla/context")
        return resp.json().get("context")

    def reset(self):
        """Reseta a memória DLA (novo contexto)."""
        requests.post(f"{self.api}/v1/dla/reset")
