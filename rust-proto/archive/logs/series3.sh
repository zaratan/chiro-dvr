#!/bin/sh
cd /Users/zaratan/Projects/dvr-wt/rust
L=/Users/zaratan/Projects/dvr-wt/mesure.lock
V=/Users/zaratan/Projects/dvr/in/video_092_original.mp4
for w in 2 3; do
echo "== A workers $w"; lockf $L sh -c "uptime; /usr/bin/time -l uv run batdetect $V --workers $w -o out/py-w$w" 2>&1 | grep -E "load|pistes|real|maximum res"
cmp out/ref-py/video_092_original/video_092_original_pistes.csv out/py-w$w/video_092_original/video_092_original_pistes.csv && echo "w$w csv identical"
done
echo "== B fused instrumented"; lockf $L env BATDETECT_RUST=fused /usr/bin/time -l uv run python /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/profile_fused.py $V -o out/rs-fused3 2>&1 | grep -vE "^  #|average|page|swaps|block|messages|signals|context|instructions|cycles|footprint"
echo SERIES3_DONE
