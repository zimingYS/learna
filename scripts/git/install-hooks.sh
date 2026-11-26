#!/usr/bin/env bash
# 安装仓库自带 Git 钩子（core.hooksPath -> .githooks）
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1

git config core.hooksPath .githooks
chmod +x .githooks/* 2>/dev/null || true
echo "已启用 .githooks（pre-commit / commit-msg / pre-push）"
echo "临时跳过钩子：LEARNA_SKIP_HOOKS=1 git commit ..."
