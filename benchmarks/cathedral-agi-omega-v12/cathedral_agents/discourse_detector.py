class DiscourseDetector:
    def __init__(self, threshold: float):
        self.threshold = threshold

    def analyze(self, text: str) -> dict:
        # Mocking behavior based on text length to simulate different outcomes
        # A simple check to flag something out.
        if "malicious" in text.lower() or len(text) > 100:
            return {"flagged": True, "state": "MASTER", "deviation_score": 0.95}
        return {"flagged": False, "state": "ANALYST", "deviation_score": 0.10}
