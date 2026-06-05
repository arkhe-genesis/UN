#!/usr/bin/env python3
import click
import json
from rich.console import Console
from rich.table import Table
from rich.panel import Panel

from vault_manager import VaultManager
from tx_lifecycle import TransactionLifecycle
from policy_engine import PolicyEngine
from care_bridge import CAREBridge
from zk_proof_generator import ZKProofGenerator
from rbb_anchor import RBBAchor
from theosis_injector import TheosisInjector

console = Console()

@click.group(name="fordefi")
def fordefi_cli():
    pass

@fordefi_cli.group()
def vault():
    pass

@vault.command("create")
@click.option("--name", required=True, help="Nome do vault")
@click.option("--chains", required=True, help="Chains separadas por virgula")
@click.option("--policy", help="Arquivo YAML de politica Axiarquia")
def vault_create(name, chains, policy):
    mgr = VaultManager()
    result = mgr.create_vault(name, chains.split(","), policy)
    console.print(Panel(json.dumps(result, indent=2), title="🏦 Vault Criado", border_style="green"))

@vault.command("list")
def vault_list():
    mgr = VaultManager()
    vaults = mgr.list_vaults()
    table = Table(title="🏦 Vaults Fordefi", show_header=True)
    table.add_column("Vault ID", style="cyan")
    table.add_column("Nome", style="green")
    table.add_column("Chains", style="yellow")
    table.add_column("Status", style="blue")
    table.add_column("Sync", style="magenta")
    for v in vaults:
        table.add_row(v["vault_id"], v["name"], ", ".join(v["chains"]), v["status"], "🟢" if v["remote_sync"] else "🔴")
    console.print(table)

@vault.command("status")
@click.argument("vault_id")
def vault_status(vault_id):
    mgr = VaultManager()
    status = mgr.get_vault_status(vault_id)
    console.print(Panel(json.dumps(status, indent=2), title=f"📊 Status {vault_id}", border_style="blue"))

@fordefi_cli.group()
def tx():
    pass

@tx.command("create")
@click.option("--vault", required=True, help="ID do vault")
@click.option("--to", required=True, help="Endereco destino")
@click.option("--amount", required=True, help="Valor")
@click.option("--chain", required=True, help="Chain")
def tx_create(vault, to, amount, chain):
    lifecycle = TransactionLifecycle()
    result = lifecycle.create(vault, to, amount, chain)
    console.print(Panel(json.dumps(result, indent=2), title="📝 Transacao Criada", border_style="yellow"))

@tx.command("simulate")
@click.argument("tx_id")
def tx_simulate(tx_id):
    lifecycle = TransactionLifecycle()
    result = lifecycle.simulate(tx_id)
    color = "red" if result["status"] == "SIMULATION_FAILED" else "green"
    console.print(Panel(json.dumps(result, indent=2), title="🔍 Simulacao", border_style=color))

@tx.command("sign")
@click.argument("tx_id")
def tx_sign(tx_id):
    lifecycle = TransactionLifecycle()
    result = lifecycle.sign(tx_id)
    console.print(Panel(json.dumps(result, indent=2), title="✍️ Assinatura MPC", border_style="blue"))

@tx.command("submit")
@click.argument("tx_id")
def tx_submit(tx_id):
    lifecycle = TransactionLifecycle()
    result = lifecycle.submit(tx_id)
    console.print(Panel(json.dumps(result, indent=2), title="🚀 Broadcast", border_style="green"))

@tx.command("watch")
@click.argument("tx_id")
def tx_watch(tx_id):
    lifecycle = TransactionLifecycle()
    result = lifecycle.watch(tx_id)
    console.print(Panel(json.dumps(result, indent=2), title="👁️ Monitoramento", border_style="blue"))

@fordefi_cli.group()
def policy():
    pass

@policy.command("apply")
@click.argument("vault_id")
@click.argument("policy_file")
def policy_apply(vault_id, policy_file):
    engine = PolicyEngine()
    result = engine.apply_policy(vault_id, policy_file)
    console.print(Panel(json.dumps(result, indent=2), title="📜 Politica Aplicada", border_style="green"))

@policy.command("audit")
@click.argument("vault_id")
def policy_audit(vault_id):
    engine = PolicyEngine()
    result = engine.audit(vault_id)
    color = "green" if result["status"] == "COMPLIANT" else "red"
    console.print(Panel(json.dumps(result, indent=2), title="🔍 Audit", border_style=color))

@fordefi_cli.group()
def care():
    pass

@care.command("enable")
@click.option("--vault", required=True, help="ID do vault")
@click.option("--name", required=True, help="Nome do trigger")
@click.option("--trigger", required=True, help="Condicao (ex: price_drop>10%)")
@click.option("--action", required=True, help="Acao (ex: hedge_via_dex)")
def care_enable(vault, name, trigger, action):
    care = CAREBridge()
    result = care.create_trigger(vault, name, trigger, action)
    console.print(Panel(json.dumps(result, indent=2), title="⚡ CARE Trigger", border_style="yellow"))

@care.command("list")
@click.option("--vault", help="Filtrar por vault")
def care_list(vault):
    care = CAREBridge()
    triggers = care.list_triggers(vault)
    table = Table(title="⚡ CARE Triggers", show_header=True)
    table.add_column("ID", style="cyan")
    table.add_column("Nome", style="green")
    table.add_column("Condicao", style="yellow")
    table.add_column("Acao", style="blue")
    table.add_column("Status", style="magenta")
    for t in triggers:
        table.add_row(t["trigger_id"], t["name"], t["condition"], t["action"], t["status"])
    console.print(table)

@fordefi_cli.group()
def zk():
    pass

@zk.command("prove")
@click.argument("operation_id")
@click.option("--type", default="vault_operation", help="Tipo de operacao")
@click.option("--vault", required=True, help="ID do vault")
def zk_prove(operation_id, type, vault):
    generator = ZKProofGenerator()
    result = generator.generate_proof(operation_id, type, vault, {}, "APPROVED", 0.93)
    console.print(Panel(json.dumps(result, indent=2), title="🔐 ZK-Proof", border_style="green"))

@zk.command("verify")
@click.argument("proof_id")
def zk_verify(proof_id):
    generator = ZKProofGenerator()
    result = generator.verify_proof(proof_id)
    color = "green" if result["valid"] else "red"
    console.print(Panel(json.dumps(result, indent=2), title="✅ Verificacao", border_style=color))

@zk.command("anchor")
@click.argument("proof_id")
@click.argument("block_number", type=int)
def zk_anchor(proof_id, block_number):
    generator = ZKProofGenerator()
    result = generator.anchor_to_rbb(proof_id, block_number)
    console.print(Panel(json.dumps(result, indent=2), title="⚓ RBB Anchor", border_style="blue"))

@fordefi_cli.command("dashboard")
def dashboard():
    injector = TheosisInjector()
    data = injector.get_dashboard_data()
    console.print(Panel(json.dumps(data, indent=2), title="📊 Theosis-Paris Dashboard", border_style="magenta"))

@fordefi_cli.command("risk")
@click.argument("vault_id")
def risk(vault_id):
    from fordefi_client import FordefiClient
    client = FordefiClient()
    score = client.get_risk_score(vault_id)
    console.print(Panel(json.dumps(score, indent=2), title="🛡️ Risk Score", border_style="yellow"))

if __name__ == "__main__":
    fordefi_cli()
