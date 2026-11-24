# p2-r6-tiling (RC 2 persona round 6, tiling)

## r6

Lane: cloud, branch claude/p2-r6-tiling at cae62f50 (session head t122, tiling-disc ca165f7b merged). **Binaries of the round: sha cae62f507f21eaf08d1a7c525563cc5498000d16** (`round.sh snapshot`, /tmp/persona-round/bin). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` read; smoke (`round.sh begin smoke`, `check.py end smoke`: "no contract for smoke", expected). fuzzel **1.11.1** on this host (above 1.11: ti-1 not in play, a click on a launcher row starts the program). Sessions 1280x800, 2 slots (Kemal and beginner tiling together, then ricer tiling+xp), PERSONA_CHECK=1, helper transcript seeded (user "Hi", agent "Hello") before each persona, haiku personas via Agent, act.py only, one `begin` before each goal and `check.py end` after it, goals one at a time by SendMessage, no coaching.
Caps written before the run (sum <= 45 each): Kemal T1 6, L1 10, T3 9 (25, T5 scripted 5 more); beginner T1 5, T2 4, T4 5, T6 6 (20); ricer T1 6, L3 8, T7 10, T2 5, L2 12 (41). Used (act.py log): Kemal 25 (20 own + 5 scripted), 12 looks; beginner 19, 10 looks; ricer 27, 12 looks. No failed action, no retry past 3.
Captures: media/2026-11-24/persona-tiling-r6-{kemal,beginner,ricer} (shots, actions.jsonl, breaches.jsonl, calls.log, evidence), -checks (check.py end output per task), -bar2 (doubled bar). Feedback: feedback/2026-11-24-p2r6ti-{kemal,beginner,ricer}.md. Slips: none (no persona Bash outside act.py; no sandbox block). The T4 stand-ins ran only wpctl/systemctl probes (no power call).

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal | T1 Super+Return | pass | 0 | yes (3 actions) |
| Kemal | L1 two terminals, focus, close one | pass | 0 | yes: Super+K found in the key list (Windows: Alt+Tab / Super+J / Super+K, Move focus); opened the list with a click on Keys |
| Kemal | T3 Super+a, ask | pass | 0 | yes (answer Hello = fake agent, known) |
| Kemal | T5 (scripted, probe, Escape, Super+x/q/w) | pass | 0 | n/a |
| beginner | T1 Apps click, Foot | pass | 0 | yes (fuzzel 1.11.1) |
| beginner | T2 Keys button | pass | 0 at the end; I6 x2 'Tags' 1.00:1 on the way | yes (3 actions) |
| beginner | T4 power cancel | FAIL: expected open [], got ['tiling-keys'] (key list of T2 still open; power menu closed, no systemctl call) | I6 x2 (same) | yes |
| beginner | T6 frobnicate | pass | 0 | no: the list is empty and silent while typing; `Nothing found` only after Return (ti-31 known, fuzzel) |
| ricer | T1 Super+Return | pass | I6 xp only | yes |
| ricer | L3 workspace and back | pass | I6 xp only | yes (Super+2, Super+1) |
| ricer | T7 tiling -> xp | FAIL: no window changed look (persona made a tag move) | I6 xp only | no: did not open the list, read "look" as a workspace, stopped at 5 of 10 actions |
| ricer | T2 key list | pass | I6 xp only | yes (2 actions) |
| ricer | L2 gap and reload | pass | I6 xp only | yes: Super+S, vim, gap_outer 12 -> 20, :wq, Super+Ctrl+r; margin grew; the persona did not read the file back |

Breaches outside known-red: Kemal 0; beginner I6 x2 (key list 'Tags' 1.00:1); ricer I6 x59, all xp (tray clock 27, My Computer 12, Recycle Bin 20; xp class). No I2, no I1, no I3, no I5.

### Fix check against r5
| Row | Result | Capture |
|---|---|---|
| Kemal L1 (focus key found from the key list?) | **pass**: Move focus is the first row of Windows; Super+K used, focus moved, one terminal closed; contract pass | kemal shots 003-007, checks/kemal-L1.txt |
| beginner T6 (launcher text before Return) | **not fixed, known**: before Return the list is empty and says nothing; "Nothing found" after Return only (ti-31 backlog: fuzzel has no empty-list text). Contract pass, persona not reached | beginner shots 009-010 |
| ricer L2 (settings door "Edit settings", gap changed and reloaded) | **pass**: found in the key list (`Super+S: edit settings`), vim on chora-tiling.yaml, gap_outer 20, reload; contract pass (r5 FAIL). The door in the bar or launcher row was not tried by the persona | ricer shots 010-012, checks/ricer-L2.txt |
| ricer T7 (handover to xp, harness no longer vacuous) | **harness fixed, task not reached**: a tag move now prints `FAIL: no window changed look` (r5 pass, vacuous); the persona never gave a window to xp (did not open the list first; "look" read as workspace) | checks/ricer-T7.txt, ricer shots 006-008 |
| r5 open point: ricer capture folder after `round.sh stop` of a tiling+xp session | **present**: media/2026-11-24/persona-tiling-r6-ricer holds shots (27 actions), actions.jsonl, breaches.jsonl, calls.log, evidence | keep/ricer |
| beginner T1 (r5 fuzzel 1.9.2) | pass on fuzzel 1.11.1 | beginner shots 003-004 |

New in r6 (not in the r5 list): the top bar is drawn twice in all three sessions (and a fresh fourth one), two waybar processes on the same config, one with ppid 1 and one left running after `round.sh stop` (killed by hand); the smoke session of the same binaries and r5 had one bar. Cause not found; suspect the session.sh waybar beside the core's panel unit since a4badd8e (CHORA_FLAVOR_DIR) or a start race; not checked, no code touched. Filed `check: none` (verify) in the Kemal file with capture (bar2/dbl-001.png, ps.txt). It is the kind of find that wants a cell (count of waybar processes = 1) before a fix.

### triage.py (the three r6 files; FORMAT OK; the script is triage.py at the root of claude/session-log, there is no tools/triage.py)
```
== tiling / I6 (4): p2r6ti-beginner Key list 'Tags' 1.00:1 (ti-30 class); p2r6ti-ricer xp tray clock and icon labels (xp class)
== tiling / contract T4:power (2): p2r6ti-beginner key list left open from T2 (ti-27, verify)
== tiling / contract T7 (5): p2r6ti-ricer give window to the other look not used, T7 red
== tiling / none (65): beginner x4 (launcher silent before Return, Foot entries, Keys panel text, double bar), kemal x3 (focus mark, locale warning, focus key found), ricer x3 (Edit settings, 'look' wording, double bar)
== tiling / none (verify: two waybar processes in the session) (1): p2r6ti-kemal The top bar is drawn twice
```

### Stop rule (p2-plan.md)
**No: tiling does not meet round 6.** Not all tasks reached: ricer T7 (the persona did not find the give key: the contract is red and honest now) and beginner T6 (ti-31, fuzzel). The other 11 tasks reached with a green contract, except beginner T4 whose red is the key list left open by the persona's own earlier goal (ti-27). No new check-backed BLOCK or MAJOR: the double bar has `check: none` (verify, needs a cell first), the I6 rows are ti-30 / xp classes, T7 red is a persona-reach point, not a check-backed product red. The L1, L2, T1 and harness rows of r5 now hold. Open for the PM: (1) double bar and orphan waybar (verify; check suspect a4badd8e in session.sh); (2) T7 reach: the row "give window to a look" is first in the list but a persona does not open the list for it and "look" is not a user word (ux, medium); (3) the key list 'Tags' 1.00:1 row survives tiling-disc (ti-30 closed too early?). No ledger edits.
