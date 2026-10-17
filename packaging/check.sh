#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)
extract=video_092_original_3m24-4m05
expected_csv=$repository/tests/fixtures/rust/$extract/tracks.csv
macos_floor=14.0
glibc_floor=2.35
glibcxx_floor=3.4.30
cxxabi_floor=1.3.13
gcc_floor=12.0.0
windows_floor=10.00

macos_libraries="/usr/lib/libSystem.B.dylib
/usr/lib/libobjc.A.dylib
/usr/lib/libc++.1.dylib
/usr/lib/libiconv.2.dylib
/System/Library/Frameworks/Foundation.framework/Versions/C/Foundation
/System/Library/Frameworks/CoreFoundation.framework/Versions/A/CoreFoundation
/System/Library/Frameworks/VideoToolbox.framework/Versions/A/VideoToolbox
/System/Library/Frameworks/CoreMedia.framework/Versions/A/CoreMedia
/System/Library/Frameworks/CoreVideo.framework/Versions/A/CoreVideo
/System/Library/Frameworks/CoreServices.framework/Versions/A/CoreServices"
linux_libraries="libstdc++.so.6
libm.so.6
libgcc_s.so.1
libc.so.6"
windows_libraries="kernel32.dll
ntdll.dll
bcrypt.dll
bcryptprimitives.dll
api-ms-win-core-synch-l1-2-0.dll"

usage() {
  echo "usage: $0 ARCHIVE [VERSION]" >&2
  exit 2
}

[ $# -ge 1 ] && [ $# -le 2 ] || usage
archive=$1
version=${2:-}
[ -f "$archive" ] || { echo "check: no archive at $archive" >&2; exit 1; }

case $(basename "$archive") in
  batdetect-darwin-arm64.tar.gz) system=macos binary=batdetect fingerprint_script=ffmpeg-unix.sh ;;
  batdetect-linux-x86_64.tar.gz) system=linux binary=batdetect fingerprint_script=ffmpeg-unix.sh ;;
  batdetect-windows-x86_64.zip) system=windows binary=batdetect.exe fingerprint_script=ffmpeg-windows.sh ;;
  *) echo "check: $archive is not a batdetect archive" >&2; exit 1 ;;
esac

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
unpacked=$work/archive
mkdir -p "$unpacked"
if [ "$system" = windows ]; then
  unzip -q "$archive" -d "$unpacked"
else
  tar -xzf "$archive" -C "$unpacked"
fi
program=$unpacked/$binary

failures=0
pass() {
  echo "ok   $1"
}
flunk() {
  echo "FAIL $1: $2"
  failures=$((failures + 1))
}

at_most() {
  [ "$(printf '%s\n' "$1" "$2" | sort -V | tail -1)" = "$2" ]
}

in_english() {
  env -u LANGUAGE -u LC_ALL -u LC_MESSAGES LANG=C "$@"
}

english() {
  in_english "$program" "$@"
}

outside() {
  local found=$1 allowed=$2
  LC_ALL=C comm -23 <(LC_ALL=C sort -u <<< "$found") <(LC_ALL=C sort -u <<< "$allowed") | paste -sd ' ' -
}

check_contents() {
  local expected found extra missing
  expected=$(printf '%s\n' "$binary" LICENSE NOTICE)
  if [ -d "$repository/packaging/licenses" ]; then
    expected+=$'\n'licenses$'\n'$(cd "$repository/packaging" && find licenses -mindepth 1)
  fi
  found=$(cd "$unpacked" && find . -mindepth 1 | sed 's|^\./||')
  extra=$(outside "$found" "$expected")
  missing=$(outside "$expected" "$found")
  if [ -n "$extra$missing" ]; then
    flunk C1-contents "unexpected: ${extra:-none}; missing: ${missing:-none}"
  else
    pass C1-contents
  fi
}

linked() {
  case $system in
    macos) otool -L "$program" | tail -n +2 | awk '{ print $1 }' ;;
    linux) readelf -d "$program" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p' ;;
    windows) dumpbin -nologo -dependents "$(cygpath -w "$program")" | grep -Eio '[^[:space:]]+\.dll' | tr '[:upper:]' '[:lower:]' ;;
  esac
}

check_links() {
  local found allowed unexpected
  case $system in
    macos) allowed=$macos_libraries ;;
    linux) allowed=$linux_libraries ;;
    windows) allowed=$windows_libraries ;;
  esac
  found=$(linked) || { flunk C2-links "could not list the linked libraries"; return; }
  unexpected=$(outside "$found" "$allowed")
  if [ -z "$found" ]; then
    flunk C2-links "no linked library found"
  elif [ -n "$unexpected" ]; then
    flunk C2-links "outside the list: $unexpected"
  else
    pass C2-links
  fi
}

highest() {
  { grep -Eo "\\b$1_[0-9]+(\\.[0-9]+)*\\b" <<< "$2" || true; } | sed "s/^$1_//" | sort -V | tail -1
}

check_floor() {
  local minos symbols family floor top headers os subsystem above=
  case $system in
    macos)
      minos=$(vtool -show-build "$program" | awk '$1 == "minos" { print $2 }' | sort -V | tail -1)
      if [ -n "$minos" ] && at_most "$minos" "$macos_floor"; then
        pass "C3-floor (minos $minos)"
      else
        flunk C3-floor "minos ${minos:-missing}, above $macos_floor"
      fi
      ;;
    linux)
      symbols=$(objdump -T "$program")
      for family in GLIBC:$glibc_floor GLIBCXX:$glibcxx_floor CXXABI:$cxxabi_floor GCC:$gcc_floor; do
        floor=${family#*:}
        family=${family%%:*}
        top=$(highest "$family" "$symbols")
        if [ -n "$top" ] && ! at_most "$top" "$floor"; then
          above+=" ${family}_$top>$floor"
        fi
      done
      if [ -n "$above" ]; then
        flunk C3-floor "needs$above"
      else
        pass "C3-floor (GLIBC $(highest GLIBC "$symbols"), GLIBCXX $(highest GLIBCXX "$symbols"))"
      fi
      ;;
    windows)
      headers=$(dumpbin -nologo -headers "$(cygpath -w "$program")" | tr -d '\r') \
        || { flunk C3-floor "could not read the headers"; return; }
      os=$(awk '/ operating system version$/ { print $1; exit }' <<< "$headers")
      subsystem=$(awk '/ subsystem version$/ { print $1; exit }' <<< "$headers")
      if [ -n "$os" ] && [ -n "$subsystem" ] && at_most "$os" "$windows_floor" && at_most "$subsystem" "$windows_floor"; then
        pass "C3-floor (subsystem $subsystem, os $os)"
      else
        flunk C3-floor "subsystem ${subsystem:-missing}, os ${os:-missing}, above $windows_floor"
      fi
      ;;
  esac
}

check_version() {
  local printed first second ffmpeg fingerprint wanted_second
  printed=$(english --version 2>&1) || { flunk C4-version "--version failed: $printed"; return; }
  printed=$(tr -d '\r' <<< "$printed")
  first=$(sed -n 1p <<< "$printed")
  second=$(sed -n 2p <<< "$printed")
  ffmpeg=$(sed -n 's/^FFMPEG_VERSION=//p' "$repository/packaging/ffmpeg-versions")
  fingerprint=$(bash "$repository/packaging/$fingerprint_script" --fingerprint)
  wanted_second="FFmpeg $ffmpeg-$fingerprint — LGPL version 2.1 or later"
  if [ -n "$version" ] && [ "$first" != "batdetect $version" ]; then
    flunk C4-version "'$first', expected 'batdetect $version'"
  elif [ -z "$version" ] && [ "${first#batdetect }" = "$first" ]; then
    flunk C4-version "'$first' does not start with 'batdetect '"
  elif [ "$second" != "$wanted_second" ]; then
    flunk C4-version "'$second', expected '$wanted_second'"
  else
    pass "C4-version ($first)"
  fi
}

check_crates() {
  local notice=$unpacked/NOTICE targets target flags=() listed shipped extra missing
  [ -f "$notice" ] || { flunk C5-crates "no NOTICE in the archive"; return; }
  targets=$(tr -d '\r' < "$repository/packaging/about.toml" | sed -n 's/^targets = \[\(.*\)\]$/\1/p' | tr -d '"' | tr ',' ' ')
  [ -n "$targets" ] || { flunk C5-crates "no targets in packaging/about.toml"; return; }
  for target in $targets; do
    flags+=(--target "$target")
  done
  listed=$(tr -d '\r' < "$notice" | sed -n '/^Rust crates:$/,/^$/p' | sed -n 's/^- \([^ ]*\) \([^ ,]*\),.*/\1 \2/p')
  shipped=$(cd "$repository" && cargo tree --locked -p batdetect -e normal,no-proc-macro --prefix none \
    --format '{p}' "${flags[@]}" | sed -e '/(/d' -e 's/ v/ /' -e '/^$/d' | LC_ALL=C sort -u) \
    || { flunk C5-crates "cargo tree failed"; return; }
  extra=$(outside "$listed" "$shipped")
  missing=$(outside "$shipped" "$listed")
  if [ -z "$shipped" ]; then
    flunk C5-crates "cargo tree lists no crate"
  elif [ -n "$extra$missing" ]; then
    flunk C5-crates "not shipped: ${extra:-none}; missing from NOTICE: ${missing:-none}"
  else
    pass "C5-crates ($(wc -l <<< "$shipped" | tr -d ' ') crates)"
  fi
}

tracks_of_the_extract() {
  local name=$1 out=$2 log=$2.log video=$repository/tests/fixtures/$extract.mp4 produced
  shift 2
  if [ ! -f "$video" ] || head -c 24 "$video" | grep -q '^version https://git-lfs'; then
    flunk "$name" "$video is missing or an LFS pointer: git lfs pull"
    return
  fi
  if ! in_english "$@" "$video" -o "$out" > "$log" 2>&1; then
    flunk "$name" "batdetect failed on the extract: $({ grep -m 1 -A 3 'SDE-ERROR' "$log" || tail -n 5 "$log"; } | tr -d '\r' | paste -sd ' ' -)"
    return
  fi
  produced=$out/$extract/${extract}_tracks.csv
  if [ ! -f "$produced" ]; then
    flunk "$name" "no ${extract}_tracks.csv in $out/$extract"
  elif ! cmp -s "$produced" "$expected_csv"; then
    flunk "$name" "the table differs from ${expected_csv#"$repository"/}: $(diff "$expected_csv" "$produced" | head -n 6 | paste -sd ' ' -)"
  else
    pass "$name"
  fi
}

check_cpu() {
  local sde
  if [ "$system" = macos ]; then
    echo "skip C3-cpu (no Intel SDE on macOS)"
    return
  fi
  sde=$(bash "$repository/packaging/sde.sh") || { flunk C3-cpu "could not fetch Intel SDE"; return; }
  if [ "$system" = linux ] && ! "$sde" -nhm -- /bin/true > /dev/null 2>&1; then
    flunk C3-cpu "this host cannot run a Nehalem process under Intel SDE (its glibc needs x86-64-v3, or kernel.yama.ptrace_scope is not 0): run it on the CI or in ubuntu:22.04"
    return
  fi
  tracks_of_the_extract C3-cpu "$work/cpu" "$sde" -nhm -- "$program"
}

check_tracks() {
  tracks_of_the_extract C6-tracks "$work/out" "$program"
}

check_contents
check_links
check_floor
check_cpu
check_version
check_crates
check_tracks

if [ "$failures" -gt 0 ]; then
  echo "check: $failures check(s) failed on $(basename "$archive")" >&2
  exit 1
fi
echo "check: $(basename "$archive") passes"
