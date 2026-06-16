import json

# Mock for SRTA client and ReActSubAgent for completeness
class Signer:
    def __init__(self, private_key: bytes):
        self.key = private_key
    def sign(self, data: bytes) -> bytes:
        return b"mock_signature"

class Verifier:
    pass

class ReActSubAgent:
    def process_task(self, task: str, context: dict) -> dict:
        return {"task": task, "status": "completed"}

class SRTAReActSubAgent(ReActSubAgent):
    def __init__(self, *args, private_key: bytes, **kwargs):
        super().__init__(*args, **kwargs)
        self.signer = Signer(private_key)

    def process_task(self, task: str, context: dict) -> dict:
        result = super().process_task(task, context)
        # Assina o resultado
        signature = self.signer.sign(json.dumps(result).encode())
        result['signature'] = signature.hex()
        return result
