#!/bin/bash
# entrypoint.sh para o container do orquestrador
# Implementação do Fluxo de Boot Seguro para a Cathedral ARKHE

set -e

# Criar pasta pro TEE caso não exista (emulação/teste)
mkdir -p /tmp/tee/keys

if [ ! -f "/tmp/tee/keys/seed.bin" ]; then
    echo "Primeira execução: gerando identidade..."
    # Para fins de simulação/mock:
    # Se tivéssemos um binário 'orchestrator' compilado com suporte CLI:
    # /usr/bin/orchestrator --generate-key --output /tmp/tee/keys/seed.bin --pubout /tmp/tee/keys/pub.bin

    # Aqui criamos arquivos mock para marcar que o gerador de chave rodou
    echo "MOCK_SEED" > /tmp/tee/keys/seed.bin
    echo "MOCK_PUB" > /tmp/tee/keys/pub.bin
    echo "Identidade soberana estabelecida e salva no TEE mock."
else
    echo "Carregando identidade existente do TEE..."
    # /usr/bin/orchestrator --load-key /tmp/tee/keys/seed.bin --pub /tmp/tee/keys/pub.bin
fi

# Iniciar o serviço BFT / Orquestrador
echo "Iniciando consenso BFT do Orquestrador: ${ORCHESTRATOR_ID:-RSI_ALPHA}..."
exec python3 /app/arkhe_os/substrate_1076.3/integrated_orchestrator_1091_1076_3.py
