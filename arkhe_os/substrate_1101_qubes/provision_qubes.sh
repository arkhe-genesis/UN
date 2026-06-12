#!/bin/bash
# ============================================================
# CATHEDRAL ARKHE v12.0 — Provisionamento de Qubes OS
# Selo: CATHEDRAL-QUBES-1101-v1.0.0-2026-06-12
# ============================================================
# Este script deve ser executado no dom0.
# ATENÇÃO: Verifique a configuração de IOMMU e GPU passthrough
# antes de executar este script em produção.

set -e

echo "=> Iniciando provisionamento da arquitetura Cathedral sobre Qubes OS..."

# ============================================================
# 1. Template Base
# ============================================================
echo "=> Criando template base..."
# Clonar template minimal Fedora
qvm-clone fedora-39-minimal cathedral-template || echo "Template cathedral-template já existe."

# Instalar dependências comuns no template
qvm-run -u root cathedral-template "dnf install -y python3 python3-pip rust cargo golang"

# Atualizar template
qvm-run -u root cathedral-template "dnf upgrade -y"

# ============================================================
# 2. AGI-CORE: Orquestrador principal
# ============================================================
echo "=> Provisionando agi-core..."
qvm-create -l red -t cathedral-template agi-core || echo "agi-core já existe."
qvm-prefs agi-core netvm sys-firewall
qvm-prefs agi-core provides_network false
qvm-prefs agi-core memory 4096
qvm-prefs agi-core maxmem 8192
qvm-prefs agi-core vcpus 4

# ============================================================
# 3. LLM-INFERENCE: Inferência com GPU passthrough
# ============================================================
echo "=> Provisionando llm-inference..."
qvm-create -l black -t cathedral-template llm-inference || echo "llm-inference já existe."
qvm-prefs llm-inference netvm none          # AIR-GAPPED
qvm-prefs llm-inference memory 16384
qvm-prefs llm-inference maxmem 32768
qvm-prefs llm-inference vcpus 8

echo "ATENÇÃO: Lembre-se de configurar o GPU passthrough para o llm-inference."
echo "Exemplo: qvm-pci attach llm-inference dom0:00:02.0 --persistent"

# ============================================================
# 4. KNOWLEDGE-BASE: Memória persistente
# ============================================================
echo "=> Provisionando knowledge-base..."
qvm-create -l black -t cathedral-template knowledge-base || echo "knowledge-base já existe."
qvm-prefs knowledge-base netvm none           # AIR-GAPPED
qvm-prefs knowledge-base memory 4096
qvm-prefs knowledge-base maxmem 8192

# Instalar PostgreSQL + pgvector no template (para ser efetivado, seria melhor em um template específico, mas seguindo o spec)
qvm-run -u root cathedral-template "dnf install -y postgresql-server postgresql-contrib"

# ============================================================
# 5. GOVERNANCE: Assinatura e ancoragem
# ============================================================
echo "=> Provisionando governance..."
qvm-create -l black -t cathedral-template governance || echo "governance já existe."
qvm-prefs governance netvm none             # AIR-GAPPED
qvm-prefs governance memory 2048
qvm-prefs governance maxmem 4096

# Instalar Rust + blst no template
qvm-run -u root cathedral-template "cargo install blst"

# ============================================================
# 6. CRYPTO-VM: Operações criptográficas (air-gapped)
# ============================================================
echo "=> Provisionando crypto-vm..."
qvm-create -l black -t cathedral-template crypto-vm || echo "crypto-vm já existe."
qvm-prefs crypto-vm netvm none              # AIR-GAPPED
qvm-prefs crypto-vm memory 2048

# ============================================================
# 7. VMs DE AÇÃO (Músculos)
# ============================================================
echo "=> Provisionando VMs de ação (browser, email, code)..."
qvm-create -l yellow -t cathedral-template browser-vm || echo "browser-vm já existe."
qvm-prefs browser-vm netvm sys-whonix       # Tor por padrão
qvm-prefs browser-vm memory 2048

qvm-create -l yellow -t cathedral-template email-vm || echo "email-vm já existe."
qvm-prefs email-vm netvm sys-firewall
qvm-prefs email-vm memory 2048

qvm-create -l yellow -t cathedral-template code-vm || echo "code-vm já existe."
qvm-prefs code-vm netvm sys-firewall
qvm-prefs code-vm memory 4096

# ============================================================
# 8. DISPVM TEMPLATE (para tarefas não confiáveis)
# ============================================================
echo "=> Provisionando template de DispVM (cathedral-dvm)..."
qvm-create -l green -t cathedral-template cathedral-dvm || echo "cathedral-dvm já existe."
qvm-prefs cathedral-dvm template_for_dispvms True

echo "=> Provisionamento concluído."
