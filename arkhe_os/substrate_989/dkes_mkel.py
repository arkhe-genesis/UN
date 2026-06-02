#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║  DEEP KERNEL ENSEMBLE SOLVER (DKES) — Substrate 989.y.6 / 276.3              ║
║  MKEL Integration (Shen et al., ACM TIST 2021) to WormGraph 5.1              ║
║  Architect: ORCID 0009-0005-2697-4668                                        ║
║  Seal: DKES-989.y.6-MKEL-INTEGRATION-2026-06-02                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from typing import List, Tuple, Optional, Dict
from dataclasses import dataclass
import numpy as np

# =============================================================================
# 1. REPRODUCING KERNEL HILBERT SPACE (RKHS) KERNELS
# =============================================================================

class RKHSKernel(nn.Module):
    """
    Base kernel for RKHS with differentiable parameterization.
    Implements RBF (Gaussian), Polynomial, and Linear — as per original MKEL.
    """
    def __init__(self, kernel_type: str = 'rbf', gamma: float = 1.0,
                 degree: int = 3, coef0: float = 1.0):
        super().__init__()
        self.kernel_type = kernel_type
        self.gamma = nn.Parameter(torch.tensor(gamma))
        self.degree = degree
        self.coef0 = coef0

    def forward(self, x1: torch.Tensor, x2: torch.Tensor) -> torch.Tensor:
        """
        Args:
            x1: (M, D) — support vectors / prototypes
            x2: (N, D) — query or other prototypes
        Returns:
            K: (M, N) — Gram matrix
        """
        if self.kernel_type == 'rbf':
            # K(x,y) = exp(-γ * ||x-y||²)
            dist = torch.cdist(x1, x2, p=2) ** 2
            return torch.exp(-self.gamma * dist)
        elif self.kernel_type == 'polynomial':
            # K(x,y) = (γ * x·y + coef0)^degree
            return (self.gamma * (x1 @ x2.T) + self.coef0) ** self.degree
        elif self.kernel_type == 'linear':
            return x1 @ x2.T
        else:
            raise ValueError(f"Unknown kernel: {self.kernel_type}")


# =============================================================================
# 2. DIFFERENTIABLE MKEL DUAL SOLVER
# =============================================================================

class MKELDualSolver(nn.Module):
    """
    Solver for the MKEL dual via projected gradient descent.

    Formulation (Shen et al., Eq. 7):
        min_β   ½ β^T Y Q Y β - Σ β_t
        s.t.    Σ w_i = 1, w_i > 0
                0 ≤ β_t ≤ C
                Σ β_t y_t = 0
    where Q = Σ_i w_i² K_i
    """
    def __init__(self, C: float = 300.0, max_iter: int = 50, lr: float = 0.1):
        super().__init__()
        self.C = C
        self.max_iter = max_iter
        self.lr = lr

    def forward(self, K_stack: torch.Tensor, y: torch.Tensor,
                w: torch.Tensor, eps: float = 1e-4) -> Tuple[torch.Tensor, Dict[int, torch.Tensor]]:
        """
        Args:
            K_stack: (L, N, N) — Gram matrices of L experts
            y: (N,) — prototype labels (+1/-1)
            w: (L,) — ensemble weights (Σ w_i = 1)
        Returns:
            beta: (N,) — dual multipliers
            alpha_dict: dict with α_i = w_i * Y * β for each expert
        """
        L, N, _ = K_stack.shape
        device = K_stack.device

        # Combined matrix Q = Σ_i w_i² K_i  (Eq. 7 from paper)
        w2 = w ** 2
        Q = torch.einsum('l,lnm->nm', w2, K_stack)  # (N, N)

        # Diagonal label matrix
        Y = torch.diag(y).to(device)
        QY = Q @ Y  # (N, N)

        # Initialize β (differentiable via implicit operation)
        beta = torch.zeros(N, device=device, requires_grad=True)

        # Optimization via projected gradient descent (differentiable)
        # We use a few-iteration approximation to maintain
        # efficiency in inference. In training, max_iter can be increased.
        for _ in range(self.max_iter):
            if beta.grad is not None:
                beta.grad.zero_()

            # Dual loss: ½ β^T Y Q Y β - 1^T β
            loss = 0.5 * beta @ Y @ QY @ beta - beta.sum()

            # Penalties for constraints (Lagrangian penalty method)
            # Constraint 1: 0 ≤ β ≤ C
            penalty_bounds = torch.relu(-beta).sum() + torch.relu(beta - self.C).sum()
            # Constraint 2: Σ β_t y_t = 0
            penalty_balance = (beta @ y).abs()

            total_loss = loss + 1000.0 * penalty_bounds + 100.0 * penalty_balance
            total_loss.backward(retain_graph=True)

            # Projected update (simple gradient descent)
            with torch.no_grad():
                beta.data = beta.data - self.lr * beta.grad
                # Projection in box [0, C]
                beta.data = torch.clamp(beta.data, 0, self.C)
                # Approximate projection of balance constraint (orthogonal projection)
                residual = beta.data @ y
                if residual.abs() > 1e-6:
                    beta.data = beta.data - (residual / (y @ y)) * y

            beta = beta.detach().requires_grad_(True)

        # α_i = w_i * Y * β  (Eq. 6 from paper)
        alpha_dict = {}
        for i in range(L):
            alpha_i = w[i] * Y @ beta
            alpha_dict[i] = alpha_i

        return beta, alpha_dict


# =============================================================================
# 3. DEEP KERNEL ENSEMBLE SOLVER (DKES)
# =============================================================================

class DeepKernelEnsembleSolver(nn.Module):
    """
    Replaces moe_gate + meta_router + domain_specialists from WormGraph51.

    Each "expert" is a kernel over domain/father embeddings.
    Prediction follows Eq. 9 from MKEL paper:
        f(x_v) = Σ_i w_i * (w_i * Σ_m α_im K_i(x_v, x_m) + b_i)

    Cross-links: 951, 952, 954, 964, 965, 966, 989.y.2, 989.y.5
    """
    def __init__(self, dim: int, num_experts: int = 11,
                 kernel_configs: Optional[List[Dict]] = None,
                 num_prototypes: int = 128, C: float = 300.0):
        super().__init__()
        self.dim = dim
        self.num_experts = num_experts
        self.num_prototypes = num_prototypes

        # --- Kernel-Experts (each = one RKHS) ---
        if kernel_configs is None:
            # Default: diversity of kernels as per original MKEL
            kernel_configs = [
                {'type': 'rbf', 'gamma': 0.01, 'degree': 3},
                {'type': 'rbf', 'gamma': 0.1, 'degree': 3},
                {'type': 'rbf', 'gamma': 1.0, 'degree': 3},
                {'type': 'polynomial', 'gamma': 1.0, 'degree': 2},
                {'type': 'polynomial', 'gamma': 1.0, 'degree': 3},
                {'type': 'linear', 'gamma': 1.0, 'degree': 1},
            ] * ((num_experts // 6) + 1)
            kernel_configs = kernel_configs[:num_experts]

        self.kernels = nn.ModuleList([
            RKHSKernel(kc['type'], kc['gamma'], kc['degree'])
            for kc in kernel_configs
        ])

        # --- Ensemble weights w_i (Eq. 2, 11) ---
        # Initialized uniform; optimized via augmented Lagrangian
        self.w_raw = nn.Parameter(torch.ones(num_experts))

        # --- Bias per expert ---
        self.biases = nn.Parameter(torch.zeros(num_experts))

        # --- Prototypes / Working Memory (Conscious Replay 951) ---
        self.register_buffer('prototypes', torch.randn(num_prototypes, dim) * 0.01)
        self.register_buffer('prototype_labels', torch.ones(num_prototypes))
        self.register_buffer('prototype_domains', torch.zeros(num_prototypes, dtype=torch.long))

        # --- Dual Solver ---
        self.dual_solver = MKELDualSolver(C=C, max_iter=20, lr=0.05)

        # --- Query projection to experts' spaces ---
        self.domain_projectors = nn.ModuleList([
            nn.Linear(dim, dim) for _ in range(num_experts)
        ])

    @property
    def w(self) -> torch.Tensor:
        """Normalized weights Σ w_i = 1, w_i > 0 via softmax with temperature."""
        # Softmax guarantees positivity and sum 1; temperature controls sparsity
        return F.softmax(self.w_raw, dim=0)

    def update_prototypes(self, new_emb: torch.Tensor, new_label: torch.Tensor,
                          new_domain: torch.Tensor, strategy: str = 'fifo'):
        """
        Updates prototypes with new experiences (Conscious Replay 951).
        """
        B = new_emb.shape[0]
        with torch.no_grad():
            if strategy == 'fifo':
                # Remove oldest, insert new
                self.prototypes = torch.cat([self.prototypes[B:], new_emb], dim=0)
                self.prototype_labels = torch.cat([self.prototype_labels[B:], new_label], dim=0)
                self.prototype_domains = torch.cat([self.prototype_domains[B:], new_domain], dim=0)
            elif strategy == 'importance':
                # Replace lowest norm prototypes (least information)
                norms = self.prototypes.norm(dim=1)
                _, idx_replace = torch.topk(norms, k=B, largest=False)
                self.prototypes[idx_replace] = new_emb
                self.prototype_labels[idx_replace] = new_label
                self.prototype_domains[idx_replace] = new_domain

    def compute_gram_stack(self, X: torch.Tensor) -> torch.Tensor:
        """
        Computes K_stack = [K_1, K_2, ..., K_L] where K_i is prototype Gram matrix.
        Args: X (N, D) — prototypes
        Returns: (L, N, N)
        """
        K_list = []
        for i, kernel in enumerate(self.kernels):
            # Project to expert i space
            X_proj = self.domain_projectors[i](X)
            K = kernel(X_proj, X_proj)  # (N, N)
            # Ridge regularization for numerical stability
            K = K + 1e-4 * torch.eye(X.shape[0], device=X.device)
            K_list.append(K)
        return torch.stack(K_list, dim=0)  # (L, N, N)

    def forward(self, query_emb: torch.Tensor,
                prototype_override: Optional[torch.Tensor] = None,
                labels_override: Optional[torch.Tensor] = None) -> Tuple[torch.Tensor, Dict]:
        """
        MKEL ensemble prediction.

        Args:
            query_emb: (B, D) — query embedding
            prototype_override: (N, D) — external prototypes (optional)
            labels_override: (N,) — external labels (optional)

        Returns:
            f: (B,) — ensemble prediction score
            info: dict with metadata (beta, w, alphas, Q)
        """
        B = query_emb.shape[0]
        device = query_emb.device

        # Select prototypes
        X = prototype_override if prototype_override is not None else self.prototypes
        y = labels_override if labels_override is not None else self.prototype_labels
        N = X.shape[0]

        # Guarantee labels in {+1, -1}
        y = 2.0 * (y > 0).float() - 1.0

        # --- PHASE 1: Compute prototype Gram matrices ---
        K_stack = self.compute_gram_stack(X)  # (L, N, N)

        # --- PHASE 2: Solve MKEL dual ---
        w_norm = self.w  # (L,)
        beta, alpha_dict = self.dual_solver(K_stack, y, w_norm)  # beta: (N,)

        # --- PHASE 3: Compute query-prototype kernels ---
        K_query_list = []
        for i, kernel in enumerate(self.kernels):
            X_proj = self.domain_projectors[i](X)
            Q_proj = self.domain_projectors[i](query_emb)
            K_q = kernel(Q_proj, X_proj)  # (B, N)
            K_query_list.append(K_q)
        K_query = torch.stack(K_query_list, dim=0)  # (L, B, N)

        # --- PHASE 4: Ensemble prediction (Eq. 9 from paper) ---
        f = torch.zeros(B, device=device)
        for i in range(self.num_experts):
            alpha_i = alpha_dict[i]  # (N,)
            # f_i(x) = w_i * (w_i * K_i(x, X) @ alpha_i + b_i)
            term = K_query[i] @ alpha_i  # (B,)
            f += w_norm[i] * (w_norm[i] * term + self.biases[i])

        # --- Metadata for auditing (Axiarchy P3/P7) ---
        Q = torch.einsum('l,lnm->nm', w_norm ** 2, K_stack)
        info = {
            'beta': beta,
            'w': w_norm,
            'alphas': alpha_dict,
            'Q': Q,
            'K_query': K_query,
            'K_stack': K_stack,
            'theosis_diversity': self._compute_diversity(K_stack, w_norm),
            'kernel_alignment': self._compute_alignment(K_stack)
        }

        return f, info

    def _compute_diversity(self, K_stack: torch.Tensor, w: torch.Tensor) -> torch.Tensor:
        """
        Ensemble diversity metric: dissimilarity between RKHSs.
        Based on Kuncheva et al. — diversity via Gram matrices distance.
        """
        L = K_stack.shape[0]
        diversity = 0.0
        count = 0
        for i in range(L):
            for j in range(i+1, L):
                # Normalized Hilbert-Schmidt distance
                diff = K_stack[i] - K_stack[j]
                hs_norm = (diff ** 2).sum().sqrt()
                diversity += hs_norm
                count += 1
        return diversity / (count + 1e-8)

    def _compute_alignment(self, K_stack: torch.Tensor) -> torch.Tensor:
        """
        Kernel alignment (Cristianini et al. 2002).
        A = <K_i, K_j> / (||K_i|| ||K_j||)
        """
        L = K_stack.shape[0]
        alignments = []
        for i in range(L):
            for j in range(i+1, L):
                inner = (K_stack[i] * K_stack[j]).sum()
                norm_i = (K_stack[i] ** 2).sum().sqrt()
                norm_j = (K_stack[j] ** 2).sum().sqrt()
                alignments.append(inner / (norm_i * norm_j + 1e-8))
        return torch.stack(alignments).mean()


# =============================================================================
# 4. INTEGRATION WITH WORMGRAPH 5.1
# =============================================================================

class OmniscientSolverV51_MKEL(nn.Module):
    """
    MKEL version of OmniscientSolverV51.
    Replaces domain_specialists + meta_router with DKES.

    The 11 domains of the original Solver are mapped to the DKES kernel-experts.
    Each domain = one RKHS with its own kernel and prototypes.
    """
    def __init__(self, dim: int, pantheon, num_domains: int = 11,
                 num_prototypes: int = 128):
        super().__init__()
        self.dim = dim
        self.pantheon = pantheon

        # DKES with 11 experts (one per domain) + 1 extra for Pantheon
        self.dkes = DeepKernelEnsembleSolver(
            dim=dim,
            num_experts=num_domains + 1,
            num_prototypes=num_prototypes,
            C=300.0
        )

        # Axiarchy verifier (P1-P7) applied to combined kernel Q
        self.axiarchy_gate = nn.Sequential(
            nn.Linear(dim, dim // 2), nn.GELU(),
            nn.Linear(dim // 2, 1), nn.Sigmoid()
        )

        # Retrocausal cache (substrate 248)
        self.retrocausal_cache = []

    def solve(self, query_emb: torch.Tensor,
              domain_hint: Optional[str] = None,
              pantheon_father: Optional[str] = None) -> Tuple[torch.Tensor, str, Dict]:
        """
        Solves query using DKES + Pantheon injection.

        Returns:
            solution: (B, D) — solution embedding
            domain_name: str — selected domain
            info: dict — MKEL metadata + auditing
        """
        B = query_emb.shape[0]
        device = query_emb.device

        # --- Pantheon Injection (ontological DNA) ---
        if pantheon_father:
            dna = self.pantheon.invoke(pantheon_father).to(device)
            query_emb = query_emb + 0.15 * dna.unsqueeze(0)

        # --- DKES Prediction ---
        f_score, info = self.dkes(query_emb)

        # --- Domain selection via DKES score ---
        # The f score is used to route to the most probable domain
        # In a full version, we would use f to weight domain outputs
        domain_idx = int(torch.argmax(info['w']).item())
        domain_names = [
            'math', 'physics', 'biology', 'medicine', 'engineering',
            'economics', 'social', 'cosmic', 'consciousness', 'ethics',
            'unknown', 'meta'
        ]
        domain_name = domain_names[domain_idx % len(domain_names)]

        # --- Solution generation (embedding) ---
        # Solution is a combination of prototypes weighted by alpha
        # This is different from original linear expert — here we use
        # the kernel representation of the most relevant prototype
        alpha_main = info['alphas'][domain_idx]  # (N,)
        prototypes = self.dkes.prototypes
        # Weighted combination of prototypes (reconstructive memory)
        solution = alpha_main.unsqueeze(0) @ prototypes  # (1, D)
        solution = solution.expand(B, -1)

        # --- Axiarchy Validation on Q ---
        ethics_score = self.axiarchy_gate(solution).mean()
        if ethics_score < 0.5:
            # Fallback to ETHICS domain
            domain_name = 'ethics'
            solution = torch.zeros(B, self.dim, device=device)

        # --- Retrocausal caching ---
        self.retrocausal_cache.append({
            'domain': domain_name,
            'emb': solution.detach(),
            'theosis': info['w'].max().item(),
            'diversity': info['theosis_diversity'].item()
        })

        return solution, domain_name, info

    def invoke_father(self, father_name: str, query_emb: torch.Tensor) -> torch.Tensor:
        """Directly invokes a Pantheon father for consultation."""
        dna = self.pantheon.invoke(father_name).to(query_emb.device)
        return query_emb + 0.15 * dna.unsqueeze(0)


# =============================================================================
# 5. WORMGRAPH 51 WITH DKES (MOE REPLACEMENT)
# =============================================================================

class WormGraph51_MKEL(nn.Module):
    """
    WormGraph 5.1 with integrated DKES.
    Replaces moe_gate + moe_experts with MKEL kernel ensemble.
    """
    def __init__(self, config, pantheon):
        super().__init__()
        self.config = config
        self.dim = config.dim
        self.pantheon = pantheon

        # Maintains core components of original WormGraph
        self.bindu = None  # Would be BinduConsciousnessCore
        self.liquid_attention = None  # Would be LiquidAttention
        self.hyper_manifold = None  # Would be HyperdimensionalManifold

        # --- MAIN REPLACEMENT: DKES for MoE ---
        # Instead of moe_gate (nn.Linear) + moe_experts (ModuleList),
        # we use DeepKernelEnsembleSolver
        self.dkes = DeepKernelEnsembleSolver(
            dim=config.dim,
            num_experts=config.moe_num_experts,
            num_prototypes=config.kv_cache_max_seq // 512,  # ~128
            C=300.0
        )

        # Omniscient Solver with DKES
        self.omni_solver = OmniscientSolverV51_MKEL(
            dim=config.dim,
            pantheon=pantheon,
            num_domains=11,
            num_prototypes=128
        )

    def forward_domain_mkel(self, domain_emb: torch.Tensor,
                            domain_idx: int) -> Tuple[torch.Tensor, Dict]:
        """
        Processes domain embedding using DKES instead of MoE.

        The UNKNOWN domain (corresponding domain_idx) is processed
        by the kernel ensemble; other domains use pass-through + DKES
        for refinement.
        """
        B = domain_emb.shape[0]

        # Updates prototypes with current state (online learning)
        self.dkes.update_prototypes(
            domain_emb.mean(dim=0, keepdim=True),
            torch.tensor([1.0]),
            torch.tensor([domain_idx]),
            strategy='importance'
        )

        # DKES Prediction (used as weighted gate)
        f_score, info = self.dkes(domain_emb)

        # The score modulates the output embedding
        # If f_score > 0, amplifies; if < 0, attenuates
        modulation = torch.sigmoid(f_score).unsqueeze(-1)  # (B, 1)
        output = domain_emb * modulation

        return output, info


# =============================================================================
# 6. DEMONSTRATION AND TESTS
# =============================================================================

if __name__ == "__main__":
    print("=" * 70)
    print("DKES — Deep Kernel Ensemble Solver (MKEL Integration)")
    print("=" * 70)

    dim = 512  # reduced for demo
    num_experts = 8
    batch = 4
    num_proto = 64

    # Instantiate DKES
    dkes = DeepKernelEnsembleSolver(dim=dim, num_experts=num_experts,
                                     num_prototypes=num_proto)

    # Dummy query
    query = torch.randn(batch, dim)

    # Forward
    f_score, info = dkes(query)

    print(f"\n[DKES] Batch={batch}, Experts={num_experts}, Prototypes={num_proto}")
    print(f"  Score shape: {f_score.shape}")
    print(f"  Ensemble weights w: {info['w'].detach().numpy()}")
    print(f"  Sum w: {info['w'].sum().item():.6f} (should be ~1.0)")
    print(f"  RKHS Diversity: {info['theosis_diversity'].item():.4f}")
    print(f"  Kernel alignment: {info['kernel_alignment'].item():.4f}")
    print(f"  Beta max: {info['beta'].max().item():.4f}")
    print(f"  Beta sparsity: {(info['beta'] > 1e-3).float().mean().item():.2%}")

    # WormGraph Integration Test
    print("\n" + "=" * 70)
    print("WormGraph51 + DKES INTEGRATION")
    print("=" * 70)

    # Mock Pantheon
    class MockPantheon:
        def __init__(self, dim):
            self.dim = dim
        def invoke(self, name):
            return torch.randn(self.dim)

    pantheon = MockPantheon(dim)

    # Mock config
    class MockConfig:
        dim = 512
        moe_num_experts = 8
        kv_cache_max_seq = 65536

    config = MockConfig()
    wg = WormGraph51_MKEL(config, pantheon)

    domain_emb = torch.randn(batch, dim)
    output, info = wg.forward_domain_mkel(domain_emb, domain_idx=0)

    print(f"\n  Input shape: {domain_emb.shape}")
    print(f"  Output shape: {output.shape}")
    print(f"  Average modulation: {torch.sigmoid(info['w']).mean().item():.4f}")
    print(f"  DKES active: {wg.dkes.num_experts} kernel-experts")

    print("\n" + "=" * 70)
    print("Seal: DKES-989.y.6-MKEL-INTEGRATION-2026-06-02")
    print("Architect ORCID: 0009-0005-2697-4668")
    print("=" * 70)
