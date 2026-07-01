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

set -e

echo "==============================================="
echo "Initializing Cathedral ARKHE v28.3 Stack"
echo "==============================================="

# 1. Check for required tools
echo "[1/4] Checking dependencies..."
if ! command -v docker &> /dev/null; then
    echo "Error: docker is not installed."
    exit 1
fi
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "Error: docker-compose is not installed."
    exit 1
fi

# 2. Set up permissions and directories
echo "[2/4] Setting up volumes and permissions..."
mkdir -p models agent/memory/knowledge_base temporal-data qdrant-data
chmod -R 777 temporal-data qdrant-data

# 3. Compile the Orchestrator (Optional check, but good for local dev)
echo "[3/4] Compiling Multi-Agent Orchestrator (if Rust is installed)..."
if command -v cargo &> /dev/null; then
    cd orchestrator
    cargo build --release
    cd ..
else
    echo "Rust/Cargo not found. Skipping local build (Docker will handle it)."
fi

# 4. Launch the stack
echo "[4/4] Launching Cathedral ARKHE stack..."
cd runtime
docker compose up -d

echo "==============================================="
echo "Stack launched successfully!"
echo "LLM Server: http://localhost:8000"
echo "Agent Runtime: http://localhost:8001"
echo "Qdrant DB: http://localhost:6333"
echo "Jaeger UI: http://localhost:16686"
echo "==============================================="
