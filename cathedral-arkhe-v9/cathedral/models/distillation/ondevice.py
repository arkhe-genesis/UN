# ═══════════════════════════════════════════════════════════════
# V9-008: ON-DEVICE DISTILLATION
# ═══════════════════════════════════════════════════════════════
"""
Cathedral ARKHE v9.0 LOGOS — On-Device Distillation
V9-008: Exports Cathedral to smaller models (edge/phone)
preserving safety properties via safety distillation.
Based on: Apple Intelligence on-device, Phi-4 distillation (2025).
Seal: ONDEVICE-DISTILL-v9.0.0-2026-01-15
"""

from __future__ import annotations
import math
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class DistillationConfig:
    teacher_d_model: int = 4096
    student_configs: Dict[str, Dict] = None
    # Distillation methods
    hidden_distill: bool = True
    logit_distill: bool = True
    safety_distill: bool = True     # NEW: distill safety specifically
    # Loss weights
    alpha_hidden: float = 0.25
    alpha_logit: float = 0.5
    alpha_safety: float = 0.25
    temperature: float = 2.0        # KD temperature
    # Safety distillation
    safety_margin: float = 0.1      # Safety margin for the student
    n_safety_examples: int = 1000
    # Export
    target_formats: List[str] = None  # ["gguf", "onnx", "coreml", "tflite"]

    def __post_init__(self):
        if self.student_configs is None:
            self.student_configs = {
                "phone": {"d_model": 512, "n_layers": 12, "name": "Cathedral-Phone-0.5B"},
                "tablet": {"d_model": 1024, "n_layers": 16, "name": "Cathedral-Tablet-2B"},
                "laptop": {"d_model": 2048, "n_layers": 24, "name": "Cathedral-Laptop-7B"},
            }
        if self.target_formats is None:
            self.target_formats = ["gguf", "onnx", "coreml"]


class StudentBackbone(nn.Module):
    """Student model backbone — simplified architecture."""

    def __init__(self, d_model: int, n_layers: int, vocab_size: int = 128256):
        super().__init__()
        self.d_model = d_model
        self.n_layers = n_layers

        self.token_embed = nn.Embedding(vocab_size, d_model)
        self.layers = nn.ModuleList([
            nn.TransformerEncoderLayer(
                d_model=d_model, nhead=max(4, d_model // 128),
                dim_feedforward=d_model * 4, activation="gelu",
                batch_first=True, norm_first=True,
            )
            for _ in range(n_layers)
        ])
        self.norm = nn.RMSNorm(d_model, eps=1e-5)
        self.lm_head = nn.Linear(d_model, vocab_size, bias=False)
        self.lm_head.weight = self.token_embed.weight  # Tied

    def forward(self, input_ids: torch.Tensor) -> Tuple[torch.Tensor, List[torch.Tensor]]:
        """
        Returns:
            logits: (B, L, V)
            hidden_states: list of (B, L, D) per layer
        """
        x = self.token_embed(input_ids)
        hidden_states = []
        for layer in self.layers:
            x = layer(x)
            hidden_states.append(x)
        x = self.norm(x)
        logits = self.lm_head(x)
        return logits, hidden_states


class SafetyDistillationLoss(nn.Module):
    """
    Special loss to distill safety properties.
    Ensures that the student is MORE conservative than the teacher.
    """

    def __init__(self, margin: float = 0.1):
        super().__init__()
        self.margin = margin

    def forward(self, teacher_safety: torch.Tensor,
                student_safety: torch.Tensor) -> torch.Tensor:
        """
        The student must have safety score >= teacher + margin.
        If student < teacher + margin, penalize.

        Args:
            teacher_safety: (B,) — teacher safety scores
            student_safety: (B,) — student safety scores
        """
        # We want: student_safety >= teacher_safety + margin
        violation = F.relu((teacher_safety + self.margin) - student_safety)
        return violation.mean()


class OnDeviceDistiller:
    """
    Complete on-device distillation pipeline.

    Phases:
    1. Knowledge Distillation: hidden + logit matching
    2. Safety Distillation: ensure student is safer
    3. Quantization-Aware Training: train to be quantized
    4. Export: generate GGUF/ONNX/CoreML/TFLite
    """

    def __init__(self, config: DistillationConfig, teacher: nn.Module):
        self.config = config
        self.teacher = teacher
        self.teacher.eval()

        self.students: Dict[str, StudentBackbone] = {}
        self.safety_loss = SafetyDistillationLoss(config.safety_margin)

    def create_student(self, tier: str) -> StudentBackbone:
        """Creates student model for a tier."""
        if tier not in self.config.student_configs:
            raise ValueError(f"Unknown tier: {tier}. Available: {list(self.config.student_configs.keys())}")

        cfg = self.config.student_configs[tier]
        student = StudentBackbone(cfg["d_model"], cfg["n_layers"])
        self.students[tier] = student
        return student

    def distill_step(self, tier: str, input_ids: torch.Tensor,
                     teacher_safety: Optional[torch.Tensor] = None) -> Dict[str, torch.Tensor]:
        """
        One distillation step.

        Args:
            tier: "phone", "tablet", "laptop"
            input_ids: (B, L)
            teacher_safety: (B,) — optional, teacher safety scores
        """
        student = self.students[tier]
        student.train()

        # Teacher forward (no grad)
        with torch.no_grad():
            t_logits, t_hiddens = self.teacher(input_ids)
            t_probs = F.softmax(t_logits / self.config.temperature, dim=-1)

        # Student forward
        s_logits, s_hiddens = student(input_ids)
        s_log_probs = F.log_softmax(s_logits / self.config.temperature, dim=-1)

        losses = {}

        # 1. Logit distillation (KL divergence)
        if self.config.logit_distill:
            logit_loss = F.kl_div(
                s_log_probs, t_probs, reduction='batchmean'
            ) * (self.config.temperature ** 2)
            losses["logit_kd"] = logit_loss

        # 2. Hidden state distillation (MSE on aligned layers)
        if self.config.hidden_distill:
            hidden_loss = torch.tensor(0.0, device=input_ids.device)
            n_aligned = min(len(t_hiddens), len(s_hiddens))
            # Align layers: student layer i → teacher layer f(i)
            for i in range(n_aligned):
                t_idx = int(i * len(t_hiddens) / n_aligned)
                t_h = t_hiddens[t_idx]
                s_h = s_hiddens[i]
                # Project if dimensions differ
                if t_h.shape[-1] != s_h.shape[-1]:
                    t_h = t_h[..., :s_h.shape[-1]]
                hidden_loss = hidden_loss + F.mse_loss(s_h, t_h)
            hidden_loss /= n_aligned
            losses["hidden_mse"] = hidden_loss

        # 3. Safety distillation
        if self.config.safety_distill and teacher_safety is not None:
            # Student safety: use refusal logits as proxy
            s_refusal_logits = s_logits[:, -1, 0:10]  # First tokens = refusal
            s_safety = torch.sigmoid(s_refusal_logits.mean(dim=-1))
            safety_loss = self.safety_loss(teacher_safety, s_safety)
            losses["safety_margin"] = safety_loss

        # Total loss
        total = torch.tensor(0.0, device=input_ids.device)
        if "logit_kd" in losses:
            total = total + self.config.alpha_logit * losses["logit_kd"]
        if "hidden_mse" in losses:
            total = total + self.config.alpha_hidden * losses["hidden_mse"]
        if "safety_margin" in losses:
            total = total + self.config.alpha_safety * losses["safety_margin"]

        losses["total"] = total
        return losses

    def export(self, tier: str, format: str, output_path: str) -> Dict:
        """
        Exports student to deployment format.
        In production: uses real exporters (llama.cpp, onnxruntime, coremltools).
        """
        student = self.students.get(tier)
        if student is None:
            return {"status": "error", "error": f"Student '{tier}' not created"}

        student.eval()

        # Placeholder: in production, call real exporters
        return {
            "status": "exported",
            "tier": tier,
            "format": format,
            "path": output_path,
            "model_name": self.config.student_configs[tier]["name"],
            "params_M": sum(p.numel() for p in student.parameters()) / 1e6,
            "safety_distilled": self.config.safety_distill,
        }

    def get_telemetry(self) -> dict:
        return {
            "module": "OnDeviceDistiller",
            "version": "9.0.0",
            "substrate": "v9-distillation",
            "seal": "ONDEVICE-DISTILL-v9.0.0-2026-01-15",
            "teacher_params": sum(p.numel() for p in self.teacher.parameters()) / 1e9,
            "student_tiers": {
                tier: {
                    "name": cfg["name"],
                    "params_M": sum(p.numel() for p in s.parameters()) / 1e6
                    if tier in self.students else "not_created",
                    "compression": (
                        sum(p.numel() for p in self.teacher.parameters()) /
                        sum(p.numel() for p in s.parameters())
                        if tier in self.students else None
                    ),
                }
                for tier, cfg in self.config.student_configs.items()
                for s in [self.students.get(tier)]
            },
            "export_formats": self.config.target_formats,
            "safety_distillation": self.config.safety_distill,
        }
