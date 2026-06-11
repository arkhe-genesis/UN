import uuid
from typing import Dict, Any

class ThresholdGovernance:
    def __init__(self, node_id: str, n: int, t: int):
        self.node_id = node_id
        self.n = n
        self.t = t
        self.proposals: Dict[str, Dict[str, Any]] = {}

    def create_proposal(self, action: str, target_node: str) -> str:
        proposal_id = f"{target_node}_isolado_{uuid.uuid4().hex[:8]}"
        self.proposals[proposal_id] = {
            "action": action,
            "target_node": target_node,
            "votes": {}
        }
        return proposal_id

    def vote(self, proposal_id: str, vote: bool):
        if proposal_id in self.proposals:
            self.proposals[proposal_id]["votes"][self.node_id] = vote

    def is_approved(self, proposal_id: str) -> bool:
        if proposal_id in self.proposals:
            yes_votes = sum(1 for v in self.proposals[proposal_id]["votes"].values() if v)
            return yes_votes >= self.t
        return False
