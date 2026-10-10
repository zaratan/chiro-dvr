#!/bin/sh
cd /Users/zaratan/Projects/dvr-wt/rust
L=/Users/zaratan/Projects/dvr-wt/mesure.lock
V=/Users/zaratan/Projects/dvr/in/video_092_original.mp4
K=$(cat /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/kernel.txt)
while pgrep -f stages.py >/dev/null; do sleep 5; done
/bin/df -g . | tail -1
echo "== A python detection only"; lockf $L /usr/bin/time -l uv run python rust-proto/compare.py $V /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_python.pkl 2>&1 | grep -E "detections|real|maximum res"
echo "== B rust standalone lgpl, rayon"; lockf $L /usr/bin/time -l rust-proto/target/ff711-lgpl/release/detect $V $K > /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs1.txt 2> /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs1.err; grep -E "reader|busy|frames|real|maximum res" /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs1.err
uv run python rust-proto/same_detections.py /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_python.pkl /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs1.txt
echo "== C rust standalone lgpl, one core for kernels"; lockf $L env BATDETECT_RUST_THREADS=0 /usr/bin/time -l rust-proto/target/ff711-lgpl/release/detect $V $K > /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs0.txt 2> /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs0.err; grep -E "reader|busy|frames|real|maximum res" /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs0.err
uv run python rust-proto/same_detections.py /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_python.pkl /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_rs0.txt
echo "== D fused pyo3 detection only"; lockf $L env BATDETECT_RUST=fused /usr/bin/time -l uv run python rust-proto/compare.py $V /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_fused.pkl 2>&1 | grep -E "detections|real|maximum res"; cmp /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_python.pkl /private/tmp/claude-501/-Users-zaratan-Projects-dvr-wt-rust/92f76348-e995-4d7b-9b06-27b06c85b422/scratchpad/092_fused.pkl && echo "fused pickle identical"
echo "== E python full CLI (2nd)"; lockf $L sh -c "uptime; /usr/bin/time -l uv run batdetect $V -o out/ref-py2" 2>&1 | grep -E "load|pistes|real|maximum res"
echo "== F fused full CLI (2nd)"; lockf $L sh -c "uptime; BATDETECT_RUST=fused /usr/bin/time -l uv run python rust-proto/run.py $V -o out/rs-fused2" 2>&1 | grep -E "load|pistes|real|maximum res"
cmp out/ref-py/video_092_original/video_092_original_pistes.csv out/ref-py2/video_092_original/video_092_original_pistes.csv && echo "py2 csv identical"
cmp out/ref-py/video_092_original/video_092_original_pistes.csv out/rs-fused2/video_092_original/video_092_original_pistes.csv && echo "fused2 csv identical"
echo SERIES_DONE
