#!/bin/sh
cd /Users/zaratan/Projects/dvr-wt/rust
L=/Users/zaratan/Projects/dvr-wt/mesure.lock
K=$(cat /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/kernel.txt)
for v in 089 091; do
V="/Users/zaratan/dossier sans titre/img_0000/video_$v.mp4"
/bin/df -g . | tail -1
echo "== $v python"; lockf $L /usr/bin/time -l uv run python rust-proto/compare.py "$V" /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_python.pkl 2>&1 | grep -E "detections|real|maximum res"
echo "== $v rust standalone"; lockf $L /usr/bin/time -l rust-proto/target/ff711-lgpl/release/detect "$V" $K > /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_rs.txt 2> /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_rs.err; grep -E "frames|real|maximum res" /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_rs.err
uv run python rust-proto/same_detections.py /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_python.pkl /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_rs.txt
echo "== $v fused"; lockf $L env BATDETECT_RUST=fused /usr/bin/time -l uv run python rust-proto/compare.py "$V" /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_fused.pkl 2>&1 | grep -E "detections|real|maximum res"; cmp /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_python.pkl /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/${v}_fused.pkl && echo "$v fused identical"
done
echo SERIES2_DONE
