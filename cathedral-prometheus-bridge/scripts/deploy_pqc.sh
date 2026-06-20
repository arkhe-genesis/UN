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

# Script para implantar com configuração PQC

ENV=${1:-staging}
case $ENV in
    staging)
        export CATHEDRAL_SIGNATURE_ALG=MlDsa
        export CATHEDRAL_FALLBACK_SIGNATURE_ALG=Ed25519
        export CATHEDRAL_DUAL_STACK=true
        ;;
    production-canary)
        export CATHEDRAL_SIGNATURE_ALG=MlDsa
        export CATHEDRAL_FALLBACK_SIGNATURE_ALG=Ed25519
        export CATHEDRAL_DUAL_STACK=true
        export CATHEDRAL_FORCE_PQC=false
        ;;
    production)
        export CATHEDRAL_SIGNATURE_ALG=MlDsa
        export CATHEDRAL_FALLBACK_SIGNATURE_ALG=Ed25519
        export CATHEDRAL_DUAL_STACK=false
        ;;
    *)
        echo "Uso: $0 {staging|production-canary|production}"
        ;;
esac

echo "Deployando com configuração:"
env | grep CATHEDRAL_

echo "docker-compose up -d --build"
