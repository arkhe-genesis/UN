# ═══════════════════════════════════════════════════════════════
# V9-003: Q-SPARSE ATTENTION
# ═══════════════════════════════════════════════════════════════
"""
Cathedral ARKHE v9.0 LOGOS — Q-Sparse Attention
V9-003: Adaptive subset of queries attends to the full KV.
The remaining ones use local attention or are skipped. O(N√N) → O(N) for the majority.
Based on: "Q-Sparse: Query-Aware Sparse Attention" (2025).
Seal: QSPARSE-ATTN-v9.0.0-2026-01-15
"""

from __future__ import annotations
import math
from dataclasses import dataclass
from typing import Dict, Optional, Tuple

import torch
import torch.nn as nn
import torch.nn.functional as F


@dataclass
class QSparseConfig:
    d_model: int = 4096
    n_heads: int = 32
    n_kv_heads: int = 8
    head_dim: int = 128
    # Specific Q-Sparse
    sparse_ratio: float = 0.5        # Fraction of queries using global attention
    local_window: int = 256          # Window for non-selected queries
    query_importance_threshold: float = 0.5
    # MLA (kept from v8)
    d_latent: int = 512
    # RoPE
    rope_base: float = 10000.0
    max_seq_len: int = 131072
    # Softcap (FA3, kept from v8)
    softcap: float = 50.0
    # Differential (kept from v8)
    use_differential: bool = True


class QueryImportanceScorer(nn.Module):
    """
    Scores if a query needs global attention or local is enough.
    Uses the query's own hidden state as signal.
    """

    def __init__(self, d_model: int, n_heads: int):
        super().__init__()
        self.scorer = nn.Sequential(
            nn.Linear(d_model, d_model // 4, bias=False),
            nn.GELU(),
            nn.Linear(d_model // 4, n_heads, bias=False),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Args:
            x: (B, L, D)
        Returns:
            scores: (B, L, n_heads) — high = needs global attention
        """
        return torch.sigmoid(self.scorer(x))


class QSparseAttention(nn.Module):
    """
    Q-Sparse Attention: queries adaptively choose global vs local.

    For each head, each query position decides:
    - GLOBAL: full attention to the KV cache (like standard attention)
    - LOCAL: attention only to the local window (O(window) instead of O(seq_len))

    Result: average complexity O(N × (sparse_ratio × N + (1-sparse_ratio) × W))
    With sparse_ratio=0.5 and W=256: ~50% reduction in long sequences.
    """

    def __init__(self, config: QSparseConfig):
        super().__init__()
        self.config = config
        self.n_heads = config.n_heads
        self.n_kv_heads = config.n_kv_heads
        self.head_dim = config.head_dim
        self.n_rep = config.n_heads // config.n_kv_heads
        self.scale = config.head_dim ** -0.5

        # Projections
        self.q_proj = nn.Linear(config.d_model, config.n_heads * config.head_dim, bias=False)

        # MLA KV compression
        self.kv_down = nn.Linear(config.d_model, config.d_latent, bias=False)
        self.k_up = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim, bias=False)
        self.v_up = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim, bias=False)

        # Differential branches (if enabled)
        if config.use_differential:
            self.q_proj_neg = nn.Linear(config.d_model, config.n_heads * config.head_dim // 2, bias=False)
            self.k_up_neg = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim // 2, bias=False)
            self.v_up_neg = nn.Linear(config.d_latent, config.n_kv_heads * config.head_dim // 2, bias=False)
            self.lambda_gate = nn.Linear(config.head_dim, 1, bias=False)

        self.out_proj = nn.Linear(config.n_heads * config.head_dim, config.d_model, bias=False)

        # Query importance scorer
        self.importance_scorer = QueryImportanceScorer(config.d_model, config.n_heads)

        self.norm = nn.RMSNorm(config.d_model, eps=1e-5)

        # RoPE
        self._register_rope(config)

    def _register_rope(self, config: QSparseConfig):
        inv_freq = 1.0 / (config.rope_base ** (
            torch.arange(0, config.head_dim, 2).float() / config.head_dim
        ))
        t = torch.arange(config.max_seq_len, dtype=torch.float32)
        freqs = torch.outer(t, inv_freq)
        emb = torch.cat([freqs, freqs], dim=-1)
        self.register_buffer("cos_cached", emb.cos()[None, None, :, :], persistent=False)
        self.register_buffer("sin_cached", emb.sin()[None, None, :, :], persistent=False)

    def _apply_rope(self, x: torch.Tensor, seq_len: int) -> torch.Tensor:
        cos = self.cos_cached[:, :, :seq_len, :x.shape[-1]]
        sin = self.sin_cached[:, :, :seq_len, :x.shape[-1]]
        x1, x2 = x[..., ::2], x[..., 1::2]
        return x * cos + torch.stack([-x2, x1], dim=-1).flatten(-2) * sin

    def _apply_softcap(self, scores: torch.Tensor) -> torch.Tensor:
        if self.config.softcap > 0:
            return self.config.softcap * torch.tanh(scores / self.config.softcap)
        return scores

    def forward(self, x: torch.Tensor,
                kv_cache: Optional[Tuple] = None) -> Tuple[torch.Tensor, Optional[Tuple], Dict]:
        B, L, D = x.shape
        x_norm = self.norm(x)

        # Query importance: which positions need global attention?
        importance = self.importance_scorer(x_norm)  # (B, L, n_heads)
        global_mask = importance > self.config.query_importance_threshold  # (B, L, H)

        # Project Q, K, V
        q = self._apply_rope(
            self.q_proj(x_norm).view(B, L, self.n_heads, self.head_dim), L
        )
        kv_lat = self.kv_down(x_norm)
        k = self._apply_rope(
            self.k_up(kv_lat).view(B, L, self.n_kv_heads, self.head_dim), L
        )
        v = self.v_up(kv_lat).view(B, L, self.n_kv_heads, self.head_dim)

        # Cache
        if kv_cache is not None:
            k_c, v_c = kv_cache
            k = torch.cat([k_c, k], dim=1)
            v = torch.cat([v_c, v], dim=1)
        new_cache = (k, v)

        # GQA expand
        k = k.repeat_interleave(self.n_rep, dim=2)
        v = v.repeat_interleave(self.n_rep, dim=2)

        # Transpose: (B, H, L, D)
        q = q.transpose(1, 2)
        k = k.transpose(1, 2)
        v = v.transpose(1, 2)

        total_kv_len = k.shape[2]

        # ── Q-Sparse: compute attention separately for global and local ──
        # For simplicity: use combined mask by head
        # In production: implement with specialized kernels
        global_mask_h = global_mask.permute(0, 2, 1)  # (B, H, L)

        # Full attention scores
        scores = torch.matmul(q, k.transpose(-2, -1)) * self.scale  # (B, H, L, total)
        scores = self._apply_softcap(scores)

        # Causal mask
        causal = torch.triu(
            torch.ones(L, total_kv_len, device=x.device, dtype=torch.bool),
            diagonal=total_kv_len - L + 1
        )
        scores = scores.masked_fill(causal[None, None, :, :], float('-inf'))

        # Local mask: for non-global queries, mask outside the window
        if self.config.local_window < total_kv_len:
            # Create local window mask
            positions_q = torch.arange(L, device=x.device).unsqueeze(1)
            positions_kv = torch.arange(total_kv_len, device=x.device).unsqueeze(0)
            local_valid = (positions_kv - positions_q) >= 0  # causal
            local_valid = local_valid & ((positions_kv - positions_q) < self.config.local_window)
            local_mask = ~local_valid  # True = should mask

            # Apply only to non-global queries
            local_expand = local_mask[None, None, :, :]  # (1, 1, L, total)
            non_global = ~global_mask_h.unsqueeze(-1)  # (B, H, L, 1)
            scores = scores.masked_fill(local_expand & non_global, float('-inf'))

        attn = F.softmax(scores, dim=-1)
        attn = attn.nan_to_num(0.0)  # Clear NaNs from fully masked rows

        out = torch.matmul(attn, v)  # (B, H, L, D)
        out = out.transpose(1, 2).contiguous().view(B, L, -1)

        # Differential branch (if enabled)
        if self.config.use_differential and hasattr(self, 'q_proj_neg'):
            # Simplified: compute negative branch and subtract
            q_neg = self._apply_rope(
                self.q_proj_neg(x_norm).view(B, L, self.n_heads // 2, self.head_dim), L
            )
            k_neg = self._apply_rope(
                self.k_up_neg(kv_lat).view(B, L, self.n_kv_heads // 2, self.head_dim), L
            )
            v_neg = self.v_up_neg(kv_lat).view(B, L, self.n_kv_heads // 2, self.head_dim)
            q_neg = q_neg.transpose(1, 2)
            k_neg = k_neg.repeat_interleave(self.n_rep, dim=2).transpose(1, 2)
            v_neg = v_neg.repeat_interleave(self.n_rep, dim=2).transpose(1, 2)
            s_neg = self._apply_softcap(
                torch.matmul(q_neg, k_neg.transpose(-2, -1)) * self.scale
            )
            s_neg = s_neg.masked_fill(causal[None, None, :, :L], float('-inf'))
            a_neg = F.softmax(s_neg, dim=-1).nan_to_num(0.0)
            out_neg = torch.matmul(a_neg, v_neg)
            out_neg = out_neg.transpose(1, 2).contiguous().view(B, L, -1)

            # Lambda gate
            lam = torch.sigmoid(self.lambda_gate(out))  # (B, L, 1)
            out_diff = lam * out + (1 - lam) * (out - out_neg[:, :, :out.shape[-1]])
            out = out_diff

        output = self.out_proj(out) + x

        global_fraction = global_mask.float().mean().item()
        info = {
            "global_query_fraction": global_fraction,
            "local_window": self.config.local_window,
            "estimated_complexity": global_fraction + (1 - global_fraction) * self.config.local_window / max(total_kv_len, 1),
        }

        return output, new_cache, info

    def get_telemetry(self) -> dict:
        return {
            "module": "QSparseAttention",
            "version": "9.0.0",
            "substrate": "v9-backbone",
            "seal": "QSPARSE-ATTN-v9.0.0-2026-01-15",
            "n_heads": self.n_heads,
            "n_kv_heads": self.n_kv_heads,
            "sparse_ratio": self.config.sparse_ratio,
            "local_window": self.config.local_window,
            "differential": self.config.use_differential,
            "softcap": self.config.softcap,
        }
