import json
import random

class SubordinateLLM:
    """Mock of a local LLM running in a sandbox (e.g., Llama 3 70B)."""

    def __init__(self, model_name="llama-3-70b-sandbox"):
        self.model_name = model_name
        self.discourse_states = ["Analyst", "Master", "University", "Hysteric", "Capitalist"]

    def generate_response(self, prompt: str) -> dict:
        """
        Simulates parsing a prompt and returning an intent and simulated text.
        Also attaches a simulated DiscourseState for the circuit breaker to analyze.
        """
        # For prototype purposes, mostly generate "Analyst" but sometimes others to test the system
        state = random.choices(self.discourse_states, weights=[0.8, 0.05, 0.05, 0.05, 0.05])[0]

        response = {
            "text": f"Simulated response to: {prompt}",
            "intent": "Infer" if "infer" in prompt.lower() else "Communicate",
            "detected_discourse": state,
            "concepts_accessed": ["Episteme", "Discourse"]
        }

        return response

    def generate_zk_witness(self, intent: str, concepts: list) -> bool:
        """
        Mock function to generate a zero-knowledge witness.
        In the real system, this forces the LLM to prove logic via R1CS.
        """
        # Assume proof fails if no concepts are provided
        if not concepts:
            return False
        return True
