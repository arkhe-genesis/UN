import re

with open("cathedral_v14/cathedral_agi_production.py", "r") as f:
    content = f.read()

world_model_class = """
class WorldModel(nn.Module):
    \"\"\"Modelo de Mundo Interno (World Model) para planejamento.\"\"\"
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

# ============================================================================
# 8. AGI Core Integrada
"""

content = content.replace("# ============================================================================\n# 8. AGI Core Integrada", world_model_class)

init_ag_core = """        self.meta_learner = MetaLearner(input_dim=128)
        self.world_model = WorldModel(obs_dim=128, action_dim=2, hidden_dim=256)
        self.monitor = IntrospectiveMonitor(confidence_threshold=0.6)"""

content = content.replace("""        self.meta_learner = MetaLearner(input_dim=128)
        self.monitor = IntrospectiveMonitor(confidence_threshold=0.6)""", init_ag_core)

perceive_and_act_planning = """        # 5.1. Planejamento com World Model (rollout de 1 passo)
        best_planned_action = 0
        best_expected_reward = -float('inf')
        for possible_action in [0, 1]:
            with torch.no_grad():
                pred_next_obs, pred_reward = self.world_model(obs_tensor.squeeze(0).unsqueeze(0), possible_action)
                if pred_reward.item() > best_expected_reward:
                    best_expected_reward = pred_reward.item()
                    best_planned_action = possible_action

        # 6. Decisão da ação (heurística combinando memória e world model)
        action = best_planned_action
        if memories and memories[0]["strength"] > 0.8:
            action = 1 if "action" in memories[0] and memories[0]["action"] == 1 else 0"""

content = re.sub(r'        # 6\. Decisão da ação \(heurística: se força da memória > 0\.5, repete ação anterior\).*?        if memories and memories\[0\]\["strength"\] > 0\.5:\n            action = 1 if "action" in memories\[0\] else 0', perceive_and_act_planning, content, flags=re.DOTALL)

with open("cathedral_v14/cathedral_agi_production.py", "w") as f:
    f.write(content)
