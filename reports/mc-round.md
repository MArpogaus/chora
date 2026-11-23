# mc-round

## r1

Branch claude/mc-round, base 42585079, head 9ed1fbc8, merged with fresh
origin/claude/autonomous-dev-session-3w-i33pg7 (already contained, no merge
commit). Commits, test before fix:

| id | red test | fix |
|---|---|---|
| FLK-MC-SCALE | af02a893 | d3caf20f |
| MH-2 | bf23d49d, cell 69856416 | 97efc46a |
| MC-HEAD | a8825dfd | 043af297, 9ed1fbc8 (cell c59cfc6b) |
| MH-1 | none, closed | none |
| lint ratchet (my new allow, moved) | gate red | 40241d14 |

### FLK-MC-SCALE
Cause: the write animation of a shown scene ends in `written`
(lifecycle.rs:104). It sends one `effects: []` update per panel, 300 in one
`apply`, 377 to 477 ms on a debug build (about 1.3 ms a command in the core).
The loop of mc is single threaded: it holds the next call behind that event.
Timing split (temporary log, removed): `settings_changed` itself takes 20 to
45 ms (name_theme 20 to 45, grabs 0.3); the first `settings.set` after
`scene.show` of 300 panels took 543 to 654 ms, the next 141 to 176 ms; the
event `AnimationDone mc-s-many-panels` took 447 to 477 ms. Under `yes` x4 the
sum reached the 5000 ms of the gate (6240 ms in t109d).
Fix (show.rs `write`, `page_edge`): a panel whose top is below the text area
of the page gets no write effect; its end then updates the few panels that
show, not 300. No limit moved.
Red: `assertion left == right failed: the last is far below the page
left: 1 right: 0` (a_panel_below_the_page_gets_no_write_effect, af02a893).
After, gate-mc-scale.sh under `yes` x4, 10/10, write time 182 to 288 ms
(before 570 to 950 ms); theme 2712 to 3236 ms (not in scope, but the
nearest to 5000 ms), scene.show again 2000 to 2260 ms.

### MH-2
The mc-56 commit hid the doors under the open chat because the chat box ran
to the foot of the column. Now `door_items` keeps them and the box of the
chat ends one baseline above the foot (region.rs `stack`, `foot_top`).
Red (unit, bf23d49d): `assertion left == right failed: the doors stay
left: 0 right: 5`. Cell in gate-mc-column.sh (69856416): the door Power lies
below the chat box and a click on the hit of the Volume door reaches mc with
the chat still open; red on the build before the fix `FAIL: no door of the
column beside the open chat`, green 5/5 in light and dark.

### MC-HEAD
Capture at 1024x768: reports/mc-round-head-1024x768.png (before the fix).
The head was cut: "1  AGEN…" beside "SUNDAY 22 NOVEMBER  22:10" (header 416
px: head 103 + gap 28.5 + clock 286 do not fit), and "7  CHORA-…" over a
window (head 165, agents 56 and clock 286 in a header of 520).
Fix (region.rs `line`, fit.rs `clock_gives_way`, `shorten_front`): when the
end of a header line takes more than the room left by head and middle, the
clock drops its leading words (weekday first, the time stays); head and
agents keep their letters.
Red (a8825dfd, gate-mc-column.sh `head_at 1024x768`): `FAIL: the running head
is cut at 1024x768: 1  AGEN…|AGENDA|`. Green 5/5 after 9ed1fbc8.
The doors gate cell `header-1024x768` asserted the date whole and failed after
the first fix; tests/doors/mc.sh (c59cfc6b) now allows a date without its
weekday and fails on a head with an ellipsis ("the running head is cut:
7  CHORA-…" was its red on the first fix). Green after 9ed1fbc8.

### MH-1
Closed with evidence, no repo change. The product commit that moved the dark
accent 204 120 102 to 216 152 138 is 10eb3a7f (2026-11-12, "dark Listening at
7:1": plugins/chora-mc/hud/hud.yaml accent and state-dark.svg). The check at
gate-mc-host.sh:158 was stale for ten days and 00406cd8 (11-22) set it to the
product colour; :233 is a card-keys check, not a colour. With hud.yaml and
state-dark.svg from 10eb3a7f^ the gate fails: `FAIL: dark: the state dot is
204 120 102, not the accent of the theme 216 152 138`; on the head it passes.
A comment naming 10eb3a7f in the script is refused by the no-inline-comments
hook, so it stays here.

### Gate
`tests/merge-gate.sh --jobs 2` (full, detached) on 9ed1fbc8: all steps PASS
except `tiling-windows` (93 s): `FAIL: the layout word took 291 ms to follow
the change` (the 250 ms bar latency of chora-tiling under two jobs; log
/tmp/merge-gate.cZMthe/tiling-windows.log). Not mine: no tiling file is in the
diff. Not rerun. mc-scale, mc-column, doors, ci-check (lint-ratchet 360)
PASS. An earlier full run on 043af297 had lint-ratchet red (361: a second
`allow`, fixed in 40241d14) and `doors` red (above, fixed).

## Review r1 (PM; 181 lines, 7 files, 42585079..9ed1fbc8)
- Red first for FLK-MC-SCALE (af02a893), MH-2 (bf23d49d), MC-HEAD (a8825dfd); MH-1 closed with product commit 10eb3a7f. ok.
- FLK-MC-SCALE (show.rs): no write effect below the text area; 10/10 under `yes` x4, 182-288 ms. ok.
- MH-2 (region.rs): the chat box ends one baseline above the foot; the doors stay; cell 5/5 light and dark. ok.
- MC-HEAD (fit.rs clock_gives_way): the clock drops leading words; the head keeps its letters. ok. The doors cell now accepts a date without its weekday: a narrower clock, not a lowered bar (the head check got stricter).
- mr-1 NIT: clock_gives_way sets the clock width to the share of the room, but shorten_front cuts whole words, so the kept text can be wider than that width; with only the time left it can still touch the head. Known issues; the 1024x768 cell passes.
- Gate: all PASS but tiling-windows "the layout word took 291 ms to follow the change" (foreign; new FLK-TILING-LAYOUT in backlog).
Verdict: MERGE-OK (0 MAJOR); joins t112.

## r2

- merge 4f64e247 (origin 86e619f3 into 9ed1fbc8), resolve 8ba700f5.
- red after merge: plugins/chora-mc/src/main.rs:1693 the_doors_leave_while_the_chat_is_open, left: 5 right: 0.
- resolve: doors stay (MH-2); door_items doc says so; that test removed (MH-2's test supersedes).
- chora-mc 195 passed, 0 failed. Gate: full merge-gate --jobs 2, 60 cells PASS, EXIT 0 (acts and mc cells green). No headless chora left.

## Review r2 (PM, 2-file diff 4f64e247..8ba700f5)
- Brief done criteria: merge and resolve commits named; red line pasted; chora-mc 195/195; full gate 60 PASS EXIT 0. Gap: gate-mc-column 5/5 light and dark not reported (the full gate ran it once green); not a MAJOR.
- The removed test asserted the I6 rule the PM decision overrides (MH-2 holds); door_items doc matches the code.
- Verdict (computed): 0 MAJOR, MERGE-OK 8ba700f5; joins t113.
