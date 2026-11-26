#!/usr/bin/env bash
# 同步本地 main 与 origin/main（快进更新；有本地提交时只提示不强推）
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1

git fetch origin --quiet
BRANCH="$(git branch --show-current)"

if [ "$BRANCH" = "main" ]; then
  git merge --ff-only origin/main && echo "main 已更新到 $(git rev-parse --short HEAD)"
else
  AHEAD=$(git rev-list --count main..origin/main 2>/dev/null || echo "?")
  BEHIND=$(git rev-list --count origin/main..main 2>/dev/null || echo "?")
  echo "当前分支: $BRANCH"
  echo "origin/main 领先本地 main $AHEAD 个提交，本地 main 领先 origin/main $BEHIND 个提交"
  echo "提示: 在 main 上执行本脚本可快进更新；或用 git rebase origin/main 变基当前分支"
fi
