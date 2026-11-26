#!/usr/bin/env bash
# 统一测试：三栈测试执行（质量检查见 scripts/check/check.sh）
# 用法: test.sh [all|backend|ai-service|frontend]
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET="${1:-all}"

FAILED=()
run() { # run <label> <cmd...>
  local label="$1"; shift
  printf '[test] %-30s ' "$label"
  if "$@" >/dev/null 2>&1; then
    echo "OK"
  else
    echo "FAIL"
    FAILED+=("$label")
    "$@" 2>&1 | tail -15
  fi
}

test_backend() {
  cd "$ROOT/backend" || exit 1
  # 集成测试需要 DATABASE_URL，未设置时从根 .env 读取（与 db 脚本同口径）
  if [ -z "${DATABASE_URL:-}" ] && [ -f "$ROOT/.env" ]; then
    DATABASE_URL="$(grep -E "^DATABASE_URL=.+" "$ROOT/.env" | head -1 | cut -d= -f2- | tr -d '"' | tr -d "'")"
    export DATABASE_URL
  fi
  run "backend: cargo test" cargo test --locked
}

test_ai_service() {
  cd "$ROOT/ai-service" || exit 1
  run "ai-service: pytest" uv run pytest
}

test_frontend() {
  cd "$ROOT/frontend" || exit 1
  run "frontend: vitest" npm test --silent
}

case "$TARGET" in
  all)         test_backend; test_ai_service; test_frontend ;;
  backend)     test_backend ;;
  ai-service)  test_ai_service ;;
  frontend)    test_frontend ;;
  *) echo "用法: test.sh [all|backend|ai-service|frontend]" >&2; exit 2 ;;
esac

echo "----------------------------------------"
if [ ${#FAILED[@]} -eq 0 ]; then
  echo "全部测试通过"
else
  echo "测试失败 (${#FAILED[@]}):"
  printf '  - %s\n' "${FAILED[@]}"
  exit 1
fi
