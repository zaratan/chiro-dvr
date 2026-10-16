#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)
tag=${1:?usage: release-guards.sh TAG}

fail() {
  echo "::error::$*" >&2
  exit 1
}

version=$(python3 -I -c 'import sys, tomllib; print(tomllib.load(open(sys.argv[1], "rb"))["workspace"]["package"]["version"])' \
  "$repository/Cargo.toml")
[ "$tag" = "v$version" ] || fail "tag $tag but Cargo.toml says $version"
echo "tag $tag matches Cargo.toml"

"$repository/packaging/release-notes.sh" "$version" > /dev/null || fail "CHANGELOG.md has no section [$version]"
echo "CHANGELOG.md has a section [$version]"

commit=$(git -C "$repository" rev-parse HEAD)
git -C "$repository" merge-base --is-ancestor "$commit" origin/main || fail "$commit is not on main"
echo "$commit is on main"

passed=$(gh run list --repo "${GITHUB_REPOSITORY:?}" --workflow ci.yml --branch main \
  --commit "$commit" --status success --json databaseId --jq length)
[ "$passed" != 0 ] || fail "CI has not passed on main for $commit yet: rerun once it has"
echo "CI passed on main for $commit"
