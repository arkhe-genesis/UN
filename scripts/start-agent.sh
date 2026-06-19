#!/bin/sh
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

AGENT_BIN="/opt/agent/bin/agent"
CONFIG_DIR="/mnt/persist/config"
DATA_DIR="/mnt/persist/data"
PID_FILE="/var/run/agent.pid"

start() {
    mkdir -p "$CONFIG_DIR" "$DATA_DIR"
    if [ ! -f "$CONFIG_DIR/agent.toml" ]; then
        cp /opt/agent/config/agent.toml.default "$CONFIG_DIR/agent.toml"
    fi
    $AGENT_BIN --config "$CONFIG_DIR/agent.toml" &
    PID=$!
    echo $PID > "$PID_FILE"
    wait $PID
}

stop() {
    if [ -f "$PID_FILE" ]; then
        kill -TERM "$(cat $PID_FILE)" || true
        rm -f "$PID_FILE"
    fi
}

case "$1" in
    start) start ;;
    stop) stop ;;
    restart) stop; sleep 1; start ;;
    *) echo "Uso: $0 {start|stop|restart}" ;;
esac
