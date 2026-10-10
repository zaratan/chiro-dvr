from __future__ import annotations

import math
import pickle
import re
import statistics
import sys
from pathlib import Path

from batdetect.damage import Probe, frames_after_pts_gaps
from batdetect.detect import DetectConfig, Detection
from batdetect.exclusion import exclude
from batdetect.spans import Span
from batdetect.stability import StabilityConfig
from batdetect.track import Track, TrackConfig, track_detections
from batdetect.video import open_video

ROOT = Path(__file__).parent
ORDER = ["089", "090", "091", "093", "122", "125", "126", "127"]
START_TOL, POINTS_TOL, NEAR_PX, MAX_DETAIL = 5, 3, 3.0, 200


def load_fast(found: Path, frames_path: Path) -> tuple[dict[int, list[Detection]], Probe]:
    frames = [line.split() for line in frames_path.read_text().splitlines()]
    detections: dict[int, list[Detection]] = {int(f[0]): [] for f in frames}
    for line in found.read_text().splitlines():
        f = line.split()
        frame = int(f[0])
        x, y, left, top, width, height, area, amplitude = map(float, f[1:])
        detections[frame].append(Detection(frame, x, y, left, top, width, height, area, amplitude))
    pts = [None if f[1] == "-" else int(f[1]) for f in frames]
    probe = Probe(
        frame_count=len(frames),
        error_frames=tuple(int(f[0]) for f in frames if int(f[4]) != 0),
        gap_frames=tuple(frames_after_pts_gaps(pts)),
        key_frames=tuple(int(f[0]) for f in frames if f[2] == "1"),
    )
    return detections, probe


def tc(frame: float, fps: float) -> str:
    t = frame / fps
    m, s = divmod(t, 60)
    return f"{int(m):02d}:{s:05.2f}"


def nearest(d: Detection, pool: list[Detection]) -> Detection | None:
    best = min(pool, key=lambda e: math.hypot(d.x - e.x, d.y - e.y), default=None)
    if best is not None and math.hypot(d.x - best.x, d.y - best.y) < NEAR_PX:
        return best
    return None


def matched_count(a: dict[int, list[Detection]], b: dict[int, list[Detection]]) -> int:
    return sum(1 for fr, v in a.items() for d in v if nearest(d, b.get(fr, [])) is not None)


def times(log: Path) -> tuple[float | None, float | None, float | None, str]:
    if not log.exists():
        return None, None, None, ""
    text = log.read_text()
    m = re.search(r"([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys", text)
    det = re.search(r"^([\d.]+) s, (\d+) detections", text, re.M)
    rss = re.search(r"(\d+)\s+maximum resident set size", text)
    extra = f"det {float(det.group(1)):.0f} s" if det else ""
    if rss:
        extra += f"{', ' if extra else ''}RSS {int(rss.group(1)) / 2**20:.0f} Mo"
    if not m:
        return None, None, None, extra
    return float(m.group(1)), float(m.group(2)), float(m.group(3)), extra


def pair_tracks(a: list[Track], b: list[Track]) -> list[tuple[int, int]]:
    cands = sorted(
        (abs(x.first.frame - y.first.frame), abs(len(x.points) - len(y.points)), i, j)
        for i, x in enumerate(a)
        for j, y in enumerate(b)
        if abs(x.first.frame - y.first.frame) <= START_TOL
    )
    used_a: set[int] = set()
    used_b: set[int] = set()
    pairs = []
    for _, _, i, j in cands:
        if i not in used_a and j not in used_b:
            used_a.add(i)
            used_b.add(j)
            pairs.append((i, j))
    return pairs


def spans_txt(spans: list[Span], fps: float) -> str:
    return ", ".join(f"{tc(s.first, fps)}–{tc(s.last + 1, fps)}" for s in spans) or "aucune"


def describe(
    t: Track,
    side: str,
    other: str,
    other_raw: dict[int, list[Detection]],
    other_kept: dict[int, list[Detection]],
    other_tracks: list[Track],
    other_unstable: list[Span],
    fps: float,
) -> str:
    n = len(t.points)
    raw_hits = [nearest(p, other_raw.get(p.frame, [])) for p in t.points]
    kept_hits = [nearest(p, other_kept.get(p.frame, [])) for p in t.points]
    n_raw = sum(h is not None for h in raw_hits)
    n_kept = sum(h is not None for h in kept_hits)
    in_unstable = sum(any(s.contains(p.frame) for s in other_unstable) for p in t.points)
    amps = [p.amplitude for p in t.points]
    areas = [p.area for p in t.points]
    amp_pairs = [(p.amplitude, h.amplitude) for p, h in zip(t.points, raw_hits, strict=True) if h is not None]
    owners: dict[int, int] = {}
    for p in t.points:
        for k, o in enumerate(other_tracks):
            if o.first.frame <= p.frame <= o.last.frame and any(
                q.frame == p.frame and math.hypot(q.x - p.x, q.y - p.y) < NEAR_PX for q in o.points
            ):
                owners[k] = owners.get(k, 0) + 1
    span = f"{tc(t.first.frame, fps)}–{tc(t.last.frame, fps)}"
    chord = t.chord()
    lines = [
        f"- **{side} seule** : {span} (images {t.first.frame}–{t.last.frame}), {n} points, corde {chord:.0f} px, "
        f"amplitude médiane {statistics.median(amps):.1f} (min {min(amps):.1f}), surface médiane {statistics.median(areas):.0f}",
        f"  - côté {other} : {n_raw}/{n} points ont une détection à moins de 3 px avant exclusion, {n_kept}/{n} après ; "
        f"{in_unstable}/{n} points dans une plage instable {other}",
    ]
    if amp_pairs:
        ratio = statistics.median(b / a for a, b in amp_pairs)
        lines.append(f"  - amplitude {other}/{side} sur les points communs : rapport médian {ratio:.2f}")
    if owners:
        own = ", ".join(
            f"piste {other} {tc(other_tracks[k].first.frame, fps)} ({len(other_tracks[k].points)} pts, {c} pts communs)"
            for k, c in sorted(owners.items())
        )
        lines.append(f"  - points repris par : {own}")
    else:
        lines.append(f"  - aucun point repris par une piste {other}")
    missing = [p.frame for p, h in zip(t.points, raw_hits, strict=True) if h is None]
    if missing and len(missing) <= 20:
        lines.append(f"  - images sans détection {other} : {missing}")
    elif missing:
        lines.append(f"  - images sans détection {other} : {len(missing)} (de {missing[0]} à {missing[-1]})")
    return "\n".join(lines)


def main() -> None:
    vid, video = sys.argv[1], Path(sys.argv[2])
    d = ROOT / vid
    py: dict[int, list[Detection]] = pickle.loads((d / "py.pkl").read_bytes())
    fa, probe = load_fast(d / "fast.txt", d / "frames.txt")
    cfg = DetectConfig()
    cap, info = open_video(video, cfg.work_width)
    cap.release()
    fps = info.fps
    n_py = sum(map(len, py.values()))
    n_fa = sum(map(len, fa.values()))
    m_py = matched_count(py, fa)
    m_fa = matched_count(fa, py)
    an_py = exclude(py, probe, cfg.half_window(fps), fps, StabilityConfig())
    an_fa = exclude(fa, probe, cfg.half_window(fps), fps, StabilityConfig())
    tr_py = track_detections(an_py.detections, TrackConfig())
    tr_fa = track_detections(an_fa.detections, TrackConfig())
    pairs = pair_tracks(tr_py, tr_fa)
    ok = [(i, j) for i, j in pairs if abs(len(tr_py[i].points) - len(tr_fa[j].points)) <= POINTS_TOL]
    off = [(i, j) for i, j in pairs if (i, j) not in ok]
    lone_py = [i for i in range(len(tr_py)) if i not in {p[0] for p in pairs}]
    lone_fa = [j for j in range(len(tr_fa)) if j not in {p[1] for p in pairs}]
    big = max(len(tr_py), len(tr_fa)) > MAX_DETAIL

    out = [
        f"# video_{vid}",
        "",
        f"fps {fps:.3f}, {probe.frame_count} images, {len(probe.error_frames)} images réparées, "
        f"{len(probe.gap_frames)} sauts de pts",
        f"Détections : python {n_py}, fast {n_fa}, python retrouvées par fast {m_py} ({m_py / max(1, n_py):.2%}), "
        f"fast retrouvées par python {m_fa} ({m_fa / max(1, n_fa):.2%})",
        f"Plages abîmées (communes) : {spans_txt(an_py.damaged, fps)}",
    ]
    hidden_py = an_py.ignored_frames
    n_py_in = sum(len(v) for fr, v in py.items() if fr not in hidden_py)
    m_py_in = sum(
        1 for fr, v in py.items() if fr not in hidden_py for dd in v if nearest(dd, fa.get(fr, [])) is not None
    )
    for side, a, b, an in (("python", py, fa, an_py), ("fast", fa, py, an_fa)):
        ignored = an.ignored_frames
        miss = [dd for fr, v in a.items() for dd in v if nearest(dd, b.get(fr, [])) is None]
        outside = [dd for dd in miss if dd.frame not in ignored]
        amps = sorted(dd.amplitude for dd in outside)
        frames_out = sorted({dd.frame for dd in outside})
        txt = f"Détections {side} sans voisine de l'autre côté : {len(miss)}, dont {len(outside)} hors plages ignorées ({side})"
        if outside:
            txt += (
                f", amplitude médiane {statistics.median(amps):.1f} (max {amps[-1]:.1f}), "
                f"surface médiane {statistics.median(dd.area for dd in outside):.0f}, sur {len(frames_out)} images "
                f"({tc(frames_out[0], fps)}…{tc(frames_out[-1], fps)})"
            )
        out.append(txt)
    if an_py.unstable == an_fa.unstable:
        out.append(f"Plages instables identiques : {spans_txt(an_py.unstable, fps)}")
    else:
        out.append(f"Plages instables python : {spans_txt(an_py.unstable, fps)}")
        out.append(f"Plages instables fast : {spans_txt(an_fa.unstable, fps)}")
    out.append(
        f"Pistes : python {len(tr_py)}, fast {len(tr_fa)}, appariées ±{START_TOL} images {len(pairs)}, "
        f"dans la tolérance de points ±{POINTS_TOL} {len(ok)}"
    )
    if not big:
        deltas = [
            (tr_fa[j].first.frame - tr_py[i].first.frame, len(tr_fa[j].points) - len(tr_py[i].points)) for i, j in pairs
        ]
        if deltas:
            out.append(
                f"Écarts de début (fast−python) : {sorted({a for a, _ in deltas})}; "
                f"écarts de points : {sorted({b for _, b in deltas})}"
            )
        out.append("")
        out.append("Pistes python : " + ", ".join(f"{tc(t.first.frame, fps)}({len(t.points)})" for t in tr_py))
        out.append("")
        for i, j in ok:
            a, b = tr_py[i], tr_fa[j]
            if a.first.frame != b.first.frame or len(a.points) != len(b.points):
                out.append(
                    f"- écart dans la tolérance : python {tc(a.first.frame, fps)} {len(a.points)} pts "
                    f"(fin {tc(a.last.frame, fps)}), fast {tc(b.first.frame, fps)} {len(b.points)} pts "
                    f"(fin {tc(b.last.frame, fps)})"
                )
        for i, j in off:
            a, b = tr_py[i], tr_fa[j]
            out.append(
                f"- **hors tolérance de points** : python {tc(a.first.frame, fps)} {len(a.points)} pts "
                f"(fin {tc(a.last.frame, fps)}), fast {tc(b.first.frame, fps)} {len(b.points)} pts (fin {tc(b.last.frame, fps)})"
            )
        for i in lone_py:
            out.append(describe(tr_py[i], "python", "fast", fa, an_fa.detections, tr_fa, an_fa.unstable, fps))
        for j in lone_fa:
            out.append(describe(tr_fa[j], "fast", "python", py, an_py.detections, tr_py, an_py.unstable, fps))
    (d / "details.md").write_text("\n".join(out) + "\n")
    print("\n".join(out))

    py_real, py_user, py_sys, py_extra = times(d / "py.log")
    fa_real, fa_user, fa_sys, fa_extra = times(d / "fast.log")
    if big:
        lone = f"(> {MAX_DETAIL} pistes) py seules {len(lone_py)}, fast seules {len(lone_fa)}, hors tol. {len(off)}"
    else:
        items = [f"py {tc(tr_py[i].first.frame, fps)} ({len(tr_py[i].points)})" for i in lone_py]
        items += [f"fast {tc(tr_fa[j].first.frame, fps)} ({len(tr_fa[j].points)})" for j in lone_fa]
        items += [
            f"hors tol. py {tc(tr_py[i].first.frame, fps)} {len(tr_py[i].points)}/fast {len(tr_fa[j].points)} pts"
            for i, j in off
        ]
        lone = "; ".join(items) or "—"
    if an_py.unstable != an_fa.unstable:
        lone += f" — instables py {len(an_py.unstable)} / fast {len(an_fa.unstable)}"
    py_t = f"{py_real:.0f} ({py_extra})" if py_real else "?"
    fa_t = f"{fa_real:.1f} / {fa_user + fa_sys:.1f}" if fa_real and fa_user is not None and fa_sys is not None else "?"
    row = (
        f"| {vid} | {n_py} | {n_fa} | {m_py / max(1, n_py):.2%} ({m_py_in / max(1, n_py_in):.2%} hors ignorées) | {len(tr_py)} | {len(tr_fa)} | "
        f"{len(ok)}/{max(len(tr_py), len(tr_fa))} | {lone} | {py_t} | {fa_t} |"
    )
    (d / "row.md").write_text(row + "\n")
    stats = (
        n_py,
        n_fa,
        m_py,
        len(tr_py),
        len(tr_fa),
        len(ok),
        len(lone_py),
        len(lone_fa),
        len(off),
        py_real or 0,
        fa_real or 0,
        (fa_user or 0) + (fa_sys or 0),
    )
    (d / "stats.txt").write_text(" ".join(map(str, stats)) + "\n")
    rebuild()


def rebuild() -> None:
    vids = sorted(p.name for p in ROOT.iterdir() if (p / "row.md").exists())
    vids.sort(key=lambda v: (ORDER.index(v) if v in ORDER else len(ORDER), v))
    header = [
        "# Détection fast contre Python, toutes les vidéos",
        "",
        "Même `Probe` (frames.txt) des deux côtés ; `exclude` puis `track_detections`, configs par défaut. "
        f"Appariement des pistes par image de début ±{START_TOL}, points ±{POINTS_TOL}.",
        "",
        "| Vidéo | Dét. py | Dét. fast | % py retrouvées | Pistes py | Pistes fast | Appariées dans tol. | "
        "Sans correspondant (début, points) | Python réel s | fast réel / CPU s |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]
    rows = [(ROOT / v / "row.md").read_text().strip() for v in vids]
    tot = [0.0] * 12
    for v in vids:
        for k, x in enumerate((ROOT / v / "stats.txt").read_text().split()):
            tot[k] += float(x)
    total = (
        f"| **Total ({len(vids)})** | {tot[0]:.0f} | {tot[1]:.0f} | {tot[2] / max(1, tot[0]):.2%} | {tot[3]:.0f} | "
        f"{tot[4]:.0f} | {tot[5]:.0f} | py seules {tot[6]:.0f}, fast seules {tot[7]:.0f}, hors tol. {tot[8]:.0f} | "
        f"{tot[9]:.0f} | {tot[10]:.0f} / {tot[11]:.0f} |"
    )
    (ROOT / "resultats.md").write_text("\n".join([*header, *rows, total]) + "\n")


if __name__ == "__main__":
    if sys.argv[1] == "rebuild":
        rebuild()
    else:
        main()
