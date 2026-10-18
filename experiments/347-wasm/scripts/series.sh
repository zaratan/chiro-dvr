#!/bin/sh
cd /home/zaratan/Projects/dvr-wt/lancement/out/347
V=/home/zaratan/Projects/dvr-wt/verrou
run() { name=$1; shift; $V chrono /usr/bin/time -v "$@" > runs/$name.json 2> runs/$name.time; grep -h "Elapsed\|Maximum resident" runs/$name.time | tr '\n' ' '; echo " $name"; tail -c 400 runs/$name.json; echo; }
for i in 1 2; do
run native1-$i env RAYON_NUM_THREADS=1 target/release/native raw/extract.yuv 1440 1080 1230
run native12-$i env RAYON_NUM_THREADS=12 target/release/native raw/extract.yuv 1440 1080 1230
run decode-$i node web/run.mjs '{"run":{"video":"/video/extract.mp4","detect":false}}'
run simd1-$i node web/run.mjs '{"run":{"video":"/video/extract.mp4","detect":true,"pkg":"/pkg-simd"},"out":"runs/simd1-'$i'"}'
run threads6-$i node web/run.mjs '{"isolated":true,"run":{"video":"/video/extract.mp4","detect":true,"pkg":"/pkg-threads","threads":6},"out":"runs/threads6-'$i'"}'
run threads12-$i node web/run.mjs '{"isolated":true,"run":{"video":"/video/extract.mp4","detect":true,"pkg":"/pkg-threads","threads":12},"out":"runs/threads12-'$i'"}'
done
