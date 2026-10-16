#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)

fail() {
  echo "package: $*" >&2
  exit 1
}

host=$(rustc -vV | tr -d '\r' | sed -n 's/^host: //p')
case $host in
  aarch64-apple-darwin) name=darwin-arm64 extension=tar.gz binary=batdetect ;;
  x86_64-unknown-linux-gnu) name=linux-x86_64 extension=tar.gz binary=batdetect ;;
  x86_64-pc-windows-msvc) name=windows-x86_64 extension=zip binary=batdetect.exe ;;
  *) fail "no archive for $host" ;;
esac
archive=$repository/dist/batdetect-$name.$extension
declared=$(tr -d '\r' < "$repository/Cargo.toml" | sed -n '/^\[workspace\.package\]$/,/^\[/s/^version = "\(.*\)"$/\1/p')
[ -n "$declared" ] || fail "no version in [workspace.package] of Cargo.toml"
export BATDETECT_VERSION=${BATDETECT_VERSION:-$declared}

case ${1:-} in
  --archive)
    echo "$archive"
    exit 0
    ;;
  --version)
    echo "$BATDETECT_VERSION"
    exit 0
    ;;
  "") ;;
  *) fail "usage: $0 [--archive | --version]" ;;
esac

cd "$repository"
cargo build --locked --release -p batdetect --bin batdetect

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
entries=("$binary" LICENSE NOTICE)
cp "target/release/$binary" "$stage/"
cp LICENSE packaging/NOTICE "$stage/"
if [ -d packaging/licenses ]; then
  cp -R packaging/licenses "$stage/licenses"
  entries+=(licenses)
fi

mkdir -p dist
rm -f "$archive"
if [ "$extension" = zip ]; then
  (cd "$stage" && 7z a -tzip -bso0 -bsp0 "$(cygpath -w "$archive")" "${entries[@]}")
else
  COPYFILE_DISABLE=1 tar --no-xattrs -czf "$archive" -C "$stage" "${entries[@]}"
fi
echo "$archive"
