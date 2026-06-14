class UNMultiAgentRouter:
    def __init__(self):
        self.domains = {
            "humanitarian": ["OCHA", "WFP", "UN Global Pulse"],
            "ethics_rights": ["UNESCO", "OHCHR", "UNICEF"],
            "health": ["WHO"],
            "environment": ["UNEP"]
        }
        self.experts = self._init_experts()

    def _init_experts(self):
        experts = {}
        for domain, agencies in self.domains.items():
            for agency in agencies:
                experts[agency] = {"endpoint": f"slow_brain_{agency.lower()}_expert"}
        return experts

    def _detect_domain(self, query: str, context: dict) -> str:
        query_lower = query.lower() if query else ""
        if any(kw in query_lower for kw in ["health", "disease", "virus", "who"]):
            return "health"
        elif any(kw in query_lower for kw in ["climate", "environment", "nature", "unep"]):
            return "environment"
        elif any(kw in query_lower for kw in ["rights", "ethics", "children", "education", "unesco", "unicef", "ohchr"]):
            return "ethics_rights"
        else:
            return "humanitarian"

    def route(self, query: str, context: dict) -> str:
        domain = self._detect_domain(query, context)
        # Prioriza agência específica se presente no contexto
        agency = context.get("agency", self.domains[domain][0])
        if agency not in self.experts:
            return "default_slow_brain"
        return self.experts[agency]["endpoint"]

    def resolve(self, fast_state):
        query = getattr(fast_state, 'query', '')
        context = getattr(fast_state, 'context', {})
        domain = self._detect_domain(query, context)
        agency = context.get("agency", self.domains[domain][0])
        return agency, domain
