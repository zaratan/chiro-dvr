#!/usr/bin/env bash
set -euo pipefail
fingerprint_script=$1
cache_home=$2
echo "XDG_CACHE_HOME=$cache_home" >> "$GITHUB_ENV"
fingerprint=$(bash "$fingerprint_script" --fingerprint)
image=${ImageOS:-$RUNNER_OS}
target_prefix="cargo-$JOB-$RUNNER_OS-$RUNNER_ARCH-$image-$fingerprint-$TOOLCHAIN-"
target_restore_key=
if [ "$EVENT" = pull_request ]; then
  target_restore_key=$target_prefix
fi
{
  echo "key=ffmpeg-$image-$RUNNER_ARCH-$fingerprint"
  echo "target-key=$target_prefix$LOCK"
  echo "target-restore-key=$target_restore_key"
} >> "$GITHUB_OUTPUT"
