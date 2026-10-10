#!/bin/sh
cd /Users/zaratan/Projects/dvr-wt/rust
for d in /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/allvideos/[0-9]*; do
  v=$(basename $d)
  V="/Users/zaratan/dossier sans titre/img_0000/video_$v.mp4"
  [ -f "$V" ] || continue
  lockf /Users/zaratan/Projects/dvr-wt/mesure.lock ffprobe -v error -threads 1 -show_log 16 -select_streams v:0 -show_entries frame=pts,key_frame:log=message -of json "$V" > /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/probecheck/$v.json 2>/dev/null
  uv run python /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/probecheck/compare.py /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/probecheck/$v.json $d/frames.txt $v
done
echo PROBECHECK_DONE
