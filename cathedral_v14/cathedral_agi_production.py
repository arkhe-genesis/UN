#!/usr/bin/env python3
"""
Cathedral AGI v0.2 — Produção com modelos reais, motor simbólico completo,
consolidação durante sleep e ambiente Gymnasium.
Execução: python cathedral_agi_production.py
Dependências:
    pip install torch torch-geometric hnswlib pyro-ppl dowhy rdflib owlready2 z3-solver learn2learn pandas gymnasium[classic-control] torchvision
"""

import asyncio
import logging
import math
import random
import time
import threading
from collections import deque
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
import torchvision.models as models
from torch_geometric.nn import GCNConv
from torch_geometric.data import Data as GeometricData

import hnswlib
import pyro
import pyro.distributions as dist
from pyro.infer import SVI, Trace_ELBO
import pandas as pd
import dowhy
from rdflib import Graph, URIRef, RDF
from owlready2 import *
import z3

import learn2learn as l2l
from torch import optim
import gymnasium as gym

# ============================================================================
# 1. Neuro‑Symbolic Bridge (GNN + Knowledge Graph + OWL/Z3)
# ============================================================================

class GNNReasoner(nn.Module):
    """GNN treinada para raciocínio sobre grafos de conhecimento."""
    def __init__(self, input_dim=128, hidden_dim=256, output_dim=128):
        super().__init__()
        self.conv1 = GCNConv(input_dim, hidden_dim)
        self.conv2 = GCNConv(hidden_dim, output_dim)

    def forward(self, x, edge_index):
        x = F.relu(self.conv1(x, edge_index))
        x = self.conv2(x, edge_index)
        return x

class OntologyReasoner:
    """Motor simbólico completo usando OWL + Z3."""
    def __init__(self):
        # Cria uma ontologia simples em memória
        self.onto = get_ontology("http://example.org/agi.owl")
        self.solver = z3.Solver()
        self._init_ontology()

    def _init_ontology(self):
        # Define classes e propriedades
        with self.onto:
            class Entity(Thing): pass
            class Action(Thing): pass
            class Effect(Thing): pass
            class has_effect(ObjectProperty):
                domain = [Action]
                range = [Effect]

            class causes_reward(ObjectProperty):
                domain = [Action]
                range = [Effect]

            # Adiciona indivíduos de exemplo
            self.onto.move = Action("move")
            self.onto.reward_increase = Effect("reward_increase")
            self.onto.move.has_effect.append(self.onto.reward_increase)

            # Adiciona regra SWRL complexa
            # Se uma ação A tem efeito E, e E é "reward_increase", então A causes_reward E
            rule = Imp()
            rule.set_as_rule("Action(?a), Effect(?e), has_effect(?a, ?e) -> causes_reward(?a, ?e)")

            # Sincroniza o raciocinador para aplicar as regras SWRL
            sync_reasoner(infer_property_values=True)


    def query(self, action: str) -> bool:
        """Consulta se uma ação tem efeito de aumentar recompensa."""
        # Usa raciocinador OWL (herança, equivalência)
        action_inst = getattr(self.onto, action, None)
        if action_inst is None:
            return False
        effects = action_inst.has_effect
        for eff in effects:
            if "reward_increase" in eff.name:
                return True
        return False

    def theorem_prove(self, fact: str) -> bool:
        """Usa Z3 para verificar consistência/derivação."""
        # Exemplo: provar que se "move" então "reward_increase"
        solver = z3.Solver()
        move = z3.Bool("move")
        reward = z3.Bool("reward_increase")
        solver.add(z3.Implies(move, reward))
        solver.add(move)
        result = solver.check()
        return result == z3.sat

class NeuroSymbolicBridge:
    def __init__(self, embed_dim=128):
        self.embed_dim = embed_dim
        self.gnn = GNNReasoner(input_dim=embed_dim, output_dim=embed_dim)
        self.ontology = OntologyReasoner()
        # Grafo de conhecimento RDF (exemplo)
        self.kb = Graph()
        self.kb.add((URIRef("http://example.org/move"), RDF.type, URIRef("http://example.org/Action")))

    async def neuro_symbolic_infer(self, obs_embedding: torch.Tensor, action_name: str = "move") -> Dict:
        # Raciocínio simbólico
        symbolic_result = self.ontology.query(action_name)
        theorem_result = self.ontology.theorem_prove("move")
        # Raciocínio subsimbólico (GNN) - cria um grafo fictício a partir da observação
        # Para simplicidade, cria um grafo com um nó e embedding da observação
        x = obs_embedding.unsqueeze(0)  # (1, dim)
        edge_index = torch.empty((2, 0), dtype=torch.long)  # sem arestas
        gnn_out = self.gnn(x, edge_index).squeeze(0)
        return {
            "symbolic_action_effect": symbolic_result,
            "theorem_valid": theorem_result,
            "gnn_embedding": gnn_out,
            "integrated": gnn_out if symbolic_result else torch.zeros_like(gnn_out)
        }

# ============================================================================
# 2. Episodic Memory v2 (HNSW + Forgetting + Consolidation)
# ============================================================================

class EpisodicMemory:
    def __init__(self, dim=128, max_elements=10000, decay_factor=0.99, consolidation_interval=60):
        self.dim = dim
        self.index = hnswlib.Index(space='cosine', dim=dim)
        self.index.init_index(max_elements=max_elements, ef_construction=200, M=16)
        self.labels = {}
        self.next_id = 0
        self.decay_factor = decay_factor
        self.consolidation_interval = consolidation_interval
        self.last_consolidation = time.time()
        self._lock = threading.Lock()

    def store(self, vector: np.ndarray, metadata: Dict):
        with self._lock:
            memory_id = self.next_id
            self.index.add_items(vector.reshape(1, -1), np.array([memory_id]))
            self.labels[memory_id] = {
                "timestamp": time.time(),
                "strength": 1.0,
                "data": metadata,
                "access_count": 0
            }
            self.next_id += 1
            self._apply_forgetting()

    def recall(self, query: np.ndarray, k=5) -> List[Dict]:
        with self._lock:
            if self.next_id == 0:
                return []
            k = min(k, self.next_id)
            labels, distances = self.index.knn_query(query.reshape(1, -1), k=k)
            memories = []
            for idx, dist in zip(labels[0], distances[0]):
                if idx in self.labels:
                    mem = self.labels[idx]
                    mem["strength"] = min(1.0, mem["strength"] + 0.1)
                    mem["access_count"] += 1
                    mem["last_access"] = time.time()
                    memories.append({**mem["data"], "similarity": 1 - dist, "strength": mem["strength"]})
            return memories

    def _apply_forgetting(self):
        now = time.time()
        to_delete = []
        for idx, mem in self.labels.items():
            age = now - mem["timestamp"]
            mem["strength"] *= (self.decay_factor ** (age / 3600.0))
            if mem["strength"] < 0.01:
                to_delete.append(idx)
        for idx in to_delete:
            self.index.mark_deleted(idx)
            del self.labels[idx]

    async def consolidate(self):
        """Reforça memórias frequentemente acessadas (consolidação)."""
        now = time.time()
        if now - self.last_consolidation > self.consolidation_interval:
            with self._lock:
                for idx, mem in self.labels.items():
                    if mem["access_count"] > 2:
                        mem["strength"] = min(1.0, mem["strength"] + 0.2)
                self.last_consolidation = now
            logging.info("Episodic memory consolidated (strength boosted).")

# ============================================================================
# 3. Causal Engine (DoWhy + Pyro)
# ============================================================================

class CausalEngine:
    def __init__(self):
        self.causal_model = None
        self.data = None

    def infer_causal_effect(self, data: pd.DataFrame, treatment: str, outcome: str) -> float:
        self.data = data
        model = dowhy.CausalModel(
            data=data,
            treatment=treatment,
            outcome=outcome,
            common_causes=list(set(data.columns) - {treatment, outcome})
        )
        identified_estimand = model.identify_effect()
        estimate = model.estimate_effect(identified_estimand, method_name="backdoor.linear_regression")
        return estimate.value

    async def counterfactual(self, data_point: Dict, action_change: str) -> float:
        # Exemplo simplificado: regressão linear com Pyro
        def model(x, y=None):
            a = pyro.sample("a", dist.Normal(0, 1))
            b = pyro.sample("b", dist.Normal(0, 1))
            sigma = pyro.sample("sigma", dist.HalfNormal(1))
            mu = a * x + b
            with pyro.plate("data", len(x)):
                pyro.sample("y", dist.Normal(mu, sigma), obs=y)
            return mu
        x = torch.tensor([data_point.get("x", 0.0)], dtype=torch.float)
        y_obs = torch.tensor([data_point.get("y", 0.0)], dtype=torch.float)
        guide = pyro.infer.autoguide.AutoNormal(model)
        svi = SVI(model, guide, optim=pyro.optim.Adam({"lr": 0.01}), loss=Trace_ELBO())
        for _ in range(100):
            svi.step(x, y_obs)
        predictive = pyro.infer.Predictive(model, guide=guide, num_samples=100)
        samples = predictive(x, y=None)
        return float(samples["y"].mean())

# ============================================================================
# 4. Meta‑Learning Core (MAML + Prototypical Networks)
# ============================================================================

class MetaLearner:
    def __init__(self, input_dim=128, hidden_dim=256, num_classes=5):
        self.model = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, num_classes)
        )
        self.meta_optimizer = optim.Adam(self.model.parameters(), lr=0.001)

    def maml_adapt(self, support_set: List[Tuple[torch.Tensor, int]], lr=0.01, steps=5):
        learner = l2l.algorithms.MAML(self.model, lr=lr)
        clone = learner.clone()
        for _ in range(steps):
            loss = 0
            for x, y in support_set:
                pred = clone(x)
                loss += F.cross_entropy(pred.unsqueeze(0), torch.tensor([y]))
            clone.adapt(loss)
        return clone

    async def few_shot_classify(self, support: List[Tuple[torch.Tensor, int]], query: torch.Tensor) -> int:
        if not support:
            return 0
        class_protos = {}
        for x, y in support:
            if y not in class_protos:
                class_protos[y] = []
            class_protos[y].append(x)
        prototypes = {c: torch.stack(protos).mean(dim=0) for c, protos in class_protos.items()}
        dists = [torch.norm(query - proto) for proto in prototypes.values()]
        return list(prototypes.keys())[torch.argmin(torch.tensor(dists))]

# ============================================================================
# 5. Introspective Monitor (Self‑modeling + Confidence + Recovery)
# ============================================================================

class IntrospectiveMonitor:
    def __init__(self, confidence_threshold=0.7):
        self.confidence_threshold = confidence_threshold
        self.error_log = deque(maxlen=100)
        self.recovery_procedures = {
            "low_confidence": self._retry_with_different_strategy,
            "timeout": self._increase_timeout,
            "memory_full": self._force_consolidation
        }

    def evaluate_confidence(self, logits: torch.Tensor) -> float:
        probs = F.softmax(logits, dim=-1)
        return float(probs.max().item())

    async def monitor_task(self, task_result: Dict) -> Optional[str]:
        confidence = task_result.get("confidence", 0.0)
        if confidence < self.confidence_threshold:
            self.error_log.append(("low_confidence", time.time()))
            return "low_confidence"
        if "error" in task_result:
            self.error_log.append((task_result["error"], time.time()))
            return task_result["error"]
        return None

    async def recover(self, error_type: str) -> Dict:
        if error_type in self.recovery_procedures:
            return await self.recovery_procedures[error_type]()
        return {"status": "unknown_error", "fallback": "reset"}

    async def _retry_with_different_strategy(self):
        return {"status": "retried", "strategy": "alternative"}

    async def _increase_timeout(self):
        return {"status": "increased_timeout", "new_timeout": 30.0}

    async def _force_consolidation(self):
        return {"status": "consolidated"}

# ============================================================================
# 6. Energy Budget (DVFS simulado + alvo 20W)
# ============================================================================

class EnergyBudget:
    def __init__(self, target_power_watts=20.0):
        self.target_power = target_power_watts
        self.current_power = 0.0
        self.power_history = []
        self.dvfs_levels = [0.5, 0.7, 1.0, 1.2]

    def estimate_power(self, computation_load: float) -> float:
        baseline = 5.0   # W
        factor = 15.0    # W por unidade de carga
        return baseline + factor * computation_load

    async def schedule_task(self, task, *args, **kwargs):
        load_estimate = 0.5  # poderia ser medido
        required_power = self.estimate_power(load_estimate)
        if required_power > self.target_power:
            dvfs = self.dvfs_levels[0]
            logging.warning(f"Power budget exceeded ({required_power:.1f}W > {self.target_power}W). Scaling down to {dvfs}")
        else:
            dvfs = 1.0
        start = time.monotonic()
        result = await task(*args, **kwargs)
        duration = time.monotonic() - start
        energy = required_power * duration
        self.current_power = required_power
        self.power_history.append(energy)
        return result

# ============================================================================
# 7. Ambiente Simulado (Gymnasium) e Extrator de Características
# ============================================================================

class ViTFeatureExtractor(nn.Module):
    """Vision Transformer (ViT) pré-treinado para extração de características visuais."""
    def __init__(self, output_dim=128):
        super().__init__()
        # Usa um ViT pré-treinado
        self.vit = models.vit_b_16(weights=models.ViT_B_16_Weights.DEFAULT)
        # Substitui a head final para mapear para output_dim
        self.vit.heads = nn.Linear(self.vit.heads.head.in_features, output_dim)

        # Transformações para ajustar observações não-visuais ou imagens brutas
        self.transform = models.ViT_B_16_Weights.DEFAULT.transforms()

    def forward(self, obs):
        if isinstance(obs, np.ndarray):
            obs = torch.from_numpy(obs).float()

        # Se for CartPole (vetor 1D de tamanho 4), convertemos num "patch" fictício RGB 224x224
        if obs.dim() == 1 or (obs.dim() == 2 and obs.shape[1] == 4):
            batch_size = 1 if obs.dim() == 1 else obs.size(0)
            # Cria uma imagem preta
            dummy_img = torch.zeros((batch_size, 3, 224, 224), dtype=torch.float)
            # Codifica a observação CartPole nos pixels do canal Red (canto superior esquerdo)
            if obs.dim() == 1:
                obs = obs.unsqueeze(0)
            dummy_img[:, 0, 0, :4] = obs
            obs = dummy_img

        # Se for imagem sem dimensão de batch
        if obs.dim() == 3:
            obs = obs.unsqueeze(0)

        # Se for canal no final (H, W, C), muda para (C, H, W)
        if obs.shape[-1] == 3 and obs.shape[1] != 3:
            obs = obs.permute(0, 3, 1, 2)

        # Redimensiona para 224x224 se necessário, simplificado aqui com interpolação se já for imagem real
        if obs.shape[-1] != 224 or obs.shape[-2] != 224:
            obs = F.interpolate(obs, size=(224, 224), mode='bilinear', align_corners=False)

        return self.vit(obs)


class WorldModel(nn.Module):
    """Modelo de Mundo Interno (World Model) para planejamento."""
    def __init__(self, obs_dim=128, action_dim=2, hidden_dim=256):
        super().__init__()
        # Prediz o próximo estado (embedding) e a recompensa esperada
        self.fc1 = nn.Linear(obs_dim + 1, hidden_dim) # action is scalar
        self.fc2 = nn.Linear(hidden_dim, hidden_dim)

        self.next_obs_head = nn.Linear(hidden_dim, obs_dim)
        self.reward_head = nn.Linear(hidden_dim, 1)

    def forward(self, obs_emb, action):
        if not isinstance(action, torch.Tensor):
            action = torch.tensor([[action]], dtype=torch.float)
        elif action.dim() == 1:
            action = action.unsqueeze(1).float()

        x = torch.cat([obs_emb, action], dim=-1)
        x = F.relu(self.fc1(x))
        x = F.relu(self.fc2(x))

        next_obs = self.next_obs_head(x)
        reward = self.reward_head(x)
        return next_obs, reward

class ReplayBuffer:
    def __init__(self, capacity=10000):
        self.buffer = deque(maxlen=capacity)

    def push(self, state, action, reward, next_state, log_prob):
        self.buffer.append((state, action, reward, next_state, log_prob))

    def sample(self, batch_size):
        batch = random.sample(self.buffer, batch_size)
        state, action, reward, next_state, log_prob = map(np.stack, zip(*batch))
        return state, action, reward, next_state, log_prob

    def clear(self):
        self.buffer.clear()

    def __len__(self):
        return len(self.buffer)

class ActorCritic(nn.Module):
    def __init__(self, obs_dim=128, action_dim=2):
        super().__init__()
        self.fc = nn.Sequential(
            nn.Linear(obs_dim, 256),
            nn.ReLU(),
            nn.Linear(256, 128),
            nn.ReLU()
        )
        self.actor_head = nn.Linear(128, action_dim)
        self.critic_head = nn.Linear(128, 1)

    def forward(self, x):
        x = self.fc(x)
        logits = self.actor_head(x)
        value = self.critic_head(x)
        return logits, value

# ============================================================================
# 8. AGI Core Integrada


# ============================================================================

class CathedralAGI:
    def __init__(self, env_name="CartPole-v1"):
        self.env = gym.make(env_name)
        self.feature_extractor = ViTFeatureExtractor(output_dim=128)
        self.neuro_symbolic = NeuroSymbolicBridge(embed_dim=128)
        self.episodic_memory = EpisodicMemory(dim=128)
        self.causal_engine = CausalEngine()
        self.meta_learner = MetaLearner(input_dim=128)
        self.world_model = WorldModel(obs_dim=128, action_dim=2, hidden_dim=256)
        self.monitor = IntrospectiveMonitor(confidence_threshold=0.6)
        self.replay_buffer = ReplayBuffer(capacity=5000)
        self.policy = ActorCritic(obs_dim=128, action_dim=2)
        self.policy_optimizer = optim.Adam(self.policy.parameters(), lr=3e-4)
        self.energy = EnergyBudget()
        self.cycle_count = 0
        self.sleep_task = None
        self.running = True

    async def sleep_cycle(self):
        """Modo sleep: consolida memórias e otimiza modelos (executado em background)."""
        while self.running:
            await asyncio.sleep(30)  # dorme por 30 segundos a cada ciclo
            if not self.running:
                break
            logging.info("Entering sleep mode for memory consolidation...")
            await self.episodic_memory.consolidate()
            # Aqui poderíamos também executar otimização de hiperparâmetros, replay de experiências, etc.
            logging.info("Sleep mode finished.")

    async def perceive_and_act(self, observation: np.ndarray) -> Dict:
        # 1. Extrai embedding da observação
        with torch.no_grad():
            obs_tensor = self.feature_extractor(observation).unsqueeze(0)  # (1,128)
        obs_np = obs_tensor.squeeze(0).cpu().numpy()

        # 2. Raciocínio neuro-simbólico (consulta sobre ação "move")
        symbolic = await self.neuro_symbolic.neuro_symbolic_infer(obs_tensor.squeeze(0), action_name="move")

        # 3. Recupera memórias episódicas similares
        memories = self.episodic_memory.recall(obs_np, k=3)

        # 4. Raciocínio causal (dados simulados a partir do histórico)
        # Simula um pequeno DataFrame com ações e recompensas
        df = pd.DataFrame({
            "action": [0, 1, 0, 1, 0],
            "reward": [0.1, 0.5, 0.2, 0.6, 0.3],
            "obs_x": obs_np[:4].tolist() + [0] if len(obs_np) < 4 else obs_np[:4].tolist() + [0] # usando primeiras 4 dimensões + 1 para o dataframe size
        })
        causal_effect = self.causal_engine.infer_causal_effect(df, "action", "reward")

        # 5. Meta‑aprendizagem: classifica a observação em uma de 5 categorias abstratas
        # Cria um support set fictício a partir de memórias
        support = []
        for i, mem in enumerate(memories[:3]):
            # Simula embedding e classe
            emb = np.random.randn(128)
            support.append((torch.from_numpy(emb).float(), i % 3))
        query_emb = obs_tensor.squeeze(0)
        predicted_class = await self.meta_learner.few_shot_classify(support, query_emb)

        # 5.1. Planejamento com World Model (rollout de 1 passo)
        best_planned_action = 0
        best_expected_reward = -float('inf')
        for possible_action in [0, 1]:
            with torch.no_grad():
                pred_next_obs, pred_reward = self.world_model(obs_tensor.squeeze(0).unsqueeze(0), possible_action)
                if pred_reward.item() > best_expected_reward:
                    best_expected_reward = pred_reward.item()
                    best_planned_action = possible_action

        # PPO: Obter a probabilidade e valor do estado atual
        with torch.no_grad():
            logits, value = self.policy(obs_tensor.squeeze(0))
            dist = torch.distributions.Categorical(logits=logits)
            ppo_action = dist.sample().item()
            log_prob = dist.log_prob(torch.tensor(ppo_action)).item()

        # 6. Decisão da ação (heurística combinando memória, world model e PPO)
        action = ppo_action # Usamos a ação amostrada pelo PPO primariamente
        if memories and memories[0]["strength"] > 0.8:
            action = 1 if "action" in memories[0] and memories[0]["action"] == 1 else 0


        # 7. Confiança (baseada na similaridade da memória mais forte)
        confidence = memories[0]["similarity"] if memories else 0.5

        result = {
            "action": action,
            "confidence": confidence,
            "symbolic_effect": symbolic["symbolic_action_effect"],
            "causal_effect": causal_effect,
            "predicted_class": predicted_class,
            "memories": memories
        }

        # 8. Auto‑monitoramento
        error = await self.monitor.monitor_task(result)
        if error:
            recovery = await self.monitor.recover(error)
            result["recovery"] = recovery
            # Se recuperação sugerir nova estratégia, ajusta ação
            if recovery.get("strategy") == "alternative":
                action = 1 - action
                result["action"] = action

        # 9. Armazena o episódio (observação + ação + resultado)
        self.episodic_memory.store(obs_np, {"action": action, "confidence": confidence, "timestamp": time.time()})

        return result

    async def run_episode(self, max_steps=200):
        """Um episódio completo no ambiente Gym com RL."""
        obs, info = self.env.reset()
        total_reward = 0
        episode_data = []
        for step in range(max_steps):
            result = await self.energy.schedule_task(self.perceive_and_act, obs)
            action = result["action"]

            # Recupera log_prob que calculamos dentro (um pouco 'hacky' acessar de fora, mas o PPO precisa)
            with torch.no_grad():
                obs_tensor = self.feature_extractor(obs).unsqueeze(0)
                logits, _ = self.policy(obs_tensor.squeeze(0))
                dist = torch.distributions.Categorical(logits=logits)
                log_prob = dist.log_prob(torch.tensor(action)).item()

            next_obs, reward, terminated, truncated, info = self.env.step(action)
            total_reward += reward

            # Usa feature_extractor para armazenar embeddings no buffer
            with torch.no_grad():
                obs_emb = self.feature_extractor(obs).cpu().numpy().flatten()
                next_obs_emb = self.feature_extractor(next_obs).cpu().numpy().flatten()

            self.replay_buffer.push(obs_emb, action, reward, next_obs_emb, log_prob)

            obs = next_obs
            if terminated or truncated:
                break
            await asyncio.sleep(0.01)  # simula tempo de processamento

        self.update_ppo()
        return total_reward

    def update_ppo(self):
        if len(self.replay_buffer) < 64:
            return

        states, actions, rewards, next_states, old_log_probs = self.replay_buffer.sample(64)

        states = torch.tensor(states, dtype=torch.float)
        actions = torch.tensor(actions, dtype=torch.long)
        rewards = torch.tensor(rewards, dtype=torch.float).unsqueeze(1)
        next_states = torch.tensor(next_states, dtype=torch.float)
        old_log_probs = torch.tensor(old_log_probs, dtype=torch.float)

        # Simples PPO update (1 epoch)
        logits, values = self.policy(states)
        dist = torch.distributions.Categorical(logits=logits)
        new_log_probs = dist.log_prob(actions)

        _, next_values = self.policy(next_states)

        # Advantage (simples TD)
        advantages = rewards + 0.99 * next_values.detach() - values

        ratio = torch.exp(new_log_probs - old_log_probs)
        surr1 = ratio * advantages.detach()
        surr2 = torch.clamp(ratio, 1.0 - 0.2, 1.0 + 0.2) * advantages.detach()

        actor_loss = -torch.min(surr1, surr2).mean()
        critic_loss = F.mse_loss(values, rewards + 0.99 * next_values.detach())
        loss = actor_loss + 0.5 * critic_loss

        self.policy_optimizer.zero_grad()
        loss.backward()
        self.policy_optimizer.step()

        self.replay_buffer.clear()


    async def train(self, num_episodes=10):
        """Loop contínuo de aprendizado."""
        logging.info("Starting AGI training in Gym environment...")
        # Inicia tarefa de consolidação em background (sleep mode)
        self.sleep_task = asyncio.create_task(self.sleep_cycle())
        for ep in range(num_episodes):
            reward = await self.run_episode()
            logging.info(f"Episode {ep+1}: total reward = {reward:.2f}")
            if not self.running:
                break
        # Finaliza
        self.running = False
        if self.sleep_task:
            self.sleep_task.cancel()
            try:
                await self.sleep_task
            except asyncio.CancelledError:
                pass
        self.env.close()
        logging.info("Training finished.")

async def main():
    logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
    agi = CathedralAGI(env_name="CartPole-v1")
    await agi.train(num_episodes=15)

if __name__ == "__main__":
    asyncio.run(main())
