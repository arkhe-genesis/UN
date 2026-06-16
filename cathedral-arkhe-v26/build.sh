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

# build.sh — Cathedral ARKHE v26.2 Build Script

set -e

echo "═══════════════════════════════════════════════════════════════"
echo "  Cathedral ARKHE v26.2 — Build System"
echo "  TensorZKP GPU Daemon + CM4 Wire Protocol"
echo "═══════════════════════════════════════════════════════════════"

# Build daemon (requires CUDA)
echo ""
echo "[1/3] Building TensorZKP GPU Daemon..."
cargo build --bin tensorzkp-daemon --features daemon --release

# Build CM4 firmware (cross-compile)
echo ""
echo "[2/3] Building CM4 Firmware..."
cargo build --bin cathedral-cm4 --no-default-features --features cm4 --target thumbv7em-none-eabihf --release

# Run tests
echo ""
echo "[3/3] Running tests..."
cargo test --features daemon --lib
cargo test --features cm4 --lib

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  BUILD COMPLETE"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Artifacts:"
echo "  target/release/tensorzkp-daemon     — GPU Daemon (x86_64 + CUDA)"
echo "  target/thumbv7em-none-eabihf/release/cathedral-cm4  — CM4 Firmware"
echo ""
