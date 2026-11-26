#!/usr/bin/env bash
# 公共逻辑：定位根目录、读取 DATABASE_URL、检查 sqlx-cli
# 被 scripts/db/*.sh source
set -uo pipefail

DB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MIGRATIONS="$DB_ROOT/backend/migrations"

db_url() {
  if [ -f "$DB_ROOT/.env" ]; then
    grep -E "^DATABASE_URL=.+" "$DB_ROOT/.env" | head -1 | cut -d= -f2- | tr -d '"' | tr -d "'"
  else
    echo "postgres://learna:devpass@localhost:5432/learna"
  fi
}

app_env() {
  if [ -f "$DB_ROOT/.env" ]; then
    grep -E "^APP_ENV=.+" "$DB_ROOT/.env" | head -1 | cut -d= -f2- | tr -d '"' | tr -d "'"
  else
    echo "dev"
  fi
}

need_sqlx() {
  if ! command -v sqlx >/dev/null 2>&1; then
    echo "错误: 未安装 sqlx-cli。安装：" >&2
    echo "  cargo install sqlx-cli --no-default-features --features postgres,rustls" >&2
    exit 1
  fi
}
