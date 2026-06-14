import re

with open("cathedral_v14/cathedral_agi_production.py", "r") as f:
    content = f.read()

ontology_init = """    def _init_ontology(self):
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
"""

content = re.sub(r'    def _init_ontology\(self\):.*?            self\.onto\.move\.has_effect\.append\(self\.onto\.reward_increase\)', ontology_init, content, flags=re.DOTALL)

with open("cathedral_v14/cathedral_agi_production.py", "w") as f:
    f.write(content)
