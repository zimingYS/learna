#!/usr/bin/env bash
# 环境体检：逐项输出「检查项 | 预期 | 结果」，任一 FAIL 退出码 1
# 用法: doctor.sh
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FAILS=0
WARNS=0
ROWS=()

row() {
  ROWS+=("$1|$2|$3")
  case "$3" in
    FAIL*) FAILS=$((FAILS+1)) ;;
    WARN*) WARNS=$((WARNS+1)) ;;
  esac
  return 0
}

probe_port() { # 0=端口有进程监听 1=空闲
  (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null && { exec 3>&- 3<&-; return 0; }
  return 1
}

tool_ver() { command -v "$1" >/dev/null 2>&1 && "$1" $2 2>/dev/null | head -1 || echo ""; }

# 版本大版本号 >= 门槛
ver_ok() { local maj; maj="$(echo "$1" | grep -oE '[0-9]+' | head -1)"; [ -n "$maj" ] && [ "$maj" -ge "$2" ]; }

echo "== Learna 环境体检 =="
echo

# 1-2 Rust / Cargo
V="$(tool_ver rustc --version)"
[ -n "$V" ] && row "Rust" "版本符合要求" "PASS|$V" || row "Rust" "版本符合要求" "FAIL|未安装"
V="$(tool_ver cargo --version)"
[ -n "$V" ] && row "Cargo" "正常" "PASS|$V" || row "Cargo" "正常" "FAIL|未安装"

# 3-4 Node.js / npm
V="$(tool_ver node --version)"
if [ -z "$V" ]; then row "Node.js" "版本符合要求" "FAIL|未安装"
elif ver_ok "$V" 20; then row "Node.js" "版本符合要求" "PASS|$V"
else row "Node.js" "版本符合要求" "FAIL|版本过低: $V（需 20+）"; fi
V="$(tool_ver npm --version)"
[ -n "$V" ] && row "npm" "正常" "PASS|v$V" || row "npm" "正常" "FAIL|未安装"

# 5-6 Python / uv
V="$(tool_ver python --version)"
if [ -z "$V" ]; then row "Python" "版本符合要求" "FAIL|未安装"
elif ver_ok "$V" 3 && [ "$(echo "$V" | grep -oE '[0-9]+' | head -1)" -eq 3 ] && \
     [ "$(echo "$V" | grep -oE '[0-9]+' | sed -n 2p)" -ge 11 ] 2>/dev/null; then
  row "Python" "版本符合要求" "PASS|$V"
else row "Python" "版本符合要求" "FAIL|版本过低: $V（需 3.11+）"; fi
V="$(tool_ver uv --version)"
[ -n "$V" ] && row "uv" "正常" "PASS|$V" || row "uv" "正常" "FAIL|未安装"

# 7 Docker
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  row "Docker" "Docker Engine 可用" "PASS|$(docker --version | head -1)"
else
  row "Docker" "Docker Engine 可用" "FAIL|Engine 不可用或未安装"
fi

# 8 PostgreSQL
if command -v pg_isready >/dev/null 2>&1 && pg_isready -h localhost -p 5432 -q 2>/dev/null; then
  row "PostgreSQL" "数据库可连接" "PASS|pg_isready 就绪"
elif command -v docker >/dev/null 2>&1 && docker exec learna-postgres-dev pg_isready -U learna -q 2>/dev/null; then
  row "PostgreSQL" "数据库可连接" "PASS|dev 容器就绪"
elif probe_port 5432; then
  row "PostgreSQL" "数据库可连接" "WARN|5432 有监听但未能确认可连接"
else
  row "PostgreSQL" "数据库可连接" "FAIL|无法连接（dev 库未启动？）"
fi

# 9 环境变量
MISSING=""
if [ ! -f "$ROOT/.env" ]; then
  row "环境变量" "必需变量存在" "FAIL|.env 不存在（bash scripts/setup.sh 生成）"
else
  for var in APP_ENV DATABASE_URL JWT_SECRET; do
    grep -qE "^${var}=.+" "$ROOT/.env" || MISSING="$MISSING $var"
  done
  if [ -z "$MISSING" ]; then
    row "环境变量" "必需变量存在" "PASS|APP_ENV/DATABASE_URL/JWT_SECRET 就绪"
  else
    row "环境变量" "必需变量存在" "FAIL|缺失:$MISSING"
  fi
fi

# 10 端口冲突（5173/8080/8001/5432）
CONFLICT=""
for p in 5173 8080 8001; do
  if probe_port "$p"; then
    # 8080 能应答 /api/healthz 视为 Learna 自身，不算冲突
    if [ "$p" = "8080" ] && curl -s --max-time 2 "http://localhost:8080/api/healthz" 2>/dev/null | grep -q '"200"'; then
      :
    else
      CONFLICT="$CONFLICT $p"
    fi
  fi
done
if [ -z "$CONFLICT" ]; then
  row "端口" "无关键服务冲突" "PASS|5173/8080/8001 可用"
else
  row "端口" "无关键服务冲突" "WARN|被占用:$CONFLICT"
fi

# 汇总
printf '%-12s %-22s %-6s %s\n' "检查项" "预期" "结果" "实际"
printf '%-12s %-22s %-6s %s\n' "------" "----" "----" "----"
for r in "${ROWS[@]}"; do
  IFS='|' read -r item expect status detail <<<"$r"
  printf '%-12s %-22s %-6s %s\n' "$item" "$expect" "$status" "$detail"
done
echo
[ "$WARNS" -gt 0 ] && echo "警告 $WARNS 项（不阻塞）。"
if [ "$FAILS" -gt 0 ]; then
  echo "失败 $FAILS 项，请修复后重试。"
  exit 1
fi
echo "环境健康。"
