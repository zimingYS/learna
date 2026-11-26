#!/usr/bin/env bash
# 检查迁移状态（make db-status）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/_common.sh"

need_sqlx
URL="$(db_url)"
echo "==> 迁移状态"
DATABASE_URL="$URL" sqlx migrate info --source "$MIGRATIONS"
