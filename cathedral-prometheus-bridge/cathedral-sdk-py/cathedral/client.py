import uuid
import grpc
from typing import Dict, Any, List, Optional
from generated.cathedral.v1 import bridge_pb2
from generated.cathedral.v1 import bridge_pb2_grpc
from google.protobuf.timestamp_pb2 import Timestamp
import time
import json

class CathedralGrpcClient:
    def __init__(self, endpoint: str = "localhost:9002", project_id: str = "default", agent_id: str = "agent-1"):
        self.channel = grpc.aio.insecure_channel(endpoint)
        self.stub = bridge_pb2_grpc.CathedralBridgeStub(self.channel)
        self.project_id = project_id
        self.agent_id = agent_id

    async def emit_design_proposed(
        self,
        design_hash: str,
        parent_hashes: List[str],
        parameters: Dict[str, float],
        rationale: str
    ):
        event = bridge_pb2.Event(
            event_id=str(uuid.uuid4()),
            timestamp=Timestamp(seconds=int(time.time()), nanos=0),
            event_type=bridge_pb2.DESIGN_PROPOSED,
            design_hash=design_hash,
            parent_hashes=parent_hashes,
            payload_json=json.dumps({"parameters": parameters, "rationale": rationale}),
            metadata=bridge_pb2.EventMetadata(
                domain="aerospace",
                confidence=0.8,
                compute_cost_usd=0.0,
                tags=["design_proposed"]
            )
        )

        request = bridge_pb2.IngestRequest(
            project_id=self.project_id,
            agent_id=self.agent_id,
            events=[event]
        )
        response = await self.stub.Ingest(request)
        return response

    async def emit_simulation_completed(
        self,
        design_hash: str,
        simulator: str,
        metrics: Dict[str, float],
        convergence: bool,
        compute_cost_usd: float
    ):
        event = bridge_pb2.Event(
            event_id=str(uuid.uuid4()),
            timestamp=Timestamp(seconds=int(time.time()), nanos=0),
            event_type=bridge_pb2.SIMULATION_COMPLETED,
            design_hash=design_hash,
            parent_hashes=[],
            payload_json=json.dumps({
                "simulator": simulator,
                "metrics": metrics,
                "convergence": convergence,
            }),
            metadata=bridge_pb2.EventMetadata(
                domain="aerospace",
                confidence=0.9,
                compute_cost_usd=compute_cost_usd,
                tags=["simulation_completed"]
            )
        )

        request = bridge_pb2.IngestRequest(
            project_id=self.project_id,
            agent_id=self.agent_id,
            events=[event]
        )
        response = await self.stub.Ingest(request)
        return response

    async def emit_agent_mutation(
        self,
        mutation_description: str,
        previous_agent_hash: str
    ):
        event = bridge_pb2.Event(
            event_id=str(uuid.uuid4()),
            timestamp=Timestamp(seconds=int(time.time()), nanos=0),
            event_type=bridge_pb2.AGENT_MUTATION,
            design_hash=str(uuid.uuid4()),
            parent_hashes=[previous_agent_hash],
            payload_json=json.dumps({"mutation_description": mutation_description}),
            metadata=bridge_pb2.EventMetadata(
                domain="meta",
                confidence=1.0,
                compute_cost_usd=0.0,
                tags=["agent_mutation"]
            )
        )

        request = bridge_pb2.IngestRequest(
            project_id=self.project_id,
            agent_id=self.agent_id,
            events=[event]
        )
        response = await self.stub.Ingest(request)
        return response

    async def request_governance(
        self,
        event_type: "bridge_pb2.EventType",
        proposed_state: Dict[str, Any]
    ):
        request = bridge_pb2.GovernanceRequest(
            request_id=str(uuid.uuid4()),
            project_id=self.project_id,
            agent_id=self.agent_id,
            event_type=event_type,
            proposed_state_json=json.dumps(proposed_state),
            current_state_json="{}",
            agent_risk_score=0.2,
            domain="general",
            metadata={}
        )
        response = await self.stub.RequestGovernance(request)
        return response

    async def close(self):
        await self.channel.close()
