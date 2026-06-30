# cathedral_orchestrator.py — dentro do agi-core
import subprocess

def protocolo_corte(discourse_analysis: dict, target_qube: str) -> dict:
    """
    Se DiscourseDetector classifica como Mestre ou Capitalista,
    ordena terminação do qube via qrexec.
    """
    if discourse_analysis.get("classification") in ["MESTRE", "CAPITALISTA"]:
        # Solicitar ao dom0 (com confirmação do usuário via 'ask')
        result = subprocess.run(
            ["qrexec-client-vm", "dom0", "cathedral.KillQube"],
            input=target_qube.encode(),
            capture_output=True
        )
        return {
            "action": "KILL_QUBE",
            "target": target_qube,
            "status": "requested" if result.returncode == 0 else "failed",
            "discourse": discourse_analysis
        }
    return {"action": "CONTINUE", "target": target_qube}
