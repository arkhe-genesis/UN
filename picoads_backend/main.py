from fastapi import FastAPI, HTTPException, Header
from pydantic import BaseModel
from typing import Optional, List
import httpx
import json

# Import your DLA binding (PyO3)
try:
    from dla_py import PyDLA
    DLA = PyDLA()  # or pass config
    HAS_DLA = True
except ImportError:
    HAS_DLA = False
    print("[Warning] DLA binding not available. Memory proof will be simulated.")

app = FastAPI(title="PicoAds Gateway with Memory Proof")

class RecommendationRequest(BaseModel):
    query: str
    hub: Optional[str] = None
    max_results: int = 5
    require_memory_proof: bool = True

@app.post("/picoads/recommendations")
async def get_recommendations(
    req: RecommendationRequest,
    authorization: str = Header(...),
    x_memory_commitment: Optional[str] = Header(None),
):
    if not authorization.startswith("Bearer "):
        raise HTTPException(status_code=401, detail="Invalid API key")

    api_key = authorization.split(" ")[1]

    memory_commitment = None

    # === Call DLA prove_memory_state() if required ===
    if req.require_memory_proof:
        if not HAS_DLA:
            # Fallback for development
            memory_commitment = "simulated_memory_commitment_" + str(hash(req.query))[:16]
        else:
            try:
                proof = DLA.prove_memory_state()  # real call to your DLA
                memory_commitment = proof.merkle_root
                print(f"[DLA] MemoryProof generated for query: {req.query[:30]}...")
            except Exception as e:
                raise HTTPException(
                    status_code=500,
                    detail=f"Failed to generate MemoryProof: {str(e)}"
                )

    # Forward to PicoAds (real or mock)
    async with httpx.AsyncClient() as client:
        try:
            headers = {
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            }
            if memory_commitment:
                headers["X-Memory-Commitment"] = memory_commitment

            payload = {
                "query": req.query,
                "hub": req.hub,
                "max_results": req.max_results,
            }

            response = await client.post(
                "https://picoads.xyz/recommendations",  # or your mock
                json=payload,
                headers=headers,
                timeout=15.0,
            )
            response.raise_for_status()
            data = response.json()

            return {
                "recommendations": data,
                "memory_proof_used": bool(memory_commitment),
                "memory_commitment": memory_commitment,
            }

        except Exception as e:
            raise HTTPException(status_code=502, detail=f"PicoAds error: {str(e)}")
