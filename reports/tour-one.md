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

## Review r1 (review-tour-one r1, claude/tour-one 8b996cee vs base 9dc384b3, 2026-11-20)

**Verdict: FIX** (2 MAJOR open, 0 BLOCK; 4 MINOR, 1 nit). One round, read-only on code, no fix made (F-1 already fixed by the lane).
Checks: `cargo check` and `cargo build --workspace` ok; `python3 -I tests/test_tour.py` 7 ok; `tools/cut-check` ok; `tests/cut-check-selftest.sh` ok; `demo/tour/run.py --check` all five chapters, 0 red, 0 broken.

**Per check**
1. One set: no `ch*`, `v4-v8_*`, `tour4-8`, `assemble4-8`, `multi`, `cons` left; `git grep` finds no import or doc that names them (gate cells: only `tests/gate-acts.sh` calls `tests/test_tour.py`). The folder holds 3 files beyond the brief's set (`dual.py`, `probe.py`, `ip.py`, rt-7). `tools/copies.toml` row plus `cut-check` selftest red on a planted `v9_xp.py`: ok.
2. v8 fixes: `launch.py` is unchanged by the diff, so PATH without writers (`path_without_writer`, launch.py:105) and keep-bus (launch.py:152) are the v8 code; the pane-centre aim in `chapters/construct.py:51-52` is line-for-line v8's; the probe bus survives only as the stand-alone `probe.py` (not used by a chapter, not in `--check`).
3. Rows: `expect` is on **3 of 90 acts** (xp 2/22, handover 1/13, tiling, mc, construct 0); `contract`, `cells`, `capture` on about half. `known=` names I3, I4, I5 only (xp 1, mc 3, handover 2); `todo` only on handover h2. Read: xp "Start, Control Panel, Sound, 75%" (contract T4:volume, no expect, fixed pointer coordinates), tiling and construct have no expect at all, mc "the helper turns the page" (no expect).
4. `--check` counts: xp 20 green/1 known, tiling 21, construct 18, handover 9/2 known/1 todo reproduced; **mc 15 green/3 known** (lane: 14/4): the light "the answer" act did not go red this run (rt-3).
5. Red first: held only for the cut-check row (as the lane says). `tests/test_tour.py` on its parent 9dc384b3 fails at import (`assemble.py` runs at import and writes `/home/user/log/...`; `tour.Checker`, `run.py`, `row_line` do not exist), not for the named reason. Gap rated MINOR: the 7 tests pass on the head and `--check` finds real reds in four chapters, so the red path is exercised end to end (rt-6).
6. F-1: the diagnosis is right. chora-notify steps on `Instant::now()` (plugins/chora-notify/src/main.rs:528, card.rs:235; default `shortest` 10 s) and `record.step` moves only the core's scene clock (crates/chora/src/record.rs:203-224); the pending segment is rendered after the push, so the card is gone before the frames are reached. The fix (two config lines) is correct for xp and not for mc (rt-2).

**Findings**
- rt-1, MAJOR, demo/tour/tour.py:342 (and assemble.py:35): an act with no `expect` is armed with a default `Expect()` and shows `green` in the timeline; 87 of 90 acts have none, so `green` means only "I1 to I14 hold" and a silent no-op act (a missed click, a skipped branch) is green; assemble.py's docstring promises `-` for no check, which is unreachable. Fix: give such rows status `inv` (shown `inv`, counted apart from `green` in `run.py` tally) and `green` only when an `expect` held.
- rt-2, MAJOR, demo/tour/chapters/mc.py:49 (notify.push at :90, :112): F-1 is fixed in `xp.py:29` only; the mc chapter has the same cause (chora-notify default 10 s wall clock) and no config, so the recorded timer card (m3) and critical note never show in the frames, and the later Screenshot notices (x8, t7, c6) will hit it too; in xp the 600 s card also stays up for the rest of the chapter and may cover later acts (not checked on frames; I9 does not flag it). Fix: put the notify timing in one place (`Launcher.start` default or a `notify_config()` helper in launch.py) and have RECORD look at the card frame in xp and mc; a product fix (notify on the scene clock) stays the clean one.
- rt-3, MINOR, demo/tour/chapters/mc.py:91-113: `--check` is not repeatable, 15/3 against the lane's 14/4: one I5 overprint (the light "the answer" act) depends on the read time; reads are not tied to a quiet frame (`busy` empty). Fix: wait for `busy` empty before `check_armed` reads, or mark that act `known` as flaky with its id.
- rt-4, MINOR, demo/tour/tour.py:376: `known=` hides any red of the act, not only the invariant it names, and the docstring (tour.py:~320) says a known act that turns green "says so" but only the red branch prints. Fix: match `known` against the invariant key (`I4`) of each red line, keep the others `RED`, and print `known act N is green: drop known`.
- rt-5, MINOR, demo/tour/chapters/construct.py:39: the C2 act "drag the pane back over the face" runs only `if pr and fr:`; with the pane or face rect missing it is silently skipped and the chapter stays green, and its result is only a `print("PANE HIT")` (:49), no `expect`. Fix: `raise RuntimeError("no pane")` as `mc.door()` does and replace the print by `Expect(verify=...)` on the owner of `s1`.
- rt-6, MINOR, tests/test_tour.py:1: test-first held only for the cut-check row; the tour test never failed for its named reason (see 5), and `contract`, `cells`, `capture` are stored in the rows but neither judged nor shown (assemble.py:21 `HEAD` has no such column). Fix: add the three columns to the timeline table, and a test that a row without `expect` is not `green` (with rt-1).
- rt-7, nit, demo/tour/probe.py:1, demo/tour/ip.py:1: two stand-alone exploration scripts outside the brief's set; `probe.py` runs at import and repeats the xp launch (without the F-1 config); the row files keep the `rows-ch1..5.json` names (assemble.py:14-19, run.py:24-28). Fix: delete `ip.py`, move `probe.py` to `tools/` or into the xp chapter's check, rename the row files to the chapter names.

Counts: BLOCK 0, MAJOR 2, MINOR 4, nit 1 (7 findings).
