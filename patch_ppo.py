import re

with open("cathedral_v14/cathedral_agi_production.py", "r") as f:
    content = f.read()

ppo_class = """class ReplayBuffer:
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
"""

content = content.replace("# ============================================================================\n# 8. AGI Core Integrada", ppo_class)

init_ag_core = """        self.monitor = IntrospectiveMonitor(confidence_threshold=0.6)
        self.replay_buffer = ReplayBuffer(capacity=5000)
        self.policy = ActorCritic(obs_dim=128, action_dim=2)
        self.policy_optimizer = optim.Adam(self.policy.parameters(), lr=3e-4)"""

content = content.replace("        self.monitor = IntrospectiveMonitor(confidence_threshold=0.6)", init_ag_core)

act_and_ppo = """        # 5.1. Planejamento com World Model (rollout de 1 passo)
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
"""

content = re.sub(r'        # 5\.1\. Planejamento com World Model \(rollout de 1 passo\).*?            action = 1 if "action" in memories\[0\] and memories\[0\]\["action"\] == 1 else 0', act_and_ppo, content, flags=re.DOTALL)


run_episode = """    async def run_episode(self, max_steps=200):
        \"\"\"Um episódio completo no ambiente Gym com RL.\"\"\"
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
"""

content = re.sub(r'    async def run_episode\(self, max_steps=200\):.*?        return total_reward', run_episode, content, flags=re.DOTALL)

with open("cathedral_v14/cathedral_agi_production.py", "w") as f:
    f.write(content)
