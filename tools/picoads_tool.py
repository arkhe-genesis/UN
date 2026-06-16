from langchain.tools import BaseTool
from pydantic import BaseModel, Field
from typing import Type, Optional
import requests
import json

# Assume you have a DLA client (via HTTP or PyO3)
from dla_py import PyDLA  # your DLA binding

class PicoAdsWithProofInput(BaseModel):
    query: str = Field(...)
    hub: Optional[str] = Field(default=None)
    max_results: int = Field(default=5)
    require_memory_proof: bool = Field(default=True)

class PicoAdsWithMemoryProofTool(BaseTool):
    name = "picoads_get_recommendations_with_proof"
    description = "Get PicoAds recommendations. Requires DLA MemoryProof for high-value/risky recommendations."
    args_schema: Type[BaseModel] = PicoAdsWithProofInput

    def __init__(self, picoads_api_key: str, dla: PyDLA):
        super().__init__()
        self.picoads_api_key = picoads_api_key
        self.dla = dla  # DLA instance with prove_memory_state()

    def _run(self, query: str, hub: Optional[str] = None,
             max_results: int = 5, require_memory_proof: bool = True) -> str:

        memory_commitment = None

        if require_memory_proof:
            try:
                proof = self.dla.prove_memory_state()  # real call
                memory_commitment = proof.merkle_root
                print(f"[DLA] MemoryProof generated: {memory_commitment[:16]}...")
            except Exception as e:
                return json.dumps({"error": f"Failed to generate MemoryProof: {e}"})

        # Call PicoAds
        try:
            headers = {
                "Authorization": f"Bearer {self.picoads_api_key}",
                "Content-Type": "application/json",
            }
            if memory_commitment:
                headers["X-Memory-Commitment"] = memory_commitment

            payload = {
                "query": query,
                "hub": hub,
                "max_results": max_results,
            }

            resp = requests.post(
                "https://picoads.xyz/recommendations",
                json=payload,
                headers=headers,
                timeout=15
            )
            resp.raise_for_status()

            return json.dumps({
                "recommendations": resp.json(),
                "memory_proof_used": bool(memory_commitment),
                "memory_commitment": memory_commitment
            }, indent=2)

        except Exception as e:
            return json.dumps({"error": str(e)})
