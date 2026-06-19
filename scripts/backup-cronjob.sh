#!/bin/bash
# scripts/backup-cronjob.sh
# Script de backup automatizado para PostgreSQL e Redis na Cathedral ARKHE
# Pode ser configurado em um cronjob (ex: 0 2 * * * /caminho/para/backup-cronjob.sh)
#
# Selo: CATHEDRAL-ARKHE-8000-BACKUP-CRONJOB-v1.0.0-2026-06-19

set -e

# Configurações
BACKUP_DIR="/mnt/persist/backups"
TIMESTAMP=$(date +'%Y%m%d_%H%M%S')
RETENTION_DAYS=30

mkdir -p "$BACKUP_DIR/postgres"
mkdir -p "$BACKUP_DIR/redis"

# Log
LOG_FILE="/var/log/cathedral-backup.log"
exec >> "$LOG_FILE" 2>&1

echo "========================================"
echo "Iniciando Backup Automatizado: $TIMESTAMP"
echo "========================================"

# 1. Backup do PostgreSQL (WormGraph)
echo "➜ Fazendo backup do PostgreSQL..."
POSTGRES_CONTAINER="cathedral-postgres"
POSTGRES_USER="cathedral"

if docker ps | grep -q "$POSTGRES_CONTAINER"; then
    PG_DUMP_FILE="$BACKUP_DIR/postgres/pg_backup_$TIMESTAMP.sql.gz"
    docker exec "$POSTGRES_CONTAINER" pg_dump -U "$POSTGRES_USER" -Fc cathedral > "$PG_DUMP_FILE"
    echo "✅ Backup PostgreSQL concluído: $PG_DUMP_FILE"
else
    echo "❌ Erro: Container PostgreSQL não está rodando."
fi

# 2. Backup do Redis (DLQ/Cache)
echo "➜ Fazendo backup do Redis..."
REDIS_CONTAINER="cathedral-redis"

if docker ps | grep -q "$REDIS_CONTAINER"; then
    REDIS_DUMP_FILE="$BACKUP_DIR/redis/redis_backup_$TIMESTAMP.rdb"
    # Salva o banco em disco
    docker exec "$REDIS_CONTAINER" redis-cli SAVE > /dev/null
    # Copia o dump
    docker cp "$REDIS_CONTAINER":/data/dump.rdb "$REDIS_DUMP_FILE"
    echo "✅ Backup Redis concluído: $REDIS_DUMP_FILE"
else
    echo "❌ Erro: Container Redis não está rodando."
fi

# 3. Limpeza de backups antigos
echo "➜ Removendo backups mais antigos que $RETENTION_DAYS dias..."
find "$BACKUP_DIR/postgres" -type f -name "*.sql.gz" -mtime +$RETENTION_DAYS -exec rm -f {} \;
find "$BACKUP_DIR/redis" -type f -name "*.rdb" -mtime +$RETENTION_DAYS -exec rm -f {} \;

echo "✅ Limpeza concluída."
echo "========================================"
echo "Backup Finalizado!"
echo "========================================"
