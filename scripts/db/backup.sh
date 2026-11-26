#!/usr/bin/env bash
# 备份本地数据库（make db-backup）→ backups/learna_<时间戳>.sql
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/_common.sh"

OUT_DIR="$DB_ROOT/backups"
mkdir -p "$OUT_DIR"
OUT="$OUT_DIR/learna_$(date +%Y%m%d_%H%M%S).sql"

if command -v docker >/dev/null 2>&1 && docker exec learna-postgres-dev pg_isready -U learna -q 2>/dev/null; then
  echo "==> 通过 dev 容器备份"
  docker exec learna-postgres-dev pg_dump -U learna learna > "$OUT"
elif command -v pg_dump >/dev/null 2>&1; then
  echo "==> 通过本地 pg_dump 备份"
  pg_dump "$(db_url)" > "$OUT"
else
  echo "错误: 无可用的备份通道（docker dev 容器或本地 pg_dump 均不可用）。" >&2
  exit 1
fi

echo "备份完成: $OUT（$(wc -c <"$OUT") 字节）"
