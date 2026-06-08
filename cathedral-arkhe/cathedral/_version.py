# cathedral/_version.py
"""Versão centralizada — único local para modificar."""

__version__ = "5.1.0"
__version_info__ = (5, 1, 0)  # major, minor, patch

# Selos (atualizados junto com a versão)
SEALS = {
    "GGUF-BRIDGE": "GGUF-BRIDGE-1094.1-v3.0.0-2026-06-07",
    "LLAMA-CPP-BRIDGE": "LLAMA-CPP-BRIDGE-1094.2-v3.0.0-2026-06-07",
    "VECTOR-THEOSIS": "VECTOR-THEOSIS-1091.2-v4.0.0-2026-06-07",
    "STETHOSCOPE": "STETHOSCOPE-1081.1-v3.0.0-2026-06-07",
    "KLEROS": "KLEROS-TRIGGER-1085.1-v2.0.0-2026-06-07",
    "ZKML-BRIDGE": "ZKML-BRIDGE-1095.1-v2.0.0-2026-06-07",
    "AGENTIC-LOOP": "AGENTIC-LOOP-1096-v1.0.0-2026-06-07",
    "TEMPORALCHAIN": "TEMPORALCHAIN-1097-v2.0.0-2026-06-07",
    "LORA-FINETUNER": "LORA-FINETUNER-1098-v1.0.0-2026-06-07",
    "GARAK-BRIDGE": "GARAK-BRIDGE-1099-v1.0.0-2026-06-07",
    "ORCHESTRATOR": f"ORCHESTRATOR-v{__version__}-2026-06-07",
    "ECOSYSTEM": f"ARKHE-ECOSYSTEM-v{__version__}-2026-06-07",
}

# Datas dos selos (para validação temporal)
SEAL_EPOCH = "2026-06-07"
