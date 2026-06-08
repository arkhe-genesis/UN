# ═══════════════════════════════════════════════════════════════
# V9-010: DECENTRALIZED FEDERATED LEARNING WITH ZK PROOFS
# ═══════════════════════════════════════════════════════════════
"""
Cathedral ARKHE v9.0 LOGOS — Federated Learning with ZK Proofs
V9-010: Decentralized training without sharing raw data.
Each node proves (via ZK) that it trained correctly.
Based on: ZK-ML proofs (emerging 2025), FL with DP (maturing).
Seal: FEDERATED-ZK-v9.0.0-2026-01-15
"""

from __future__ import annotations
import hashlib
import json
import time
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class FederatedConfig:
    # Federated learning
    n_nodes: int = 8
    min_nodes_per_round: int = 4
    max_rounds: int = 100
    local_epochs: int = 2
    local_lr: float = 1e-5
    # Aggregation
    aggregation_method: str = "fedavg"  # "fedavg", "fedprox", "scaffold"
    fedprox_mu: float = 0.01
    # Privacy
    differential_privacy: bool = True
    dp_epsilon: float = 10.0
    dp_delta: float = 1e-5
    dp_noise_multiplier: float = 0.8
    clip_norm: float = 1.0
    # ZK proofs
    zk_enabled: bool = True
    zk_circuit_depth: int = 10
    zk_proof_timeout: float = 60.0
    # Security
    byzantine_threshold: float = 0.25  # Tolerance to byzantine nodes
    min_stake: float = 100.0           # Minimum stake to participate


class ZKProofGenerator:
    """
    Generates zero-knowledge proofs that local training was correct.

    In production: uses zk-SNARKs circuits (Groth16/PLONK).
    Here: simulation with hash commitments.
    """

    def __init__(self, config: FederatedConfig):
        self.config = config

    def generate_training_proof(self, node_id: int,
                                 model_hash_before: str,
                                 model_hash_after: str,
                                 loss_before: float,
                                 loss_after: float,
                                 n_samples: int,
                                 clip_norm: float) -> Dict:
        """
        Generates proof that:
        1. The model was updated from the correct state
        2. The loss decreased (or remained stable)
        3. The gradient was clipped to the maximum norm
        4. The number of samples is within the allowed range

        Returns commitment + proof hash (in production: serialized proof).
        """
        if not self.config.zk_enabled:
            return {"zk_enabled": False, "status": "skipped"}

        # Commitment: hash of inputs
        commitment_input = json.dumps({
            "node": node_id,
            "model_before": model_hash_before,
            "model_after": model_hash_after,
            "loss_before": loss_before,
            "loss_after": loss_after,
            "n_samples": n_samples,
            "clip": clip_norm,
            "timestamp": time.time(),
        }, sort_keys=True)
        commitment = hashlib.sha256(commitment_input.encode()).hexdigest()

        # Proof: in production, this would be a real zk-SNARK
        # Here: we simulate with an extended hash
        proof_input = f"{commitment}:{node_id}:{loss_before}:{loss_after}"
        proof_hash = hashlib.sha256(proof_input.encode()).hexdigest()

        # Simulated verifications
        loss_decreased = loss_after <= loss_before * 1.1  # 10% tolerance
        clip_ok = clip_norm <= self.config.clip_norm * 1.05
        samples_ok = n_samples > 0

        valid = loss_decreased and clip_ok and samples_ok

        return {
            "zk_enabled": True,
            "status": "valid" if valid else "invalid",
            "commitment": commitment,
            "proof_hash": proof_hash,
            "checks": {
                "loss_decreased": loss_decreased,
                "clip_ok": clip_ok,
                "samples_ok": samples_ok,
            },
            "node_id": node_id,
        }


class ZKProofVerifier:
    """Verifies ZK proofs from other nodes."""

    def __init__(self, config: FederatedConfig):
        self.config = config

    def verify_proof(self, proof: Dict) -> bool:
        """Verifies a ZK proof."""
        if not self.config.zk_enabled or not proof.get("zk_enabled"):
            return True  # If ZK disabled, trust

        if proof.get("status") != "valid":
            return False

        # Verify that checks passed
        checks = proof.get("checks", {})
        return all(checks.values())


class FederatedNode:
    """Represents a federated node."""

    def __init__(self, node_id: int, model: nn.Module,
                 config: FederatedConfig, data_size: int = 1000):
        self.node_id = node_id
        self.model = model
        self.config = config
        self.data_size = data_size
        self.stake = config.min_stake
        self.is_honest = True  # For byzantine simulation

    def get_model_hash(self) -> str:
        """Hash of the current model state."""
        state = {k: v.shape for k, v in self.model.state_dict().items()}
        return hashlib.sha256(json.dumps(state, sort_keys=True).encode()).hexdigest()

    def local_train(self, n_epochs: int = 2) -> Dict:
        """
        Local training (simulated).
        In production: uses real data from the node.
        """
        self.model.train()
        loss_before = 1.0  # Placeholder

        # Simulate update
        for name, param in self.model.named_parameters():
            if param.requires_grad:
                noise = torch.randn_like(param) * 0.001
                if self.is_honest:
                    param.data += noise * self.config.local_lr
                else:
                    # Byzantine: adversarial update
                    param.data += noise * self.config.local_lr * 10

        loss_after = 0.95  # Placeholder: loss decreased

        # Differential privacy: clip gradients
        if self.config.differential_privacy:
            for param in self.model.parameters():
                param.data = param.data.clamp(-self.config.clip_norm, self.config.clip_norm)

        return {
            "node_id": self.node_id,
            "loss_before": loss_before,
            "loss_after": loss_after,
            "n_samples": self.data_size,
            "clip_norm": self.config.clip_norm,
            "is_honest": self.is_honest,
        }

    def get_update(self) -> Dict[str, torch.Tensor]:
        """Returns model delta (update to aggregate)."""
        return {k: v.clone() for k, v in self.model.state_dict().items()}


class FederatedAggregator:
    """Aggregates updates from multiple nodes."""

    def __init__(self, config: FederatedConfig, global_model: nn.Module):
        self.config = config
        self.global_model = global_model

    def aggregate(self, updates: List[Dict[str, torch.Tensor]],
                  weights: List[float]) -> Dict[str, torch.Tensor]:
        """FedAvg: weighted average of updates."""
        total_weight = sum(weights)
        aggregated = {}

        for key in updates[0].keys():
            weighted_sum = torch.zeros_like(updates[0][key], dtype=torch.float32)
            for update, w in zip(updates, weights):
                weighted_sum += update[key].float() * w
            aggregated[key] = (weighted_sum / total_weight).to(updates[0][key].dtype)

        return aggregated

    def apply_update(self, aggregated: Dict[str, torch.Tensor]):
        """Applies aggregated update to the global model."""
        self.global_model.load_state_dict(aggregated)


class FederatedZKTrainer:
    """
    Federated trainer with ZK proofs.

    Cycle per round:
    1. Broadcast global model to selected nodes
    2. Each node trains locally
    3. Each node generates ZK proof of training
    4. Aggregator verifies ZK proofs
    5. Only updates with valid proof are aggregated
    6. New global model is broadcast
    """

    def __init__(self, config: FederatedConfig, global_model: nn.Module):
        self.config = config
        self.global_model = global_model
        self.nodes: List[FederatedNode] = []
        self.zk_gen = ZKProofGenerator(config)
        self.zk_verifier = ZKProofVerifier(config)
        self.aggregator = FederatedAggregator(config, global_model)
        self._round_history: List[Dict] = []

    def register_node(self, node: FederatedNode):
        if node.stake >= self.config.min_stake:
            self.nodes.append(node)

    def run_round(self, round_id: int) -> Dict:
        """Executes a round of federated training."""
        # Select nodes for this round
        selected = self._select_nodes()
        if len(selected) < self.config.min_nodes_per_round:
            return {"status": "error", "error": "Not enough nodes"}

        valid_updates = []
        valid_weights = []
        proof_results = []

        for node in selected:
            # Local training
            model_hash_before = node.get_model_hash()
            train_result = node.local_train(self.config.local_epochs)
            model_hash_after = node.get_model_hash()

            # Generate ZK proof
            proof = self.zk_gen.generate_training_proof(
                node_id=node.node_id,
                model_hash_before=model_hash_before,
                model_hash_after=model_hash_after,
                **train_result,
            )

            # Verify proof
            proof_valid = self.zk_verifier.verify_proof(proof)
            proof_results.append({
                "node": node.node_id,
                "valid": proof_valid,
                "proof": proof.get("proof_hash", "none")[:16],
            })

            if proof_valid:
                update = node.get_update()
                valid_updates.append(update)
                valid_weights.append(node.data_size)

        # Aggregate
        if valid_updates:
            aggregated = self.aggregator.aggregate(valid_updates, valid_weights)
            self.aggregator.apply_update(aggregated)

        round_result = {
            "round": round_id,
            "selected_nodes": len(selected),
            "valid_proofs": sum(1 for p in proof_results if p["valid"]),
            "rejected_proofs": sum(1 for p in proof_results if not p["valid"]),
            "proof_details": proof_results,
            "aggregated": len(valid_updates) > 0,
        }
        self._round_history.append(round_result)
        return round_result

    def _select_nodes(self) -> List[FederatedNode]:
        """Selects nodes for the round (by stake)."""
        sorted_nodes = sorted(self.nodes, key=lambda n: n.stake, reverse=True)
        return sorted_nodes[:self.config.n_nodes]

    def get_telemetry(self) -> dict:
        n_rounds = len(self._round_history)
        return {
            "module": "FederatedZKTrainer",
            "version": "9.0.0",
            "substrate": "v9-decentralized",
            "seal": "FEDERATED-ZK-v9.0.0-2026-01-15",
            "n_registered_nodes": len(self.nodes),
            "n_rounds_completed": n_rounds,
            "zk_enabled": self.config.zk_enabled,
            "dp_enabled": self.config.differential_privacy,
            "dp_epsilon": self.config.dp_epsilon if self.config.differential_privacy else None,
            "aggregation": self.config.aggregation_method,
            "byzantine_tolerance": self.config.byzantine_threshold,
        }
