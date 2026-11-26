#!/usr/bin/env bash
# 执行数据库迁移（make db-migrate）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/_common.sh"

need_sqlx
URL="$(db_url)"
echo "==> 执行迁移（$MIGRATIONS）"
DATABASE_URL="$URL" sqlx migrate run --source "$MIGRATIONS"
