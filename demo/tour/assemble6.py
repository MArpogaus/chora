import glob, json, os, subprocess, sys
T = os.environ.get("TOUR_OUT", "/tmp/claude-0/tour-v6")
BUILD = os.environ.get("TOUR_BUILD", "unknown")
order = [("1 xp", "01-xp.mp4", "rows-ch1.json", "-"),
         ("2 tiling", "02-tiling.mp4", "rows-ch2.json", "-"),
         ("3 mc (light)", "03-mc-light.mp4", "rows-ch3-light.json", "-"),
         ("3b mc (dark)", "03b-mc-dark.mp4", "rows-ch3b-dark.json", "-"),
         ("4 construct", "04-construct.mp4", "rows-ch4.json", "-"),
         ("5 handover", "05-handover.mp4", "rows-ch5.json", "-")]
def dur(p):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", p], capture_output=True, text=True).stdout
    return float(out.strip())
def mmss(s):
    s = int(s); return f"{s // 60:02d}:{s % 60:02d}"
def lookup(notes, still):
    if not still: return ""
    for k in (still, still.replace("-light", "").replace("-dark", "")):
        if k in notes: return notes[k]
    return ""
lines = ["# Timeline of tour-v6, 2026-11-16", "",
         f"One row per act. `mm:ss` is the start of the act in the MP4 of its chapter; `joined` is the same moment in `tour-all.mp4`. Stills are in `stills/` (two files `-L` and `-R` for the two-screen chapter 7). Build: {BUILD}, cargo build, debug, software rendering. Each MP4 starts with a 2.5 s chapter slide. The clock is stepped, so the pace is the one the script gives, not that of the renderer. Waits for animations are cut (`Tour.cut`), the acts are not.", "",
         "| chapter | mm:ss | joined | what the person does | what should be seen | still |", "|---|---|---|---|---|---|"]
joined = []
off = 0.0
for ch, mp4, rows_f, notes_f in order:
    p = os.path.join(T, mp4)
    if not os.path.exists(p): continue
    d = dur(p)
    rows = json.load(open(os.path.join(T, rows_f))) if os.path.exists(os.path.join(T, rows_f)) else []
    observed = {}
    if isinstance(rows, dict):
        observed = rows.get("observed", {}); rows = rows["rows"]
    nf = os.path.join(T, "notes", notes_f)
    notes = json.load(open(nf)) if os.path.exists(nf) else {}
    for r in rows:
        sec = r.get("abs", r["frame"]) / 30
        what = lookup(notes, r.get("still")) or observed.get(r.get("still") or "", "") or ""
        if isinstance(what, list): what = str(what)
        kind = r["kind"]
        still = (r.get("still") or "")
        lines.append(f"| {ch} | {mmss(sec)} | {mmss(off + sec)} | {r['description']} | {r['expected']} | {still} |".replace("\n", " "))
    off += d
    joined.append(p)
open(os.path.join(T, "timeline-table.md"), "w").write("\n".join(lines) + "\n")
lst = os.path.join(T, "joined.list")
open(lst, "w").write("".join(f"file '{p}'\n" for p in joined))
norm = []
for p in joined:
    q = p.replace(".mp4", "-1080.mp4")
    subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", p, "-vf", "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:0x20242c,setsar=1", "-r", "30", "-c:v", "libx264", "-preset", "veryfast", "-crf", "20", "-pix_fmt", "yuv420p", q], check=True)
    norm.append(q)
open(lst, "w").write("".join(f"file '{p}'\n" for p in norm))
subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i", lst, "-c:v", "libx264", "-preset", "slow", "-crf", "28", "-pix_fmt", "yuv420p", "-movflags", "+faststart", os.path.join(T, "tour-all.mp4")], check=True)
print("joined", mmss(off))
