# media r2 report (rc 11, 1b), 2026-11-16

Build: session branch 6658888 (cargo build, debug, headless, software rendering). Scripts on
claude/media at 2ef891d (pushed, hooks passed). Sonnet worker, one round.

## Done
- design/tour-v4.md: one scenario per look, each door shown once.
- Scripts `demo/tour/v4_*.py`, `tour4.py` (Quick: 12 cps, pauses <= 0.6 s, `cut` for waits),
  `assemble4.py`. Output `media/2026-11-16/tour-v4/`: six chapter MP4, `tour-all.mp4`
  (3:18, 23 MB with stills), `timeline.md`.
- Chapters: xp 0:55 (Erika: letter, wobble, paperclip asks about sound, Start > Control Panel >
  Sound 75%, helper asks before copying the letter), tiling 0:47 (project in nano, Apps door,
  four tiles, layouts, master, tags, float, Super+drag wobble, key list, Super+Print),
  mc light 0:24 (kitchen: voice, timer card, Next, remote, column hidden, alarm), mc dark 0:15
  (sofa: remote, Programs, chat), construct 0:19, handover 0:38 (xp > tiling > construct > xp >
  mc > xp, two key gives, audit: one window s1, owner xp).

## Deviations
- Length: 3:18 in all, under 6 min, but looks run 15 to 55 s, not 60 to 75: with pauses <= 0.6 s,
  no static page over 1.5 s and cuts there is little left to show. Not padded.
- `freezedetect` (-80 dB, 1.5 s) still finds 25 spans, 6 of them the chapter slides (2.5 s); the
  rest are 1.5 to 2.4 s (reading pages, animations after a cut). The 1.5 s limit is not met.
- Construct: pulling a window off a face with a scripted drag did not work (the drag selects text
  in the client); dropped from the chapter, the pull is in the handover. Manager panes not shown.
- Stills of the dark mc, handover and construct were not reviewed one by one.
- Harness: a modifier press and the button press in two inject calls do not make a Super+drag;
  in one call they do. `Tour.drag` now sends them together.

## Findings (not fixed)
| Time (joined) | Sev | Finding |
|---|---|---|
| tiling, last act | MINOR | Super+Print shows no notice or any state in the session (no notify daemon here); unverified whether a file was written. |
| tiling, after Apps | MINOR | The tooltip "Run a program (Super+Space)" stays at the bar after the launcher closes (as r1 for Keys). |
| mc light | MINOR | A long chapter title (a scene named "Spaghetti with garlic and oil") runs under the right column; shortened in the tour. |
| mc | note | No timer feature: the timer card is a notification pushed by the script. A scene file added after start is ordered after the built-in ones (scene.move needed). |
| construct | MINOR | Second Ctrl+Alt+Right about 1.5 s after the first showed "face 1 of 4" again, not face 3 (unverified; may be a key during the turn). |
| construct | MAJOR (unverified) | Window pull from a face by drag does not pull (r1 saw the same); the drag-turn from empty floor sent the cube off screen in a first run (as r1 06-06). |
| xp | note | Control Panel cascade items are not nodes in the scene (no ids to query); only one wallpaper ("Bloff"), so the Desktop entry has no choice. |
