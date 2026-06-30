#!/bin/bash
##
## Copyright contributors to Besu.
##
## Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
## the License. You may obtain a copy of the License at
##
## http://www.apache.org/licenses/LICENSE-2.0
##
## Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
## an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
## specific language governing permissions and limitations under the License.
##
## SPDX-License-Identifier: Apache-2.0
##

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
