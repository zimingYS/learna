#!/usr/bin/env bash
# 从最新 origin/main 创建规范命名分支
# 用法: new-branch.sh <type> <name>   例: new-branch.sh feat user-auth
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1

TYPE="${1:-}"; NAME="${2:-}"
case "$TYPE" in
  feat|fix|docs|chore|refactor|test|build|ci) : ;;
  *) echo "用法: new-branch.sh <feat|fix|docs|chore|refactor|test|build|ci> <name>" >&2; exit 2 ;;
esac
[ -n "$NAME" ] || { echo "错误: 缺少分支名" >&2; exit 2; }

git fetch origin main --quiet
git switch -c "$TYPE/$NAME" origin/main
echo "已创建分支 $TYPE/$NAME（基于 origin/main）"
