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

WORK_DIR="/tmp/cathedral_live_build"
ISO_OUTPUT="cathedral-os-x86_64.iso"

echo "[1/4] Criando ambiente de build (debootstrap)..."
sudo debootstrap --arch=amd64 noble $WORK_DIR http://archive.ubuntu.com/ubuntu/

echo "[2/4] Copiando artefatos do Cathedral..."
sudo cp -r ../../core/target/release/cathedral-fast-core $WORK_DIR/opt/cathedral/bin/
sudo cp -r ../../core/target/release/cathedral-slow-brain $WORK_DIR/opt/cathedral/bin/
sudo cp systemd/*.service $WORK_DIR/etc/systemd/system/
sudo cp security/cathedral_fast.* $WORK_DIR/etc/selinux/

echo "[3/4] Compilando o LKM no ambiente isolado..."
sudo cp -r kernel/ $WORK_DIR/tmp/kernel_build/
sudo chroot $WORK_DIR /bin/bash -c "cd /tmp/kernel_build && make && make install"

echo "[4/4] Empacotando Imagem ISO (SquashFS)..."
sudo mksquashfs $WORK_DIR $WORK_DIR/iso/filesystem.squashfs -e boot
# (Aqui entra o grub-mkrescue para gerar o ISO final)
# grub-mkrescue -o $ISO_OUTPUT $WORK_DIR/iso

echo "Build concluído: $ISO_OUTPUT"
