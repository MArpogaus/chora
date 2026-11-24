# p2-r8-tiling (RC 2 persona round 8, tiling)

## r8

Lane: cloud, branch claude/p2-r8-tiling at bb4c07d9 (tiling-calm r2 head, MERGE-OK). **Binaries of the round: sha bb4c07d9cde2b20c1385087458a46c4696ecc6b0** (`round.sh snapshot`, /tmp/persona-round/bin; no product or test code changed). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` read; smoke (`round.sh begin smoke`, `check.py end smoke`: "no contract for smoke", expected; one waybar). fuzzel 1.11.1 (ti-1 not in play). Sessions 1280x800, 2 slots (Kemal run 1 + beginner, then ricer tiling+xp + Kemal run 2), PERSONA_CHECK=1, helper transcript seeded (user "Hi", agent "Hello") per session, haiku personas via Agent, act.py only, one `begin` before each goal and `check.py end` after it, goals one at a time by SendMessage, no coaching. No persona Bash outside act.py; no slip. The T5 keys of Kemal (probe, Escape, Super+x/q/w) were scripted by the driver as in r7.
Goal words: r7's goal texts are not in the repo (only the T7 text, in feedback); I wrote the goals in the same plain words as r7's report and feedback give them, T7 verbatim: "Send the terminal over to the second look (the one with a start button)". No goal names a row, key or desktop. L3 was worded "second workspace" as in r7's steps; it primed the T7 reading (see ricer).
Caps written before the run (sum <= 45 each): Kemal T1 6, L1 10, T3 9 (25, T5 scripted 6 more); beginner T1 5, T2 4, T4 5, T6 6 (20); ricer T1 6, L3 8, T7 10, T2 5, L2 12 (41). Used (act.py log): Kemal run 1 26 actions / 13 looks, run 2 27 / 13, beginner 19 / 11, ricer 25 / 12. No failed action, no retry past 3.
Captures: media/2026-11-24/persona-tiling-r8-{kemal1,kemal2,beginner,ricer} (shots, actions.jsonl, calls.log, evidence, breaches.jsonl for ricer), media/2026-11-24/persona-tiling-r8-checks (check.py end output per task, calls). Feedback: feedback/2026-11-24-p2r8ti-{kemal,beginner,ricer}.md (Kemal's file holds both runs). End of run: `pgrep -fa 'chora --headless'` empty, no waybar left.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal run 1 | T1 Super+Return | pass | 0 | yes (2 actions + look) |
| Kemal run 1 | L1 two terminals, focus, close one | FAIL: expected 1 window closed, got [] | 0 | no: Super+Right (workspace), Super+l (split); Super+q on the last action, never looked at; key list not opened |
| Kemal run 1 | T3 ask the helper | pass | 0 | yes (purple dot, typed, Return; answer "Hello" = fake agent) |
| Kemal run 1 | T5 (scripted) | pass | 0 | n/a |
| Kemal run 2 | T1 | pass | 0 | yes |
| Kemal run 2 | L1 | FAIL: expected 1 window closed, got [] | 0 | no: Super+Right, Super+Left, Super+j (focus worked), Super+q (unbound), cap; key list not opened |
| Kemal run 2 | T3 | pass | 0 | yes (dot, click entry, type, Return; "Hello") |
| Kemal run 2 | T5 (scripted) | pass | 0 | n/a |
| beginner | T1 Apps click, Foot | pass | 0 | yes |
| beginner | T2 Keys button | pass | 0 | yes (3 actions; read the list) |
| beginner | T4 power | **pass** (open [], no power call) | 0 | yes (power icon, Escape; "the Keys overlay was still open, it had closed by the time the Power menu was dismissed") |
| beginner | T6 frobnicate | FAIL: no notice, no "Nothing found" row | 0 | no: typed in Apps, empty silent list, no Return (ti-31 known; r7 persona pressed Return) |
| ricer | T1 Super+Return | pass | I6 xp only | yes |
| ricer | L3 workspace and back | pass | I6 xp only | yes (Super+2, Super+1) |
| ricer | T7 tiling -> xp | FAIL: no window changed look | I6 xp only | no (persona says reached): Super+Shift+2, Super+2; no list opened |
| ricer | T2 key list | pass | I6 xp only | yes (Keys click, 2 actions) |
| ricer | L2 gap and reload | pass | I6 xp only | yes (Super+s, :%s gap_inner 12 -> 24, :wq, Super+Ctrl+r) |

Breaches outside known-red: Kemal 0 / 0, beginner 0 (r7: 116), ricer I6 x25, all the xp tray clock (3.2:1). No I1, I2, I3, I5, no tiling row.

### Noise check (Kemal twice; r7 open point 2)
L1 red in both r8 runs, red in r7, pass in r6. Different keys each time (run 1: Super+Right, Super+l, Super+q; run 2: Super+Right, Super+Left, Super+j, Super+q), never the Keys button; run 2 found the focus key alone, then closed with the unbound Super+q (the list says Super+Shift+C). So L1 red is not noise: this persona's path is to guess keys, and r6's pass came from opening the key list. T1, T3, T5 pass in both runs. No product red behind L1 (no invariant, focus moves with Super+j).

### Fix check against r7
| Row | Result | Capture |
|---|---|---|
| beginner T4 (ti-27: no key list left behind the power menu) | **pass**: key list of T2 open, power menu opened and closed by one Escape, `check.py end T4:power` pass; the persona saw the list gone | beg shots 005-008, checks/beg-T4.txt |
| Kemal T3 | **pass** in both runs (r7 red) | kemal1 shots 008-011, kemal2 011, checks |
| helper row in the launcher (ti-50) | **not exercised by a persona**: no run opened the Apps list for the helper (both used the purple dot). The beginner's Apps list (nine rows, Dark look .. LibreOffice Draw) shows no helper row; per gate-tiling-apps the row is appended last, so it sits below the visible rows. Whether it is reachable by scrolling or by typing "help" was not tried: open | beg shot 010 |
| ricer T7 (window moved to xp) | **fail, not reached, third round**: Super+Shift+2 then Super+2. Persona's words: "I read 'the second look (the one with a start button)' as the second screenshot of the previous goal (004), which showed workspace 2 with the start button ... the wording was ambiguous". It did not open the Apps list or the key list | ricer shots 006-008, checks/ricer-T7.txt |
| one bar (waybar per session, start / end) | **pass**: kemal1 1 / 1, beginner 1 / 1, ricer 1 / 1, kemal2 1 / 1 (counted by XDG_RUNTIME_DIR of /proc/PID/environ); none left after `round.sh stop` (smoke too) | counts above |
| beginner T6 (ti-31 known) | **not fixed, known**: contract red, the persona stopped at the empty list without Return | beg shots 010-011 |

### triage.py (the three r8 files only, run on a copy; FORMAT OK)
```
FORMAT OK
== tiling / I6 (1)
  2026-11-24-p2r8ti-ricer.md: Xp I6: tray clock under the contrast bar (0 days)
== tiling / contract L1 (1)
  2026-11-24-p2r8ti-kemal.md: L1: second terminal opens, the persona closes nothing; contract red again (r7 red, r6 pass) (0 days)
== tiling / contract T6 (1)
  2026-11-24-p2r8ti-beginner.md: T6: the persona typed the name and never pressed Return; contract red (ti-31 known) (0 days)
== tiling / contract T7 (1)
  2026-11-24-p2r8ti-ricer.md: T7: "the second look" read as workspace 2 for the third round; no handover, no door tried (0 days)
== tiling / none (9)
  2026-11-24-p2r8ti-beginner.md: ux: the Apps list is empty and silent for a name it does not know (0 days)
  2026-11-24-p2r8ti-beginner.md: ux: the key list says "lock, log out, screenshot", the power menu has Restart and Shut down too (0 days)
  2026-11-24-p2r8ti-beginner.md: ux: the key list is much bigger than the Keys button suggests, and tiny (0 days)
  2026-11-24-p2r8ti-kemal.md: ux: a keyboard user guesses close and focus keys and never opens the key list (0 days)
  2026-11-24-p2r8ti-kemal.md: ux: helper reached by the purple dot or Super+a, not by the launcher (0 days)
  2026-11-24-p2r8ti-kemal.md: ux: locale warning in the first terminal line (0 days)
  2026-11-24-p2r8ti-ricer.md: ux: Super+Shift+2 sends the window to tag 2 and leaves the view on tag 1 (0 days)
  2026-11-24-p2r8ti-ricer.md: ux: the bar and the taskbar read as one desktop; "look" is not a word on screen (0 days)
  2026-11-24-p2r8ti-ricer.md: ux: the visible effect of a gap change needs two windows (0 days)
waiting 13, overdue 0, missing ux 0
OK
```

### Stop rule (p2-plan.md)
**No: tiling does not meet round 8.** Not all tasks reached: ricer T7 (third round: the goal's "look" is read as a workspace; the persona opens no list), Kemal L1 (twice, persona path: it never opens the key list), beginner T6 (ti-31, known, plus a persona that did not press Return). Fixed and holding: ti-27 (T4 pass, I6 x116 gone), ti-47 (one bar in all four sessions), Kemal T3. No new check-backed product BLOCK or MAJOR: the reds are persona reach with no invariant behind them; I6 is the xp tray clock only (xp class). Open for the PM: (1) T7 reach is a goal-wording / discoverability problem three rounds running: either the goal says "the Windows-style desktop" or a door must be visible from the bar (ux, medium; the same words are in r6, r7, r8); (2) L1: a keyboard persona does not open the key list; a hint after a stray Super key is a feature question, not a bug; (3) helper row in the Apps list: not seen below the nine rows, a persona run or a scripted look is needed to close ti-50. No ledger edits.
