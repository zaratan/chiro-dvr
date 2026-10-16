#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)
tag=${1:?usage: release-guards.sh TAG}
slug=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is not set}
changelog=$repository/CHANGELOG.md

fail() {
  echo "::error::$*" >&2
  exit 1
}

link() {
  awk -v label="[$1]: " 'index($0, label) == 1' "$changelog"
}

previous_section() {
  awk -v heading="## [$1]" '
    found && /^## \[[0-9]/ { sub(/^## \[/, ""); sub(/\].*/, ""); print; exit }
    index($0, heading) == 1 { found = 1 }
  ' "$changelog"
}

version=$(python3 -I -c 'import sys, tomllib; print(tomllib.load(open(sys.argv[1], "rb"))["workspace"]["package"]["version"])' \
  "$repository/Cargo.toml")
[ "$tag" = "v$version" ] || fail "tag $tag but Cargo.toml says $version"
echo "tag $tag matches Cargo.toml"

"$repository/packaging/release-notes.sh" "$version" > /dev/null || fail "CHANGELOG.md has no section [$version]"
echo "CHANGELOG.md has a section [$version]"

previous=$(previous_section "$version")
if [ -n "$previous" ]; then
  wanted="[$version]: https://github.com/$slug/compare/v$previous...v$version"
else
  wanted="[$version]: https://github.com/$slug/releases/tag/v$version"
fi
found=$(link "$version")
[ "$found" = "$wanted" ] || fail "CHANGELOG.md links [$version] as '${found:-nothing}', expected '$wanted'"
echo "CHANGELOG.md links [$version] to ${previous:+v$previous...}v$version"

if grep -q '^## \[Non publiée\]' "$changelog" || [ -n "$(link 'Non publiée')" ]; then
  wanted="[Non publiée]: https://github.com/$slug/compare/v$version...HEAD"
  found=$(link "Non publiée")
  [ "$found" = "$wanted" ] || fail "CHANGELOG.md links [Non publiée] as '${found:-nothing}', expected '$wanted'"
  echo "CHANGELOG.md links [Non publiée] to v$version...HEAD"
fi

commit=$(git -C "$repository" rev-parse HEAD)
git -C "$repository" merge-base --is-ancestor "$commit" origin/main || fail "$commit is not on main"
echo "$commit is on main"

passed=$(gh run list --repo "$slug" --workflow ci.yml --branch main \
  --commit "$commit" --status success --json databaseId --jq length)
[ "$passed" != 0 ] || fail "CI has not passed on main for $commit yet: rerun once it has"
echo "CI passed on main for $commit"
