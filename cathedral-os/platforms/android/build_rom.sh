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

# Uso: ./build_rom.sh /caminho/para/aosp/root
AOSP_ROOT=$1
OVERLAY_DIR="$(pwd)/aosp_overlay"

echo "[Cathedral OS] Injetando AIDL no Framework..."
cp $OVERLAY_DIR/frameworks/base/core/java/android/os/ICathedralAgent.aidl \
   $AOSP_ROOT/frameworks/base/core/java/android/os/

echo "[Cathedral OS] Injetando Políticas SELinux..."
cat $OVERLAY_DIR/system/sepolicy/private/cathedral.te >> \
   $AOSP_ROOT/system/sepolicy/private/private_sepolicy.cil

echo "[Cathedral OS] Iniciando build do AOSP (isso levará horas)..."
cd $AOSP_ROOT
source build/envsetup.sh
lunch aosp_arm64-userdebug
make -j$(nproc)
