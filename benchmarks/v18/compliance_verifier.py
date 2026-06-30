class UNComplianceVerifier:
    def __init__(self):
        self.unicef_requirements = {
            "safety_children": True,
            "data_privacy_by_design": True,
            "non_discrimination": True,
            "child_participation": False  # requer aprovação explícita
        }
        self.unesco_principles = [
            "human_rights", "inclusion", "diversity",
            "transparency", "fairness", "sustainability"
        ]
        self.ohchr_concerns = ["surveillance", "discrimination", "creativity_restriction"]

    def verify(self, action: dict, agency: str) -> tuple[bool, list[str]]:
        violations = []
        if agency == "UNICEF":
            if action.get("uses_child_data") and not action.get("privacy_by_design"):
                violations.append("UNICEF data privacy violation")
            if action.get("affects_children") and not action.get("child_safety_assessed"):
                violations.append("UNICEF child safety requirement")
        elif agency == "UNESCO":
            for principle in self.unesco_principles:
                if not action.get(f"unesco_{principle}", False):
                    violations.append(f"Missing UNESCO principle: {principle}")
        elif agency == "OHCHR":
            for concern in self.ohchr_concerns:
                if action.get(concern, False):
                    violations.append(f"OHCHR concern: {concern} not mitigated")
        return len(violations) == 0, violations
