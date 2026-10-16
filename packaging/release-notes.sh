#!/usr/bin/env bash
set -euo pipefail
version="${1:?usage: release-notes.sh X.Y.Z}"
notes=$(awk -v v="$version" '
  $0 ~ "^## \\[" v "\\]" { found = 1; next }
  found && /^## \[/ { exit }
  found && /^\[[^]]+\]: / { exit }
  found { print }
' "$(dirname "$0")/../CHANGELOG.md" | sed -e '/./,$!d')
test -n "$notes" || { echo "CHANGELOG.md has no section [$version]" >&2; exit 1; }
printf '%s\n' "$notes"
