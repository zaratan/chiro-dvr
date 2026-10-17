#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)
versions=$repository/packaging/sde-versions

fail() {
  echo "sde: $*" >&2
  exit 1
}

sha256() {
  if command -v sha256sum > /dev/null; then
    sha256sum | cut -d ' ' -f 1
  else
    shasum -a 256 | cut -d ' ' -f 1
  fi
}

pinned() {
  local line
  line=$(grep -E "^$1=" "$versions") || fail "$1 missing from $versions"
  echo "${line#*=}"
}

cache_home=${XDG_CACHE_HOME:-$HOME/.cache}
case $(uname -s) in
  Linux) system=LINUX executable=sde64 ;;
  *_NT*) system=WINDOWS executable=sde.exe cache_home=$(cygpath -u "$cache_home") ;;
  *) fail "Intel SDE runs only on Linux and Windows" ;;
esac
url=$(pinned "SDE_${system}_URL")
sum=$(pinned "SDE_${system}_SHA256")
cache=$cache_home/batdetect/sde
kit=$cache/$(basename "$url" .tar.xz)

if [ ! -x "$kit/$executable" ]; then
  mkdir -p "$cache"
  work=$(mktemp -d "$cache/.fetch.XXXXXX")
  trap 'rm -rf "$work"' EXIT
  archive=$work/$(basename "$url")
  curl -sSfL --retry 3 --retry-all-errors --connect-timeout 20 -o "$archive" "$url"
  [ "$(sha256 < "$archive")" = "$sum" ] || fail "$archive does not match its pinned sha256"
  tar -xf "$archive" -C "$work"
  mv -T "$work/$(basename "$kit")" "$kit" || [ -x "$kit/$executable" ] || fail "could not move the kit to $kit"
fi
echo "$kit/$executable"
