import os

dst = "./cathedral-arkhe-v9"
os.makedirs(dst, exist_ok=True)

dirs_to_create = [
    "cathedral/models/backbone/v9",
    "cathedral/models/theosis/v9",
    "cathedral/models/world_model",
    "cathedral/models/agentic",
    "cathedral/models/multimodal",
    "cathedral/models/distillation",
    "cathedral/models/verification",
    "cathedral/models/decentralized",
    "cathedral/orchestrator",
    "cathedral/config/v9",
    "config/plugins",
    "tests", "docs", "scripts", "examples",
]
for d in dirs_to_create:
    os.makedirs(f"{dst}/{d}", exist_ok=True)
