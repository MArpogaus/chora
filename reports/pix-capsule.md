# pix-capsule

## r1

Head: claude/pix-capsule 05828b32 (red 434e66c3, green 05828b32), on e905195e. Already contains origin/claude/autonomous-dev-session-3w-i33pg7 (merge was a no-op).

**Cause.** The capsule over a stepped-into face is the hint set of the camera: it shows for FLOWN = 3 s after the step in (plugins/chora-construct/src/hints.rs:32, started at overlay.rs:506), then only while the pointer rests low (hints.rs:139 resting_set). tests/construct-captures.sh, stepped-into-face, took its shot after `key Super+c 2.5` (a bare 2.5 s sleep) plus the capture itself: a slow look on a loaded host passes the 3 s and the capsule is gone, so `lum(640,712)` is the face and not the capsule (gate-construct-pixels.sh:78 red). No product change.

**Repro.** Not red from real load here: PIX_LOAD=1 (4 busy loops per CPU, nproc 4) 11/11 green before the fix; 8 more captures alone at 16 loops also green (capture ~2.5 s after the key in each). Deterministic stand-in: PIX_LAG=1.5 (seconds before each look). Red line at 434e66c3: `FAIL: the capsule over the face stands clear of the taskbar`.

**Fix 05828b32.** The shot rests the pointer at the bottom (`act move 100 745`) and waits for the node (`ctl wait id:construct-hints`), so the capsule is held by state; the gate reads the glass at y 708 (the held set is the shorter `Super F1 All keys` and x 640 hit a keycap at y 712: 62 vs 60). Bars unchanged (<=60, >=100). New knobs: PIX_LOAD, PIX_LAG.

**After.** Whole pixels cell under PIX_LOAD=1: 10/10 PASS; PIX_LOAD=1 PIX_LAG=3: 3/3; PIX_LAG 0, 1.5, 4 alone: all PASS.

**Gate** (`merge-gate.sh --jobs 2`, detached, /tmp/merge-gate.EJanug): EXIT 1, the one red cell is `acts` (232 pass, 5 known red, 2 blocking), both in the construct hint of a face, not the pixels scripts:
- `FAIL: construct I6: in act 'co-42: the hint Keys of a face stays after 3 s and opens the key list by a click': on HEADLESS-1 'PANES' (construct-help-body-head-6) is 11 px, the guide wants 14 px`
- `FAIL: construct act:the hint Keys of a face names a key that opens the key list: ... RuntimeError: the hint keys names no key`
Not rerun. All other cells PASS (cube, pull, placed, construct-hud, construct-extra, matrix, hud, tiling-audit, xp-*). `pgrep -fa 'chora --headless'` empty.

## Review r1 (PM; 60 lines, e905195e..05828b32, test code only)
- Red first: 434e66c3 (PIX_LAG=1.5 red line) before 05828b32. Cause names hints.rs:32 and :139, no product change. ok.
- Fix waits on state (pointer low, `ctl wait id:construct-hints`); bars unchanged (<=60, >=100). ok.
- 10/10 under PIX_LOAD=1; PIX_LAG 0, 1.5, 3, 4 pass (both hint sets). ok.
- Gate: the two blocking acts reds are construct hint flakes that flake-hint d2634fe0 fixes (reports/flake-hint.md: "the hint keys names no key" 1 of 8 before); pix-capsule's base lacks it. Foreign, named. ok.
- pc-1 NIT: the rim check now compares y 705 with y 708 (3 px apart, was 7); a light line at 706-707 would pass. Next construct pixel round.
Verdict (computed): 0 MAJOR -> MERGE-OK 05828b32; joins t112 after flake-hint.
