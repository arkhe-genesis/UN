#!/usr/bin/env bash
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
# scripts/bisect-arkhe.sh
# Uso: git bisect start <bad_commit> <good_commit> && git bisect run ./scripts/bisect-arkhe.sh

set -e

echo "🧪 Testando commit $(git rev-parse --short HEAD)"

# 1. Limpar cache para evitar falsos positivos (opcional, mas recomendado)
# cargo clean > /dev/null 2>&1

# 2. Compilar o workspace (se falhar, pula o commit)
if ! cargo check --workspace --all-features 2>&1; then
    echo "⚠️  Não compila - pulando"
    exit 125
fi

# 3. Executar testes críticos de regressão (SSRF, Orquestração, Memória)
# Modifique os filtros '-p' e '--test' conforme o bug que você está caçando.
if cargo test -p arkhe-ssrf-guard -p arkhe-orchestrator --quiet; then
    echo "✅ Teste passou - commit BOM"
    exit 0
else
    echo "❌ Teste falhou - commit RUIM"
    exit 1
fi
