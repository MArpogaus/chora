import glob, json, os, subprocess, sys
T = "/home/user/log/media/2026-11-12/tour-v3"
order = [("1 xp", "01-xp.mp4", "rows-ch1.json", "notes-ch1.json"),
         ("2 tiling", "02-tiling.mp4", "rows-ch2.json", "notes-ch2.json"),
         ("3 mc (light)", "03-mc-light.mp4", "rows-ch3-light.json", "notes-ch3.json"),
         ("3 mc (dark)", "03-mc-dark.mp4", "rows-ch3-dark.json", "notes-ch3.json"),
         ("4 construct", "04-construct.mp4", "rows-ch4.json", "notes-ch4.json"),
         ("5 construct with manager panes", "05-manager-panes.mp4", "rows-ch5.json", "notes-ch5.json"),
         ("6 xp cube", "06-cube.mp4", "rows-ch6.json", "notes-ch6.json"),
         ("7 window drags", "07-drags.mp4", "rows-ch7.json", "notes-ch7.json"),
         ("8 voice and chat", "08-voice.mp4", "rows-ch8.json", "notes-ch8.json")]
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
lines = ["# Timeline of tour-v2, 2026-11-12", "",
         "One row per act. `mm:ss` is the start of the act in the MP4 of its chapter; `joined` is the same moment in `tour-all.mp4`. Stills are in `stills/` (two files `-L` and `-R` for the two-screen chapter 7). Build: 06044cf (the merge of the construct HUD rework), cargo build, debug. Each MP4 starts with a 2.5 s chapter slide. Nothing is cut; each act is preceded and followed by a pause of about one second.", "",
         "| chapter | mm:ss | joined | act (key or pointer) | expected result | what happened | still |", "|---|---|---|---|---|---|---|"]
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
    nf = os.path.join(T, "tools", notes_f)
    notes = json.load(open(nf)) if os.path.exists(nf) else {}
    for r in rows:
        sec = r.get("abs", r["frame"]) / 30
        what = lookup(notes, r.get("still")) or observed.get(r.get("still") or "", "") or ""
        if isinstance(what, list): what = str(what)
        kind = r["kind"]
        still = (r.get("still") or "")
        lines.append(f"| {ch} | {mmss(sec)} | {mmss(off + sec)} | {kind}: {r['description']} | {r['expected']} | {what} | {still} |".replace("\n", " "))
    off += d
    joined.append(p)
open(os.path.join(T, "timeline-table.md"), "w").write("\n".join(lines) + "\n")
lst = os.path.join(T, "tools", "joined.list")
open(lst, "w").write("".join(f"file '{p}'\n" for p in joined))
subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i", lst, "-c:v", "libx264", "-preset", "slow", "-crf", "28", "-pix_fmt", "yuv420p", "-movflags", "+faststart", os.path.join(T, "tour-all.mp4")], check=True)
print("joined", mmss(off))
