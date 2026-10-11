#!/usr/bin/env bash
set -euo pipefail

script=$(cd "$(dirname "$0")" && pwd -P)/$(basename "$0")
repository=$(cd "$(dirname "$0")/.." && pwd -P)
versions=$repository/packaging/ffmpeg-versions
link=$repository/.ffmpeg

fail() {
  echo "ffmpeg-windows: $*" >&2
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
  { cat "$script"; grep -E '^(FFMPEG|OPENH264|WINDOWS)_' "$versions"; } | sha256 | cut -c 1-16
}

cache=
target=
work=

locate() {
  cache=$(cygpath -u "${XDG_CACHE_HOME:-$HOME/.cache}")/batdetect/ffmpeg
  target=$cache/$(fingerprint)-windows-x86_64
}

complete() {
  [ -f "$1/lib/pkgconfig/libavcodec.pc" ]
}

download() {
  local name=$1 work=$2 url sum archive
  url=$(pinned "${name}_URL")
  sum=$(pinned "${name}_SHA256")
  archive=$work/downloads/$(basename "$url")
  mkdir -p "$work/downloads"
  curl -sSfL --retry 3 --retry-all-errors --connect-timeout 20 -o "$archive" "$url"
  [ "$(sha256 < "$archive")" = "$sum" ] || fail "$archive does not match its pinned sha256"
  echo "$archive"
}

fetch() {
  local name=$1 work=$2 archive
  archive=$(download "$name" "$work")
  mkdir -p "$work/src/$name"
  tar -xf "$archive" -C "$work/src/$name" --strip-components 1
}

install_nasm() {
  local work=$1 archive
  archive=$(download WINDOWS_NASM "$work")
  mkdir -p "$work/tools/bin"
  unzip -q -j "$archive" '*/nasm.exe' -d "$work/tools/bin"
  PATH=$work/tools/bin:$PATH
}

build_openh264() {
  local work=$1 prefix=$2 jobs=$3
  fetch OPENH264 "$work"
  make -C "$work/src/OPENH264" -j "$jobs" OS=msvc ARCH=x86_64 PREFIX="$prefix" install-static
}

keep_configure_log() {
  mkdir -p "$cache/logs"
  cp ffbuild/config.log "$cache/logs/"
  tail -n 80 ffbuild/config.log >&2
}

build_ffmpeg() {
  local work=$1 prefix=$2 jobs=$3
  fetch FFMPEG "$work"
  (
    cd "$work/src/FFMPEG"
    PKG_CONFIG_LIBDIR=$prefix/lib/pkgconfig PKG_CONFIG_PATH='' ./configure \
      --toolchain=msvc --arch=x86_64 --target-os=win64 --extra-cflags=-MT \
      --prefix="$prefix" --pkg-config-flags=--static --extra-version="$(fingerprint)" \
      --disable-everything --disable-programs --disable-doc --disable-network --disable-autodetect \
      --enable-static --disable-shared --disable-avdevice --disable-avfilter --disable-swresample --enable-swscale \
      --enable-decoder=h264,hevc --enable-parser=h264,hevc --enable-demuxer=mov --enable-protocol=file \
      --enable-libopenh264 --enable-muxer=mp4 --enable-encoder=libopenh264 \
      || { keep_configure_log; fail "FFmpeg configure failed: end of config.log above"; }
    grep -q '^#define FFMPEG_LICENSE "LGPL version 2.1 or later"$' config.h || fail "FFmpeg is not configured as LGPL 2.1+"
    grep -q '^#define HAVE_W32THREADS 1$' config.h || fail "FFmpeg is configured without Win32 threads"
    make -j "$jobs"
    make install
  )
}

rename_archives() {
  local prefix=$1 archive name
  for archive in "$prefix"/lib/lib*.a; do
    name=$(basename "$archive" .a)
    mv "$archive" "$prefix/lib/${name#lib}.lib"
  done
}

rewrite_msvc_flags() {
  local prefix=$1 pc
  for pc in "$prefix"/lib/pkgconfig/*.pc; do
    awk '/^Libs/ {
      for (i = 2; i <= NF; i++) {
        if ($i ~ /^-libpath:/) $i = "-L" substr($i, 10)
        else if ($i ~ /^[^-\/][^\/]*\.lib$/) $i = "-l" substr($i, 1, length($i) - 4)
      }
    } { print }' "$pc" > "$pc.rewritten"
    mv "$pc.rewritten" "$pc"
  done
}

relocate() {
  local prefix=$1 posix pc
  posix=$(cygpath -u "$prefix")
  for pc in "$posix"/lib/pkgconfig/*.pc; do
    sed -e "s|$prefix|\${pcfiledir}/../..|g" -e "s|$posix|\${pcfiledir}/../..|g" "$pc" > "$pc.relocated"
    mv "$pc.relocated" "$pc"
  done
  if grep -l -e "$prefix" -e "$posix" "$posix"/lib/pkgconfig/*.pc > /dev/null; then
    fail "build path left in $posix/lib/pkgconfig"
  fi
}

require_tools() {
  local tool linker
  for tool in cl lib dumpbin link pkg-config make cmp curl tar unzip cygpath cmd; do
    command -v "$tool" > /dev/null || fail "$tool is required to build FFmpeg"
  done
  linker=$(link 2>&1 || true)
  case $linker in
    *Microsoft*) ;;
    *) fail "link is not the MSVC linker: coreutils' /usr/bin/link.exe hides it" ;;
  esac
}

build() {
  local prefix mixed jobs
  require_tools
  [ ! -e "$target" ] || fail "$target is incomplete: remove it, then build again"
  jobs=$(nproc)
  mkdir -p "$cache"
  work=$(mktemp -d "$cache/.build.XXXXXX")
  trap 'rm -rf "$work" || true' EXIT
  prefix=$work/$(basename "$target")
  mixed=$(cygpath -m "$prefix")
  install_nasm "$work"
  build_openh264 "$work" "$mixed" "$jobs"
  build_ffmpeg "$work" "$mixed" "$jobs"
  rename_archives "$prefix"
  rewrite_msvc_flags "$prefix"
  relocate "$mixed"
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
  local previous=
  if [ -L "$link" ]; then
    previous=$(cygpath -u "$(readlink "$link")")
  elif [ -e "$link" ]; then
    fail "$link exists and is not a link"
  fi
  [ "$previous" != "$target" ] || return 0
  if [ -n "$previous" ]; then
    clean_bindings
    rm "$link"
  fi
  MSYS2_ARG_CONV_EXCL='*' cmd /c mklink /J "$(cygpath -w "$link")" "$(cygpath -w "$target")" > /dev/null
}

verify() {
  local flags flag archive library directives
  flags=$(PKG_CONFIG_LIBDIR=$(cygpath -w "$link/lib/pkgconfig") PKG_CONFIG_PATH='' \
    pkg-config --cflags --libs libavformat libavcodec libswscale libavutil)
  case $flags in *\\*) fail "pkg-config prints a backslash, which pkg-config-rs reads as an escape: $flags" ;; esac
  for flag in $flags; do
    case $flag in
      -L* | -I*) [ -d "$(cygpath -u "${flag#-?}")" ] || fail "pkg-config points to a missing directory: $flag" ;;
      -libpath:* | *.lib) fail "pkg-config keeps an MSVC flag that pkg-config-rs ignores: $flag" ;;
    esac
  done
  case " $flags " in *" -lopenh264 "*) ;; *) fail "libavcodec.pc does not link openh264" ;; esac
  for archive in "$link"/lib/*.a; do
    [ ! -e "$archive" ] || fail "$archive is not named .lib, so rustc cannot link it"
  done
  for library in "$link"/lib/*.lib; do
    directives=$(dumpbin -nologo -directives "$(cygpath -w "$library")")
    if grep -Eiq 'msvcp?rt' <<< "$directives"; then
      fail "$library asks for the dynamic C runtime"
    fi
    grep -iq 'libcmt' <<< "$directives" || fail "$library does not ask for the static C runtime"
  done
}

case ${1:-} in
  --fingerprint)
    fingerprint
    ;;
  "")
    locate
    complete "$target" || build
    point_link
    verify
    ;;
  *)
    fail "usage: $0 [--fingerprint]"
    ;;
esac
