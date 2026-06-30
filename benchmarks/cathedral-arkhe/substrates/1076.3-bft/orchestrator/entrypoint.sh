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

if [ ! -f "/tee/keys/seed.bin" ]; then
    echo "Primeira execução: gerando identidade..."
    /usr/bin/orchestrator --generate-key --output /tee/keys/seed.bin --pubout /tee/keys/pub.bin
else
    echo "Carregando identidade existente..."
    /usr/bin/orchestrator --load-key /tee/keys/seed.bin --pub /tee/keys/pub.bin
fi

# Iniciar o serviço BFT
exec /usr/bin/orchestrator --consensus --id ${ORCHESTRATOR_ID} --config /config/bft.yaml
