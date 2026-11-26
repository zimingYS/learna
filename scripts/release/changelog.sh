#!/usr/bin/env bash
# 从 Conventional Commits 生成变更日志段落并入 CHANGELOG.md
# 用法: changelog.sh [X.Y.Z]   不带版本写入 [Unreleased]，带版本写入 [vX.Y.Z]
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 1

VERSION="${1:-Unreleased}"
if [ "$VERSION" != "Unreleased" ]; then
  VERSION="v${VERSION#v}"
fi

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
RANGE="$LAST_TAG..HEAD"
[ -z "$LAST_TAG" ] && RANGE="HEAD"

# 已存在该版本段落则跳过，避免重复
if [ -f CHANGELOG.md ] && grep -q "^## \[$VERSION\]" CHANGELOG.md; then
  echo "CHANGELOG.md 已存在 [$VERSION] 段落，跳过生成"
  exit 0
fi

SUBJECTS=$(git log "$RANGE" --pretty=%s | grep -vE '^(Merge |chore\(release\))' || true)
if [ -z "$SUBJECTS" ]; then
  echo "自 ${LAST_TAG:-仓库初始} 以来无新提交，不生成段落"
  exit 0
fi

section() { # section <标题> <类型前缀...> —— 输出到 $BODY_FILE
  local title="$1"; shift
  local items
  items=$(echo "$SUBJECTS" | grep -E "^($1)(\(.+\))?!?: " | sed -E 's/^[a-z]+(\([^)]*\))?!?: /- /' || true)
  if [ -n "$items" ]; then
    echo "### $title" >>"$BODY_FILE"
    echo >>"$BODY_FILE"
    echo "$items" >>"$BODY_FILE"
    echo >>"$BODY_FILE"
  fi
}

BODY_FILE=$(mktemp)
section 新增 feat
section 修复 fix
section 变更 refactor perf
section 工程与文档 docs chore ci build test

HEADER="## [$VERSION]"
[ "$VERSION" != "Unreleased" ] && HEADER="$HEADER - $(date +%Y-%m-%d)"

ENTRY_FILE=$(mktemp)
{
  echo "$HEADER"
  echo
  cat "$BODY_FILE"
} >"$ENTRY_FILE"
rm -f "$BODY_FILE"

if [ ! -f CHANGELOG.md ]; then
  { echo '# Changelog'; echo; cat "$ENTRY_FILE"; } > CHANGELOG.md
else
  TMP=$(mktemp)
  ENTRY_FILE="$ENTRY_FILE" awk '
    NR==1 {print; next}
    /^## \[/ && !done {
      ef = ENVIRON["ENTRY_FILE"]
      while ((getline line < ef) > 0) print line
      close(ef)
      print ""
      done=1
    }
    {print}
    END {
      if (!done) {
        print ""
        ef = ENVIRON["ENTRY_FILE"]
        while ((getline line < ef) > 0) print line
        close(ef)
      }
    }
  ' CHANGELOG.md > "$TMP" && mv "$TMP" CHANGELOG.md
fi
rm -f "$ENTRY_FILE"

echo "已生成 CHANGELOG.md [$VERSION] 段落（基于 ${LAST_TAG:-仓库初始}..HEAD）"
