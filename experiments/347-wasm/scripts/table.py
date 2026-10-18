import json, re, glob, os
os.chdir("/home/zaratan/Projects/dvr-wt/lancement/out/347/runs")
for name in ["native1", "native12", "decode", "simd1", "threads6", "threads12"]:
    for i in (1, 2):
        t = open(f"{name}-{i}.time").read()
        wall = re.search(r"Elapsed.*: (\S+)", t).group(1)
        cpu = float(re.search(r"User time.*: (\S+)", t).group(1)) + float(re.search(r"System time.*: (\S+)", t).group(1))
        line = ""
        for l in open(f"{name}-{i}.json"):
            if l.startswith("{"): line = l
        j = json.loads(line) if line else {}
        err = re.search(r"frames \d+ seconds (\S+)", t)
        print(name, i, "wall", wall, "cpu %.1f" % cpu, "total_s %.2f" % (j.get("totalMs", 0) / 1000), "decode_s %.2f" % (j.get("decodedMs", 0) / 1000), "detect_s %.2f" % (j.get("detectMs", 0) / 1000), "wasm_MB", round(j.get("peakMemory", 0) / 1e6), "chrome_MB", j.get("peakChromeRssMB"), "native_s", err.group(1) if err else "")
