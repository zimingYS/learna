#!/usr/bin/env bash
# 提 PR 前检查：分支命名 + 提交信息规范 + 质量检查 + 测试
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1

FAILS=0
BRANCH="$(git branch --show-current)"

echo "== PR 前检查 =="
case "$BRANCH" in
  main) echo "[FAIL] 不应在 main 上直接开发，请先创建功能分支"; FAILS=$((FAILS+1)) ;;
  feat/*|fix/*|docs/*|chore/*|refactor/*|test/*|build/*|ci/*) echo "[PASS] 分支命名: $BRANCH" ;;
  *) echo "[WARN] 分支命名: $BRANCH（不在推荐前缀内）" ;;
esac

BAD_MSGS=$(git log origin/main..HEAD --pretty=%s 2>/dev/null | grep -vE '^(feat|fix|docs|chore|refactor|test|build|ci)(\(.+\))?!?: .+|^Merge |^Revert ' || true)
if [ -n "$BAD_MSGS" ]; then
  echo "[FAIL] 提交信息不符合 Conventional Commits:"
  echo "$BAD_MSGS" | sed 's/^/       /'
  FAILS=$((FAILS+1))
else
  echo "[PASS] 提交信息规范"
fi

bash scripts/check/check.sh && echo "[PASS] 质量检查" || { echo "[FAIL] 质量检查"; FAILS=$((FAILS+1)); }
bash scripts/test/test.sh   && echo "[PASS] 测试" || { echo "[FAIL] 测试"; FAILS=$((FAILS+1)); }

echo
if [ "$FAILS" -gt 0 ]; then
  echo "PR 前检查未通过（$FAILS 项）。"
  exit 1
fi
echo "PR 前检查全部通过，可以发起 Pull Request。"
