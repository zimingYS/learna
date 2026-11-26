#!/usr/bin/env bash
# 版本发布：校验 → 检查 → 变更日志 → 版本号三处同步 → 提交打标 → 推送
# 用法: release.sh <X.Y.Z> [--skip-checks]
# 推送 tag 后由 .github/workflows/release.yml 自动创建 GitHub Release
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1

VERSION="${1:-}"
SKIP=0
[ "${2:-}" = "--skip-checks" ] && SKIP=1

case "$VERSION" in
  [0-9]*.[0-9]*.[0-9]*) : ;;
  *) echo "用法: release.sh <X.Y.Z> [--skip-checks]" >&2; exit 2 ;;
esac
echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$' || { echo "错误: 版本号须为 X.Y.Z" >&2; exit 2; }

[ -z "$(git status --porcelain)" ] || { echo "错误: 工作区不干净，请先提交" >&2; exit 1; }
[ "$(git branch --show-current)" = "main" ] || { echo "错误: 请在 main 上发布" >&2; exit 1; }
git rev-parse "v$VERSION" >/dev/null 2>&1 && { echo "错误: tag v$VERSION 已存在" >&2; exit 1; }

if [ "$SKIP" -eq 0 ]; then
  bash scripts/check/check.sh || exit 1
  bash scripts/test/test.sh || exit 1
fi

bash scripts/release/changelog.sh "$VERSION"

# 三处版本号同步
sed -i -E "0,/^version = \"[0-9.]+\"/s//version = \"$VERSION\"/" backend/Cargo.toml
sed -i -E "0,/\"version\": \"[0-9.]+\"/s//\"version\": \"$VERSION\"/" frontend/package.json
sed -i -E "0,/^version = \"[0-9.]+\"/s//version = \"$VERSION\"/" ai-service/pyproject.toml

git add CHANGELOG.md backend/Cargo.toml frontend/package.json ai-service/pyproject.toml
git commit -m "chore(release): v$VERSION"
git tag "v$VERSION"

echo "已提交并打标 v$VERSION"
echo "推送命令: git push origin main --follow-tags"
echo "推送后 Release 工作流将自动创建 GitHub Release"
