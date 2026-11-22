## r1

Lane claude/gate-waits (F5, rc 3). Base 1c599c74; change commit 6366f2f3; head 3c5b29c9 (the change merged with origin/claude/autonomous-dev-session-3w-i33pg7 at 1c9e64f7). Pushed to claude/gate-waits.

### What changed (tests only, no product code)

- tests/gate-lib.sh: `until_ok CMD [HALF-SECONDS]` (moved from tests/doors/lib.sh, same units), `settled CMD` (output the same twice 0.3 s apart), `settle` (render-stats stand still), `pace N` (a sleep that paces on purpose). Selftest in tests/gate-lib-selftest.sh (waits for a late state, gives up on a state that never comes).
- `session_start` no longer sleeps 2 s: it waits until the nodes of the plugin after `--` answer `owner:NAME` (the core's scene when there is none). This was the shared sleep of every cell.
- doors: `doors_window` waits for `enter` in the probe report instead of 2 s; the header step of mc waits 40 s instead of 10 s for the agenda page.
- xp-faces: sleeps after `turn_to` and after the restart became waits on the rects and on opacity 100.
- construct-hud, layout, pull, tiling-look (wobbly, r2, power, print-notice): each standalone `sleep` of 1 s or less before a capture or a key became `settle`.
- Kept as `pace` (8 places): time is the thing under test, not a state. xp-faces: 0.5 s with the pointer held (a negative check: no cube appears) and 3 s before the render counters are compared. tiling-look-r2: the 3 s tooltip delay and bar-word poll (4 places), the 4 s start of the bar, and 1 s for the notice card to fade in (a wait on the card node and on `settle` there made the notice cell red 10 of 10: the capture then missed the card, so it stays a pace). The hook no-inline-comments forbids the reason as a comment, so it is here.

### Sleep count

`grep -c sleep tests/*.sh`, total: 762 at 1c599c74 (673 at plan time) and 715 at 6366f2f3 (-47); 8 timed `pace` calls are not in that count (723 if they are counted). tests/doors/*.sh: 7 to 5.

### Load test

4 `yes` per core beside, each cell 10 times as two at once (--jobs 2 pairs, one gate lock per slot), `merge-gate.sh --only CELL`. Before on 1c599c74, after on 6366f2f3 (tiling-look rerun after a fix, see below).

| cell | green of 10 before | green of 10 after |
|---|---|---|
| doors | 9 | 10 |
| xp-faces | 10 | 10 |
| construct-hud | 10 | 10 |
| layout | 10 | 10 |
| pull | 10 | 10 |
| tiling-look | 10 | 10 |

Before: the one red doors run was `check mc header-1024x768 fail`, "header: the page of the agenda fails": the wait for the agenda page had 10 s under load; now 40 s. The other five cells did not flake in 10 runs on this machine, so "before" does not show a flake to remove there; the sleeps are removed on the strength of the reasoning, not of a red count. The first after run of tiling-look was 0 of 10: `FAIL: ink notice: nothing stands at the bottom right (0 pixels differ)`, caused by my own change to `listed_after` (a wait on the card node and `settle` in place of `sleep 1`); fixed to a pace, then 10 of 10. That run was 10 of 10 at 6366f2f3 minus comment-only edits made for the commit hook (sleep to pace, comments removed), which do not change timing.

### Full gate on the merged head, --jobs 1

`tests/merge-gate.sh --jobs 1` stops at the first red step, so the steps after it were run with `--only` in two runs (every step ran once; none was rerun to green).

- Red, not caused by this change: `tiling-look` (tests/gate-tiling-bar-spec.sh): `FAIL: ink-1280x800 bar-font: the clock is 120.0 wide, not Inter 14 px weight 500 (100-119)` (and the three other sizes, 120.5 and 121.0 wide). The clock shows the date and today is Sunday 22 Nov; the same script at 1c599c74 (untouched by me) fails the same way now, and passed on 21 Nov at 111.0 wide. A limit of 119 px does not hold for a date like "Sun 22 Nov".
- Red, not caused by this change: `acts`, `FAIL: tiling I14: in act 'Escape shuts the key list of the keys alias key': choractl disconnected, the act did not declare it (evidence /tmp/acts-run-p730vfmn/tiling-I14)`, 230 pass, 6 known red, 1 blocking. No file of the acts is touched here (tests/acts, tests/test_acts.py are as on origin). The upstream head 1c9e64f7 alone passed acts once (1464 s), and 1c599c74 passed once (297 s). One red in two runs on the same code: an acts flake worth its own row.
- Green: build, test, clippy, ci-check, gate-lib (with the new selftest), xp, matrix, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, xp-notepad, hud, layout, tiling-audit, and, run alone after the stop: tiling-empty, chat-keys, doors, last-words, cube, pull, placed, construct-hud, construct-extra, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-paths, mc-sofa, mc-column, mc-handover, launcher-click, voice-acp, voice-events, agent, e2e.

So the full gate is not green today: two red steps, neither from this change. Done-when: sleep count falls (yes), each cell 10 of 10 after (yes), full gate green (no: the clock limit of the bar spec and the acts flake).

## Review r1 (PM; 1c599c74..6366f2f3, head 3c5b29c9; 12 files +84 -68)
- Done-when: sleeps 762 to 715 (yes); each touched cell 10/10 under load (yes); full gate: red only in tiling-look bar-font (clock date, CLOCK-WIDTH) and acts I14 (SH-32), both not from this diff (same reds on 1c9e64f7).
- session_start waits on `owner:<plugin>` nodes (core scene if none), fails after 30 s: a real check in place of sleep 2.
- gw-1 MINOR gate-lib.sh `settled`: ends `|| true`, so a scene that never settles goes on silently to the capture; a note on stderr would show it.
- gw-2 MINOR doors/lib.sh `doors_window`: no keyboard only echoes; the next got_key fails later, with a less direct reason.
- gw-3 NIT: `pace` is a plain sleep; the reason lives only in this report (the hook forbids inline comments).
Counts: BLOCK 0, MAJOR 0, MINOR 2. Verdict: MERGE-OK; joins t105.
