#!/usr/bin/env bash
set -euo pipefail

script=$(cd "$(dirname "$0")" && pwd -P)/$(basename "$0")
repository=$(cd "$(dirname "$0")/.." && pwd -P)
versions=$repository/packaging/ffmpeg-versions
link=$repository/.ffmpeg
macos_min=14.0

fail() {
  echo "ffmpeg-unix: $*" >&2
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

fingerprint() {
  { cat "$script"; grep -E '^(FFMPEG|OPENH264|NASM)_' "$versions"; } | sha256 | cut -c 1-16
}

os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
case $os-$arch in
  linux-x86_64) cpu=-march=x86-64-v2 ;;
  darwin-arm64) cpu=-mcpu=apple-m1 ;;
  *) fail "no pinned CPU target for $os-$arch" ;;
esac
cache=${XDG_CACHE_HOME:-$HOME/.cache}/batdetect/ffmpeg
target=$cache/$(fingerprint)-$os-$arch

complete() {
  [ -f "$1/lib/pkgconfig/libavcodec.pc" ]
}

fetch() {
  local name=$1 work=$2 url sum archive
  url=$(pinned "${name}_URL")
  sum=$(pinned "${name}_SHA256")
  archive=$work/downloads/$(basename "$url")
  mkdir -p "$work/downloads" "$work/src/$name"
  curl -sSfL --retry 3 --retry-all-errors --connect-timeout 20 -o "$archive" "$url"
  [ "$(sha256 < "$archive")" = "$sum" ] || fail "$archive does not match its pinned sha256"
  tar -xf "$archive" -C "$work/src/$name" --strip-components 1
}

build_nasm() {
  local work=$1 jobs=$2
  fetch NASM "$work"
  (cd "$work/src/NASM" && ./configure --prefix="$work/tools" > /dev/null && make -j "$jobs" > /dev/null && make install > /dev/null)
  PATH=$work/tools/bin:$PATH
}

build_openh264() {
  local work=$1 prefix=$2 jobs=$3
  local flags=()
  fetch OPENH264 "$work"
  if [ "$os" = darwin ]; then
    flags=(STATIC_LDFLAGS=-lc++)
  fi
  make -C "$work/src/OPENH264" -j "$jobs" PREFIX="$prefix" CFLAGS_OPT="-O3 $cpu" "${flags[@]+"${flags[@]}"}" install-static
}

build_ffmpeg() {
  local work=$1 prefix=$2 jobs=$3
  local platform=()
  fetch FFMPEG "$work"
  if [ "$os" = darwin ]; then
    platform=(
      --enable-videotoolbox
      "--enable-hwaccel=h264_videotoolbox,hevc_videotoolbox"
      "--enable-encoder=libopenh264,h264_videotoolbox"
    )
  else
    platform=(--enable-encoder=libopenh264)
  fi
  (
    cd "$work/src/FFMPEG"
    PKG_CONFIG_LIBDIR=$prefix/lib/pkgconfig PKG_CONFIG_PATH='' ./configure \
      --prefix="$prefix" --pkg-config-flags=--static --extra-version="$(fingerprint)" --extra-cflags="$cpu" \
      --disable-everything --disable-programs --disable-doc --disable-network --disable-autodetect \
      --enable-static --disable-shared --enable-pic --disable-avdevice --disable-avfilter --disable-swresample --enable-swscale \
      --enable-decoder=h264,hevc --enable-parser=h264,hevc --enable-demuxer=mov --enable-protocol=file \
      --enable-libopenh264 --enable-muxer=mp4 "${platform[@]}"
    grep -q '^#define FFMPEG_LICENSE "LGPL version 2.1 or later"$' config.h || fail "FFmpeg is not configured as LGPL 2.1+"
    grep -q '^#define HAVE_PTHREADS 1$' config.h || fail "FFmpeg is configured without pthreads"
    make -j "$jobs"
    make install
  )
}

relocate() {
  local prefix=$1 pc
  for pc in "$prefix"/lib/pkgconfig/*.pc; do
    sed -e "s|$prefix|\${pcfiledir}/../..|g" "$pc" > "$pc.relocated"
    mv "$pc.relocated" "$pc"
  done
  if grep -l "$prefix" "$prefix"/lib/pkgconfig/*.pc > /dev/null; then
    fail "build path left in $prefix/lib/pkgconfig"
  fi
}

work=

require_tools() {
  local tool tools="pkg-config make cc c++ curl tar"
  [ "$os" = darwin ] || tools="$tools xz"
  for tool in $tools; do
    command -v "$tool" > /dev/null || fail "$tool is required to build FFmpeg"
  done
}

build() {
  local prefix jobs
  require_tools
  [ ! -e "$target" ] || fail "$target is incomplete: remove it, then build again"
  jobs=$(getconf _NPROCESSORS_ONLN)
  mkdir -p "$cache"
  work=$(mktemp -d "$cache/.build.XXXXXX")
  trap 'rm -rf "$work"' EXIT
  prefix=$work/$(basename "$target")
  export MACOSX_DEPLOYMENT_TARGET=$macos_min
  if [ "$arch" = x86_64 ]; then
    build_nasm "$work" "$jobs"
  fi
  build_openh264 "$work" "$prefix" "$jobs"
  build_ffmpeg "$work" "$prefix" "$jobs"
  relocate "$prefix"
  mv "$prefix" "$cache/" || complete "$target" || fail "could not move the build to $target"
}

clean_bindings() {
  local directory profile folder
  [ -f "$repository/Cargo.toml" ] || return 0
  for directory in "$repository/target" "$repository/target/llvm-cov-target"; do
    for profile in dev release ci; do
      folder=$profile
      [ "$profile" != dev ] || folder=debug
      if [ -d "$directory/$folder/build" ]; then
        (cd "$repository" && cargo clean --quiet -p ffmpeg-sys-next --target-dir "$directory" --profile "$profile")
      fi
    done
  done
}

point_link() {
  local previous
  previous=$(readlink "$link" || true)
  if [ -n "$previous" ] && [ "$previous" != "$target" ]; then
    clean_bindings
  fi
  ln -sfn "$target" "$link"
}

verify() {
  local flags flag minos
  flags=$(PKG_CONFIG_LIBDIR=$link/lib/pkgconfig PKG_CONFIG_PATH='' \
    pkg-config --static --cflags --libs libavformat libavcodec libswscale libavutil)
  for flag in $flags; do
    case $flag in
      -L* | -I*) [ -d "${flag#-?}" ] || fail "pkg-config points to a missing directory: $flag" ;;
    esac
  done
  case " $flags " in *" -lopenh264 "*) ;; *) fail "libavcodec.pc does not link openh264" ;; esac
  case " $flags " in *" -lc++ "* | *" -lstdc++ "*) ;; *) fail "libavcodec.pc does not link the C++ library" ;; esac
  if [ "$os" = darwin ]; then
    minos=$(otool -l "$link"/lib/*.a | awk '$1 == "minos" { print $2 }' | sort -V | tail -1)
    if [ -z "$minos" ] || [ "$(printf '%s\n' "$minos" "$macos_min" | sort -V | tail -1)" != "$macos_min" ]; then
      fail "FFmpeg needs macOS $minos, the binary promises $macos_min"
    fi
  fi
}

case ${1:-} in
  --fingerprint)
    fingerprint
    ;;
  --link)
    complete "$target" || fail "no FFmpeg build at $target: run 'mise run rust:ffmpeg'"
    point_link
    verify
    ;;
  "")
    complete "$target" || build
    point_link
    verify
    ;;
  *)
    fail "usage: $0 [--fingerprint | --link]"
    ;;
esac
