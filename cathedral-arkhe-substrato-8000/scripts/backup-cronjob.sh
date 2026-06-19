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

# Backup script for Cathedral ARKHE Postgres (WormGraph) and Redis (DLQ/Cache)
# Designed to be run via cronjob
# Selo: CATHEDRAL-ARKHE-8000-BACKUP-v2.1.0-2026-06-19

set -euo pipefail

BACKUP_DIR="/mnt/persist/backups"
DATE=$(date +"%Y%m%d_%H%M%S")
RETENTION_DAYS=30

mkdir -p "$BACKUP_DIR"

echo "[$(date)] Starting backup process..."

# 1. Postgres Backup (WormGraph + tokens)
PG_BACKUP_FILE="${BACKUP_DIR}/postgres_${DATE}.sql.gz"
echo "[$(date)] Backing up PostgreSQL..."
docker exec cathedral-postgres pg_dump -U cathedral cathedral | gzip > "$PG_BACKUP_FILE"
echo "[$(date)] PostgreSQL backup saved to $PG_BACKUP_FILE"

# 2. Redis Backup (DLQ + Cache)
REDIS_BACKUP_FILE="${BACKUP_DIR}/redis_${DATE}.rdb"
echo "[$(date)] Backing up Redis..."
# Trigger BGSAVE and wait
docker exec cathedral-redis redis-cli BGSAVE
sleep 5
docker cp cathedral-redis:/data/dump.rdb "$REDIS_BACKUP_FILE"
echo "[$(date)] Redis backup saved to $REDIS_BACKUP_FILE"

# 3. Clean up old backups
echo "[$(date)] Cleaning up backups older than ${RETENTION_DAYS} days..."
find "$BACKUP_DIR" -type f -name "postgres_*.sql.gz" -mtime +${RETENTION_DAYS} -exec rm {} \;
find "$BACKUP_DIR" -type f -name "redis_*.rdb" -mtime +${RETENTION_DAYS} -exec rm {} \;

echo "[$(date)] Backup process completed successfully."
