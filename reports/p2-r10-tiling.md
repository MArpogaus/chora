# p2-r10-tiling (RC 2 persona round 10, tiling)

## r10

Lane: cloud, branch claude/p2-r10-tiling at e39448ad (session head t139, holds tiling-doors r3 35e31662 and tiling-minor r1). **Binaries of the round: sha e39448ad0f21ad9c9dac9d6e7c0b9aad5a7ee898** (`round.sh snapshot`, /tmp/persona-round/bin; no product or test code changed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` read; smoke (`round.sh begin smoke`, `check.py end smoke`: "no contract for smoke", expected; stopped clean). fuzzel 1.11.1 (ti-1 not in play). Sessions 1280x800, 2 slots (Kemal 1 + beginner, then ricer 1 tiling+xp + Kemal 2, then ricer 2), PERSONA_CHECK=1, helper transcript seeded (user "Hi", agent "Hello") per session, haiku personas via Agent, act.py only, one `begin` before each goal and `check.py end` after it (outputs in media/2026-11-25/persona-tiling-r10-checks), goals sent one at a time, no coaching.
Goal words: **T7 verbatim from the README "Tiling goals": "Send the terminal over to the xp look (the other desktop that is running)"**. The other goals: r9's goal texts are not in the repo (r9 report says the same); I used the plain words of r9's report and feedback, no goal names a row, key or desktop: Kemal T1 "Open a terminal.", L1 "Get two terminals open next to each other, move over to the other one, then close one of them.", T3 "Ask the helper what time it is."; beginner T1 "Open a program so you can type commands (a terminal).", T2 "Find out which keyboard shortcuts this desktop has.", T4 "Open the power menu (the one for shutting down or locking) and then close it again without choosing anything.", T6 "Try to start a program called frobnicate."; ricer T1 "Open a terminal.", L3 "Go to a second workspace and then come back to the first one.", T2 as the beginner, L2 "Change the gap between windows from 12 to 20 in the settings and make the desktop use it."
Caps written before the run (sum <= 45): Kemal T1 6, L1 10, T3 9 (25; T5 scripted by the driver, 4 more actions); beginner T1 5, T2 4, T4 5, T6 6 (20); ricer T1 6, L3 8, T7 10, T2 5, L2 12 (41). Used (act.py log): Kemal 1 26 (20 own + 4 scripted T5 + 2 driver hint probe), Kemal 2 25 (21 + 4), beginner 19, ricer 1 29, ricer 2 32 (30 + 2 driver hint probe before the first goal); no failed action, no retry past 3. Every task of every persona ran. No persona Bash outside act.py is known; I did not audit the tool calls of the subagents beyond their reports.
Captures: media/2026-11-25/persona-tiling-r10-{kemal1,kemal2,beginner,ricer1,ricer2} (shots, actions.jsonl, calls.log, breaches.jsonl; ricer evidence trimmed to 3 captures), -hint (driver-only look at the chip), -checks (check.py end per task, the T5 probe reports). Waybar: one process per session counted at every start (3, then 6, ...: 1 per session); no `chora --headless` left after `round.sh stop`.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal run 1 | T1 Super+Return | pass | 0 | yes |
| Kemal run 1 | L1 two terminals, focus, close one | pass | 0 | yes: Super+Return, click for focus, `exit` + Return in the shell; no unbound chord, key list not opened |
| Kemal run 1 | T3 helper | pass | 0 | persona: no ("Hello" is the fake agent's fixed answer, known); Keys button, then Super+a |
| Kemal run 1 | T5 (scripted) | pass | 0 | n/a |
| Kemal run 2 | T1 | pass | 0 | yes |
| Kemal run 2 | L1 | pass | 0 | yes: Super+Right (workspace, ti-12/28), Super+1, click, Ctrl+d; no unbound chord, key list not opened |
| Kemal run 2 | T3 | pass | 0 | persona: no ("Hello"); bar word "atlas · idle" |
| Kemal run 2 | T5 (scripted) | pass | 0 | n/a |
| beginner | T1 | pass | 0 | yes (the Apps door, click Foot: not a key this time) |
| beginner | T2 Keys button | pass | 0 | yes (3 actions) |
| beginner | T4 power | pass (open [], no power call) | 0 | yes (Escape for the open list, power icon, Escape) |
| beginner | T6 frobnicate | pass (Nothing found row) | 0 | persona: no (read the row as a failure) |
| ricer run 1 | T1 | pass | I6 xp only | yes |
| ricer run 1 | L3 workspace and back | pass | I6 xp only | yes |
| ricer run 1 | T7 tiling -> xp | **FAIL** no window changed look | I6 xp only | persona "reached" (tag 2); no door tried |
| ricer run 1 | T2 | pass | I6 xp only | yes (Keys click) |
| ricer run 1 | L2 gap and reload | pass (gap_inner 20 on disk) | I6 xp only | persona: no (could not read the value) |
| ricer run 2 | T1, L3, T2 | pass | I6 xp only | yes |
| ricer run 2 | T7 | **FAIL** | I6 xp only | persona "reached" (tag 2); no door tried |
| ricer run 2 | L2 | pass (gap_inner 20 on disk) | I6 xp only | persona: no (steps 10-12 sent blind) |

Breaches outside known-red: Kemal 0 / 0, beginner 0, ricer 1 I6 x35, ricer 2 I6 x44, all the xp desktop (tray clock 3.27/3.21/2.96/2.93, pager tabs 4.14/3.95, one icon 4.20). No I1, I2, I3, no tiling row.

### Noise check (Kemal and the ricer twice; both runs above)
Kemal: T1, L1, T3, T5 identical in contract across the two runs (all pass); L1 closed by the shell in both (exit / Ctrl+d), never the close key; neither run pressed an unbound chord (run 2 used Super+Right again). T3 not reached in both (known fake answer). Ricer: T1, L3, T2, L2 green in both and T7 red in both with the same reading (workspace 2); L2 now passes in both with the value 20 on disk (r9 run 2's false green did not recur because the persona changed the value). So the T7 red is not noise (five rounds, 7 runs).

### Fix check against r9
| Row | Result | Capture |
|---|---|---|
| ricer T7 (ti-55: window moved to xp; did the persona read `Looks: xp` as the door) | **FAIL, 2 of 2 runs red, the fifth round.** `Looks: xp` is in every first capture and the README goal says "the xp look (the other desktop that is running)". Run 1: "the top bar shows 'Looks: xp'. Workspace 2 is the XP-look desktop I visited before ... I took 'the other desktop that is running' to mean workspace 2"; run 2: "sent the focused window to workspace 2", never named the bar word. The persona reads the word but ties it to workspace 2 (the L3 goal, which shows tag 2 with the xp wallpaper and taskbar, primes it again); neither clicked the word or opened the key list before T7 | ricer1 shots 005-008, ricer2 shots 008-010, checks/ricer{1,2}-T7.txt |
| Kemal L1 (ti-56: after an unbound Super chord a look within 8 s shows the hint; does the persona open the key list) | **L1 contract pass x2; ti-56 half seen.** Chip on the live frame: driver, clean session, no persona: Super+q then a look 0.3 s later and Super+x then a look 3 s later both show "Super+Q/X does nothing here. Super+F1 shows all keys." (the 8 s fix works). Persona side not observed: no persona pressed an unbound chord in 2 runs. Driver finding: with the checker on, act.py returns from an unbound key only after 9.7 s (a bound key 3.4-3.9 s; seen on Super+q/x/z in both Kemal runs), so a persona look after an unbound key lands 1.7 s after the 8 s chip is gone | hint shots 001-002, kemal1/kemal2 actions.jsonl |
| helper row (ti-50) | **pass**: Apps list from the Apps button has "Chat with a helper (atlas)" as the first, preselected row (beginner T1 shot 002, ricer 2 T1 shot 003); no persona clicked it | beginner shot 002 |
| beginner T4 (ti-27) | **pass**: key list from T2 still open, Escape closed it, power icon, Escape, `T4:power` pass, open [] | beginner shots 004-007, checks/beginner-T4:power.txt |
| one bar per session (sh-41) | **pass**: 1 waybar per session counted at every start, none left after `round.sh stop` | start counts |
| beginner T6 (ti-31 known) | contract pass, persona still reads "Nothing found" as not reached; ti-31/ti-49 stay known | beginner shots 008-009 |

### triage.py (the three r10 files only, run on a copy in a scratch dir; FORMAT OK)
```
== tiling / I6 (1)
  2026-11-25-p2r10ti-ricer.md: Xp I6: tray clock, pager tabs and an icon under the contrast bar (0 days)
== tiling / contract T7 (1)
  2026-11-25-p2r10ti-ricer.md: T7: "the xp look" still read as workspace 2; the bar word "Looks: xp" is read but not as the door (0 days)
== tiling / none (10)
  2026-11-25-p2r10ti-beginner.md: ux: T1 by the Apps door this time (mouse); the helper row stands first and is preselected (0 days)
  2026-11-25-p2r10ti-beginner.md: ux: the key list stays open until Escape (ti-27 known); the power menu preselects Lock (0 days)
  2026-11-25-p2r10ti-beginner.md: ux: an unknown program name: "Nothing found" only after Return, read as a failed start (0 days)
  2026-11-25-p2r10ti-beginner.md: ux: the key list is dense and small at 800x500 (0 days)
  2026-11-25-p2r10ti-kemal.md: ti-56: the hint chip shows on a clean session, but no persona pressed an unbound chord, and the checker holds the key action 9.7 s (0 days)
  2026-11-25-p2r10ti-kemal.md: ux: L1 passes by the shell, not by a close key (both runs) (0 days)
  2026-11-25-p2r10ti-kemal.md: ux: helper: reached by the key list (run 1) and the bar word (run 2); one click on the Listening badge does nothing (0 days)
  2026-11-25-p2r10ti-kemal.md: ux: locale warning in the first terminal line (0 days)
  2026-11-25-p2r10ti-ricer.md: ux: L2 passes and the file holds 20, but the persona cannot see it (0 days)
  2026-11-25-p2r10ti-ricer.md: ux: Super+Shift+2 sends the window to tag 2 and the view stays on tag 1 (0 days)
waiting 12, overdue 0, missing ux 0
OK
```
Feedback: feedback/2026-11-25-p2r10ti-{kemal,beginner,ricer}.md (each holds both runs where there are two).

### Stop rule (p2-plan.md)
**No: tiling does not meet round 10.** Not all tasks reached: ricer T7 (ti-55) red in both runs, the fifth round; the bar word `Looks: xp` and the "xp look" goal word are read, but as workspace 2. ti-56: the chip is correct on the live frame (8 s), but no persona pressed an unbound chord, and the checker holds an unbound key 9.7 s, longer than the chip, so a persona look cannot see it (a harness gap: the observation, not the product). Holding: ti-50, ti-27, sh-41, T1, T2, T3, T4, T5, T6, L1, L2, L3 contracts (T3 and T6 persona-side "not reached" are known). No new check-backed product BLOCK or MAJOR: T7 is the open ti-55 (duplicate); the I6 rows are the xp class (pager tabs 4.14/3.95 and icon 4.20 are new element names, xp, LOW). Open for the PM: (1) ti-55: five rounds of wording, doors and a bar word; the persona ties the look to workspace 2 after the L3 goal (the xp taskbar shows on every tag), so the goal order or a door on the bar that is not a tag is a decision, not a fix; (2) the ti-56 observation: let act.py return from a key before the checker waits (or show the chip longer than the check's quiet wait) so a persona look can fall inside the 8 s; (3) Kemal's L1 passes by the shell, not by the close key (r9 note stands). No ledger edits.
