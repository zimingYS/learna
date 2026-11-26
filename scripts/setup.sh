#!/usr/bin/env bash
# 开发环境初始化
# 用法: setup.sh [dev|docker] [--full]
#   dev    （默认）集成开发环境模式：工具链检查 + 依赖安装
#   docker Docker 模式：Docker 检查 + 镜像构建与容器启动（部署自动化）
#   --full Docker 模式下启动全栈（deploy/compose.yaml），默认只起开发数据库
# 已存在的 .env 不会被覆盖。
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="dev"
FULL=0
for arg in "$@"; do
  case "$arg" in
    dev|docker) MODE="$arg" ;;
    --full) FULL=1 ;;
  esac
done

RESULTS=()
note() { RESULTS+=("$1|$2|$3"); printf '%-10s %-36s %s\n' "$2" "$1" "$3"; }

ok()   { note "$1" "OK" "${2:-}"; }
warn() { note "$1" "WARN" "${2:-}"; }
fail() { note "$1" "FAIL" "${2:-}"; FAILS=$((FAILS+1)); }
FAILS=0

check_tool() { # check_tool <label> <cmd> <version-args> <最低大版本|空>
  local label="$1" cmd="$2" varg="$3" minmaj="${4:-}"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    fail "$label" "未安装"
    return
  fi
  local ver maj
  ver="$("$cmd" $varg 2>/dev/null | head -1)"
  if [ -n "$minmaj" ]; then
    maj="$(echo "$ver" | grep -oE '[0-9]+' | head -1)"
    if [ -n "$maj" ] && [ "$maj" -lt "$minmaj" ]; then
      fail "$label" "版本过低: $ver（需 >= $minmaj）"
      return
    fi
  fi
  ok "$label" "$ver"
}

echo "== Learna 开发环境初始化（模式: $MODE）=="
echo

echo "-- 工具链检查 --"
check_tool "Rust"    rustc "--version" ""
check_tool "Cargo"   cargo "--version" ""
check_tool "Node.js" node "--version" 20
check_tool "npm"     npm  "--version" ""
check_tool "Python"  python "--version" 3
check_tool "uv"      uv   "--version" ""

if [ "$MODE" = "docker" ]; then
  if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    ok "Docker" "$(docker --version | head -1)"
  else
    fail "Docker" "Docker Engine 不可用"
  fi
  docker compose version >/dev/null 2>&1 && ok "Compose" "$(docker compose version --short)" || fail "Compose" "docker compose 不可用"
fi

echo
echo "-- 环境变量 --"
if [ -f "$ROOT/.env" ]; then
  ok ".env" "已存在，保留不覆盖"
else
  cp "$ROOT/.env.example" "$ROOT/.env"
  warn ".env" "已从 .env.example 创建，请填写真实值"
fi

echo
echo "-- 依赖安装 --"
if [ "$MODE" = "dev" ]; then
  (cd "$ROOT/frontend" && npm install --no-fund --no-audit) >/dev/null 2>&1 \
    && ok "前端依赖" "npm install" || fail "前端依赖" "npm install 失败"
  (cd "$ROOT/ai-service" && uv sync --quiet) >/dev/null 2>&1 \
    && ok "Python 依赖" "uv sync" || fail "Python 依赖" "uv sync 失败"
  (cd "$ROOT/backend" && cargo fetch --locked) >/dev/null 2>&1 \
    && ok "Rust 依赖" "cargo fetch" || fail "Rust 依赖" "cargo fetch 失败"
else
  if [ "$FAILS" -eq 0 ]; then
    if [ "$FULL" -eq 1 ]; then
      # --env-file 显式指向根 .env：compose 项目目录是 deploy/，默认读不到根 .env
      (cd "$ROOT/deploy" && docker compose --env-file "$ROOT/.env" up -d --build) >/dev/null 2>&1 \
        && ok "部署自动化" "全栈已启动（compose.yaml）" || fail "部署自动化" "compose up 失败"
    else
      (cd "$ROOT/deploy" && docker compose -f compose.dev.yaml up -d) >/dev/null 2>&1 \
        && ok "部署自动化" "开发数据库已启动（compose.dev.yaml）" \
        || fail "部署自动化" "compose.dev up 失败"
    fi
  else
    warn "部署自动化" "工具链检查未通过，跳过部署"
  fi
fi

echo
echo "== 环境初始化结果 =="
printf '%-10s %-36s %s\n' "状态" "项目" "说明"
printf '%-10s %-36s %s\n' "----" "----" "----"
for r in "${RESULTS[@]}"; do
  IFS='|' read -r name status extra <<<"$r"
  printf '%-10s %-36s %s\n' "$status" "$name" "$extra"
done
if [ "$FAILS" -eq 0 ]; then
  echo "初始化完成。下一步参考 docs/开发指南/本地开发环境.md 第 4 节启动服务。"
else
  echo "存在 $FAILS 项失败，请先修复后重试。"
  exit 1
fi
