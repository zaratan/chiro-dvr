#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)
versions=$repository/packaging/ffmpeg-versions
out=${1:?usage: sources.sh DIRECTORY}

fail() {
  echo "sources: $*" >&2
  exit 1
}

sha256() {
  if command -v sha256sum > /dev/null; then
    sha256sum "$1" | cut -d ' ' -f 1
  else
    shasum -a 256 "$1" | cut -d ' ' -f 1
  fi
}

pinned() {
  local line
  line=$(grep -E "^$1=" "$versions") || fail "$1 missing from $versions"
  echo "${line#*=}"
}

fetch() {
  local name=$1 file=$2 url
  url=$(pinned "${name}_URL")
  curl -sSfL --retry 3 --retry-all-errors --connect-timeout 20 -o "$out/$file" "$url" || fail "could not download $url"
  [ "$(sha256 "$out/$file")" = "$(pinned "${name}_SHA256")" ] || fail "$file does not match its pinned sha256"
  echo "$out/$file"
}

mkdir -p "$out"
fetch FFMPEG "ffmpeg-$(pinned FFMPEG_VERSION).tar.xz"
fetch OPENH264 "openh264-$(pinned OPENH264_VERSION).tar.gz"
