#!/usr/bin/env bash
# 重建本地开发数据库（make db-reset）—— 丢弃全部数据并重放迁移
# 安全线：非交互环境拒绝；APP_ENV=prod/production 拒绝；非 localhost 目标拒绝。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/_common.sh"

ENV="$(app_env)"
URL="$(db_url)"

case "$ENV" in
  prod|production)
    echo "拒绝执行：APP_ENV=$ENV。db-reset 禁止在生产环境执行。" >&2
    exit 1 ;;
esac

case "$URL" in
  *localhost*|*127.0.0.1*) : ;;
  *)
    echo "拒绝执行：DATABASE_URL 目标不是 localhost（$URL）。" >&2
    exit 1 ;;
esac

if [ ! -t 0 ]; then
  echo "拒绝执行：db-reset 需要交互确认，请在终端中运行。" >&2
  exit 1
fi

echo "!!! 危险操作：将销毁并重建开发数据库（$URL）"
echo "!!! 全部本地数据将丢失，生产环境禁止执行。"
printf "输入 yes 确认继续: "
read -r ans
if [ "$ans" != "yes" ]; then
  echo "已取消。"
  exit 0
fi

echo "==> 重建数据库并重放迁移"
need_sqlx
DATABASE_URL="$URL" sqlx database reset --force --source "$MIGRATIONS"
