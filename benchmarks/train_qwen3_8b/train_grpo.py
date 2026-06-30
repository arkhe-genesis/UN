def accuracy_score(task_output, ground_truth):
    # Dummy accuracy score implementation for completeness
    return 1.0

def reward_function(task_output, ground_truth, subtask_delegations, zk_costs):
    # Recompensa base (acurácia da tarefa)
    reward = accuracy_score(task_output, ground_truth)
    # Penaliza cada prova desnecessária
    for delegation in subtask_delegations:
        if delegation['needs_zk'] and not delegation['actual_zk_used']:
            reward -= 0.1  # penalidade por não usar prova quando necessário
        elif not delegation['needs_zk'] and delegation['actual_zk_used']:
            reward -= delegation['zk_cost']  # custo proporcional (ex: 0.02)
    return reward
