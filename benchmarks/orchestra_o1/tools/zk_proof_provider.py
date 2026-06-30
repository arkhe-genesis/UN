import requests
import json
from typing import Dict, Any

class ZKProofProvider:
    """Tool provider para gerar provas ZK via Cathedral ARKHE."""

    def __init__(self, api_base_url: str = "http://localhost:8000"):
        self.api_base = api_base_url

    def prove_balance(self, balance: int, threshold: int, recipient: str) -> Dict[str, Any]:
        """Gera prova de que balance >= threshold sem revelar valor exato."""
        payload = {"balance": balance, "threshold": threshold, "recipient": recipient}
        response = requests.post(f"{self.api_base}/v1/prove/balance", json=payload)
        response.raise_for_status()
        return response.json()  # contém token, merkle_root, challenges, etc.

    def prove_memory_state(self, dla_state: Dict[str, Any]) -> Dict[str, Any]:
        """Gera prova do estado atual da memória DLA."""
        response = requests.post(f"{self.api_base}/v1/prove/memory", json=dla_state)
        response.raise_for_status()
        return response.json()

    def prove_consent(self, consent_data: Dict[str, Any]) -> Dict[str, Any]:
        """Gera ConsentTokenV3 para uma ação autorizada."""
        response = requests.post(f"{self.api_base}/v1/consent/prove", json=consent_data)
        response.raise_for_status()
        return response.json()
