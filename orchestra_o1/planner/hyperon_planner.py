class DummyLLM:
    def decompose(self, task: str, context: dict) -> list:
        return [f"decompose_1_for_{task}", f"decompose_2_for_{task}"]

class DummySpace:
    def run_metta(self, query: str):
        return 0.9

def init_cathedral_atomspace():
    return DummySpace()

class HyperonPlanner:
    def __init__(self):
        self.space = init_cathedral_atomspace()
        self.llm = DummyLLM()
        # Carrega regras de planejamento específicas do domínio
        self.load_rules("rules.metta")

    def load_rules(self, rules_file: str):
        pass

    def plan(self, task: str, context: dict) -> list:
        # 1. LLM gera possíveis decomposições (seed)
        llm_candidates = self.llm.decompose(task, context)
        # 2. Para cada candidato, consulta Hyperon para verificar factibilidade
        best_plan = None
        best_score = -float('inf')
        for candidate in llm_candidates:
            query = f"(plan-valid? '{candidate}')"
            score = self.space.run_metta(query)  # retorna um valor simbólico
            if score > best_score:
                best_plan = candidate
                best_score = score
        return best_plan
