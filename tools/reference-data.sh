#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repository"
remote=${BATDETECT_DATA_REMOTE:-chiro-dvr:chiro-dvr-videos}
sums=${BATDETECT_DATA_SUMS:-tests/fixtures/reference/SHA256SUMS}
sums_v020=${BATDETECT_DATA_SUMS_V020:-tests/fixtures/reference-v0.2.0/SHA256SUMS}
bucket_readme=README.md
fail() { echo "rust:data: $*" >&2; exit 1; }
for tool in rclone shasum python3; do
  command -v "$tool" > /dev/null || fail "$tool is not installed"
done
[ -n "${BATDETECT_REFERENCE_DIR:-}" ] || fail "BATDETECT_REFERENCE_DIR is not set"
[ -n "${BATDETECT_VIDEOS_DIR:-}" ] || fail "BATDETECT_VIDEOS_DIR is not set"
checkout=$(dirname "$(git -C "$repository" rev-parse --path-format=absolute --git-common-dir)")
python3 -I - "$BATDETECT_REFERENCE_DIR" "$BATDETECT_VIDEOS_DIR" "${BATDETECT_REFERENCE_V020_DIR:-}" "$repository/in" "$checkout/in" "$HOME/dossier sans titre/img_0000" <<'EOF'
import os
import sys

def lineage(path):
    path = os.path.realpath(path)
    while True:
        yield path
        if os.path.dirname(path) == path:
            return
        path = os.path.dirname(path)

def same(a, b):
    return a == b or (os.path.exists(a) and os.path.exists(b) and os.path.samefile(a, b))

def overlap(a, b):
    return any(same(up, os.path.realpath(b)) for up in lineage(a)) or any(same(up, os.path.realpath(a)) for up in lineage(b))

reference, videos, v020, *protected = sys.argv[1:]
targets = {"BATDETECT_REFERENCE_DIR": reference, "BATDETECT_VIDEOS_DIR": videos}
if v020:
    targets["BATDETECT_REFERENCE_V020_DIR"] = v020
for target in targets.values():
    if not os.path.isabs(target):
        sys.exit(f"rust:data: {target} is not an absolute path")
    for kept in protected:
        if overlap(target, kept):
            sys.exit(f"rust:data: {target} overlaps {kept}, which holds the user's videos")
names = list(targets)
for index, first in enumerate(names):
    for second in names[index + 1:]:
        if overlap(targets[first], targets[second]):
            sys.exit(f"rust:data: {first} and {second} overlap")
EOF

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
sums_under() {
  local prefix=$1 file=$2
  [ -f "$file" ] || fail "$file is missing"
  echo "$file" > "$work/$prefix.file"
  grep -F "  $prefix/" "$file" > "$work/$prefix.lines" || fail "$file has no $prefix/ entry"
  sed "s|  $prefix/|  |" "$work/$prefix.lines" > "$work/$prefix.sums"
  cut -c67- "$work/$prefix.sums" | LC_ALL=C sort > "$work/$prefix.known"
}
sums_of() { command cat "$work/$1.file"; }
refuse_strangers() {
  local prefix=$1 dir=$2 depth=$3 pattern=$4 strangers
  [ -d "$dir" ] || return 0
  strangers=$( (cd "$dir" && find . -mindepth 1 -maxdepth "$depth" -type f -name "$pattern" -not -name .DS_Store -not -name '._*' -not -path "./$bucket_readme") \
    | sed 's|^\./||' | LC_ALL=C sort | LC_ALL=C comm -23 - "$work/$prefix.known" | tr '\n' ' ')
  [ -z "$strangers" ] || fail "$dir holds files that are not in $(sums_of "$prefix"): $strangers"
}
failed_in() { sed -n 's/: FAILED.*$//p' "$1" | paste -sd ' ' -; }
refuse_altered() {
  local prefix=$1 dir=$2 altered
  [ -d "$dir" ] || return 0
  (cd "$dir" && while read -r sum name; do
    if [ -f "$name" ]; then printf '%s  %s\n' "$sum" "$name"; fi
  done < "$work/$prefix.sums") > "$work/present.sums"
  [ -s "$work/present.sums" ] || return 0
  (cd "$dir" && shasum -a 256 -c --quiet "$work/present.sums" > "$work/check" 2>&1) && return 0
  altered=$(failed_in "$work/check")
  [ -n "$altered" ] || { command cat "$work/check" >&2; fail "$dir: shasum failed"; }
  fail "$dir holds videos whose SHA-256 differ from $(sums_of "$prefix"): $altered; delete them by hand if they are damaged copies, and run again"
}
fetch() {
  local prefix=$1 dir=$2 downloaded
  mkdir -p "$dir"
  rclone copy "$remote/$prefix" "$dir" --files-from "$work/$prefix.known" --log-level INFO --log-file "$work/log" \
    || { command cat "$work/log" >&2; fail "rclone copy $remote/$prefix failed"; }
  downloaded=$(grep -cE ': (Multi-thread )?Copied \(' "$work/log" || true)
  if ! (cd "$dir" && shasum -a 256 -c --quiet "$work/$prefix.sums" > "$work/check" 2>&1); then
    command cat "$work/check" >&2
    fail "$dir: SHA-256 differ from $(sums_of "$prefix") for $(failed_in "$work/check"); delete them by hand and run again"
  fi
  echo "$prefix in $dir: $(wc -l < "$work/$prefix.known" | tr -d ' ') files, $downloaded downloaded, SHA-256 all equal"
  rm -f "$work/log"
}
sums_under reference-v0.1.1 "$sums"
sums_under img_0000 "$sums"
refuse_strangers reference-v0.1.1 "$BATDETECT_REFERENCE_DIR" 64 "*"
refuse_strangers img_0000 "$BATDETECT_VIDEOS_DIR" 1 "*.mp4"
refuse_altered img_0000 "$BATDETECT_VIDEOS_DIR"
if [ -n "${BATDETECT_REFERENCE_V020_DIR:-}" ]; then
  sums_under reference-v0.2.0 "$sums_v020"
  refuse_strangers reference-v0.2.0 "$BATDETECT_REFERENCE_V020_DIR" 64 "*"
fi
fetch reference-v0.1.1 "$BATDETECT_REFERENCE_DIR"
fetch img_0000 "$BATDETECT_VIDEOS_DIR"
if [ -n "${BATDETECT_REFERENCE_V020_DIR:-}" ]; then
  fetch reference-v0.2.0 "$BATDETECT_REFERENCE_V020_DIR"
fi
