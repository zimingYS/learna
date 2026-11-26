#!/usr/bin/env bash
# 统一质量检查：格式 + 静态分析 + 类型检查（不含测试，测试见 scripts/test/test.sh）
# 用法: check.sh [all|backend|ai-service|frontend] [--fast]
#   --fast: 跳过重型检查（clippy / vue-tsc），供 pre-commit 使用
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET="all"
FAST=0
for arg in "$@"; do
  case "$arg" in
    --fast) FAST=1 ;;
    all|backend|ai-service|frontend) TARGET="$arg" ;;
    *) echo "用法: check.sh [all|backend|ai-service|frontend] [--fast]" >&2; exit 2 ;;
  esac
done

FAILED=()
run() { # run <label> <cmd...>
  local label="$1"; shift
  printf '[check] %-28s ' "$label"
  if "$@" >/dev/null 2>&1; then
    echo "OK"
  else
    echo "FAIL"
    FAILED+=("$label")
    "$@" 2>&1 | tail -15
  fi
}

check_backend() {
  cd "$ROOT/backend" || exit 1
  run "backend: cargo fmt" cargo fmt --check
  if [ "$FAST" -eq 0 ]; then
    run "backend: clippy" cargo clippy --locked -- -D warnings
  fi
}

check_ai_service() {
  cd "$ROOT/ai-service" || exit 1
  run "ai-service: ruff format" uv run ruff format --check app tests
  run "ai-service: ruff check" uv run ruff check app tests
}

check_frontend() {
  cd "$ROOT/frontend" || exit 1
  run "frontend: eslint" npm run lint --silent
  if [ "$FAST" -eq 0 ]; then
    run "frontend: vue-tsc" npm run type-check --silent
  fi
}

case "$TARGET" in
  all)         check_backend; check_ai_service; check_frontend ;;
  backend)     check_backend ;;
  ai-service)  check_ai_service ;;
  frontend)    check_frontend ;;
  *) echo "用法: check.sh [all|backend|ai-service|frontend] [--fast]" >&2; exit 2 ;;
esac

echo "----------------------------------------"
if [ ${#FAILED[@]} -eq 0 ]; then
  echo "质量检查全部通过"
else
  echo "检查失败 (${#FAILED[@]}):"
  printf '  - %s\n' "${FAILED[@]}"
  exit 1
fi
