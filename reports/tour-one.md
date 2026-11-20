# tour-one

## r1 (lane 1 of design/tour-v9.md section 8, 2026-11-20)

Branch `claude/tour-one`, base 9dc384b3, commit **8b996cee** (pushed). Gate on it: `pre-commit`, `ci-check`, `gate-lib`, `build`, `test`, `acts`: all PASS (`--jobs 1`; 1 s, 70 s, 85 s, 20 s, 167 s, 248 s).

**Lines (demo/tour, `*.py`):** before 5,307 in 59 files, after 1,937 in 15 files (`tour`, `human`, `launch`, `slides`, `assemble`, `run`, `dual`, `probe`, `ip`, `chapters/{xp,tiling,mc,construct,handover,__init__}`). **Copies: 0** (`tools/copies.toml` row "tour: a versioned copy of a tour script": a new `files` form that counts file names; `tests/cut-check-selftest.sh` plants `v9_xp.py` and sees it fail).

**What changed**
- One set. `Quick` moved into `human.py`, `multi.py` into `chapters/handover.py`, mc light and dark are one chapter, `cons.py` and ch1-8, ch7b, v4-v8_*, tour4-8, assemble4-8 are gone. v8 fixes kept: PATH without writers (`launch.path_without_writer`), pane-centre aim, keep-bus (`CHORA_TEST_KEEP_BUS`), the probe bus (`probe.py`).
- An act is a row: `expect`, `contract`, `cells`, `capture`, plus `known`, `todo`, `opens`, `closes`. After `done()` the tour records what is pending and reads the session, then `Checker` runs `Expect.check` and every invariant (I1-I14) of tests/acts on the snapshots (each act alone). Red act = `RED: ...` in the `check` column of `timeline-table.md`, a `RED` line and exit 1 in `run.py`.
- `run.py --check [chapter...]`: same rows, no video, no stills, no pace work.
- Red tests: `tests/test_tour.py` (a failing expect is a red row; a holding one green; `todo` and `known` are not failures; no snapshot is red), called from `tests/gate-acts.sh`; the copies row in `tests/cut-check-selftest.sh` was red (KeyError) before `tools/cut-check` learned `files`. (The row tests were written beside the code, not strictly before it; only the cut-check one was seen red first.)
- A first `--check` found harness gaps in the tour, fixed in the chapters: the checker needs its grant; construct needs the look `xp` (the scene runs xp on the faces) and its edge output is bare (I7); handover leaves tiling's wallpaper to swaybg (I7); the mc light chapter had no notification bus or chora-notify, so `notify.push` broke it (v8 had the same hole in this container).

**`run.py --check` on all five chapters (build 9dc384b3, this container, software rendering)**
```
chapter xp: 21 acts, 20 green, 0 red, 1 known, 0 todo
chapter tiling: 21 acts, 21 green, 0 red, 0 known, 0 todo
chapter mc: 18 acts, 14 green, 0 red, 4 known, 0 todo
chapter construct: 18 acts, 18 green, 0 red, 0 known, 0 todo
chapter handover: 12 acts, 9 green, 0 red, 2 known, 1 todo
tour: check: 0 red, 0 broken
```
`known` rows are real reds the invariants find today, listed as `known=` on the act (not failures; a note will say when one turns green):
- xp, Super+drag turns the cube: I4 the Start menu opens (Super released after the drag).
- mc light, timer card, "how much garlic", the answer (3 acts): I5 the Power glyph of the column lies under the notice card body (v8 open row).
- mc dark, Escape closes the chat: I5 the headers of two pages (mc-r0, mc-r1) are drawn at once.
- handover, first two acts: I3 construct and xp both say they hold the focus on a four-output session.
`todo`: handover h2 (title-bar right click, Send To > Tiling desktop), a stub that waits for xp-give. The rows carry `contract`, `cells` and `capture` for the acts that map to the table of section 1 to 5; CHAPTERS-A and CHAPTERS-B bring the rest of the v9 acts (x3, x7-x11, t2, t6-t10, m1-m11, c3-c7, h3-h7) and the `expect` fields beyond `new_window`.

**F-1 (no notice in `choractl record`) diagnosed and fixed (under 20 lines)**
Cause: chora-notify expires a notice on the wall clock (`Instant::now()`, `shortest: 10` s, plugins/chora-notify/src/card.rs, config.rs); `choractl record` steps the scene clock and a frame takes seconds of wall time, so a notice is closed before the frames that should show it are reached. Not the recorder drawing, and not the product (the bus carries it, as media-r6 saw).
Evidence at bus level, no video needed: with an `Expect(verify=...)` on the Notepad act, chora-notify said `accepted [1], closed [1], shown []` when the act was read (RED); with `chora-notify.yaml` `shortest: 600, longest: 600` in the xp chapter it is in `shown` (green, nothing else turned red).
Fix: two config lines in `chapters/xp.py` (`extra_config`). Not verified in a recorded video here (no record run in this lane): RECORD confirms the frame at the notice, and says if a notice that stays 600 s covers a later act. A product fix (chora-notify on the scene clock while recording) is the clean one and is a core slice if the owner wants it.

**Not done / open**
- No recording in this lane (cost): `run.py` recording mode and `assemble.py` were exercised by the unit test and by `--check`, not by an MP4 run.
- Section 8's `intended.toml`, tour-diff and the video checker are lanes 2 and 6.
- `notes/` (v1 and v2 stills notes) stays in the folder, unread by `assemble.py` now.
