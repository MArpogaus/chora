# p2-r9-tiling (RC 2 persona round 9, tiling)

## r9

Lane: cloud, branch claude/p2-r9-tiling at 721be592 (session head t134, holds tiling-doors b1ba6743 and stray-super r3). **Binaries of the round: sha 721be5926a76dafaa3fe79d996c9b1f742aebe76** (`round.sh snapshot`, /tmp/persona-round/bin; no product or test code changed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` read; smoke (`round.sh begin smoke`, `check.py end smoke`: "no contract for smoke", expected; stopped clean). fuzzel 1.11.1 (ti-1 not in play). Sessions 1280x800, 2 slots (Kemal 1 + beginner, then ricer 1 tiling+xp + Kemal 2, then ricer 2), PERSONA_CHECK=1, helper transcript seeded (user "Hi", agent "Hello") per session, haiku personas via Agent, act.py only, one `begin` before each goal and `check.py end` after it, goals sent one at a time, no coaching.
Goal words: r8's goal texts are not in the repo (only the T7 text); I wrote the goals in the plain words of r8's report and feedback; **T7 verbatim: "Send the terminal over to the second look (the one with a start button)"**. L3 is worded "second workspace" as in r8 (it primes the T7 reading, as in r8). No goal names a row, key or desktop.
Caps written before the run (sum <= 45): Kemal T1 6, L1 10, T3 9 (25; T5 scripted by the driver, 4 more actions); beginner T1 5, T2 4, T4 5, T6 6 (20); ricer T1 6, L3 8, T7 10, T2 5, L2 12 (41). Used (act.py log): Kemal 1 21 own actions + 4 scripted, Kemal 2 19 + 4, beginner 15, ricer 1 25, ricer 2 20; no failed action, no retry past 3.
Slips: the beginner (after T1) and ricer 2 (after T1) each called a documentation tool once, outside "act.py only" (not Bash, no effect on the session, results unused). The driver did not count the waybar processes of the Kemal 1 and beginner sessions (see table). Captures: media/2026-11-25/persona-tiling-r9-{kemal1,kemal2,beginner,ricer1,ricer2} (shots, actions.jsonl, calls.log, evidence, breaches.jsonl), -hint (driver-only look at the chip), -checks (check.py end per task).

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal run 1 | T1 Super+Return | pass | 0 | yes |
| Kemal run 1 | L1 two terminals, focus, close one | pass | 0 | yes: Super+Return, click for focus, Super+q (unbound, no chip seen), Ctrl+d in the shell; key list not opened |
| Kemal run 1 | T3 helper | pass | 0 | persona: no ("Hello" is the fake agent's fixed answer, known; panel opened by the bar word "atlas · idle") |
| Kemal run 1 | T5 (scripted) | pass | 0 | n/a |
| Kemal run 2 | T1 | pass | 0 | yes |
| Kemal run 2 | L1 | pass | 0 | yes: Super+Right (workspace, ti-12/28), Super+1, click, Ctrl+d; no unbound chord, key list not opened |
| Kemal run 2 | T3 | pass | 0 | yes (orb click, typed, Return; "Hello") |
| Kemal run 2 | T5 (scripted) | pass | 0 | n/a |
| beginner | T1 | pass | 0 | yes (Super+Return, a key; not the Apps door) |
| beginner | T2 Keys button | pass | 0 | yes (2 actions) |
| beginner | T4 power | pass (open [], no power call) | 0 | yes (Super+Shift+E, Escape) |
| beginner | T6 frobnicate | pass (Nothing found row) | 0 | persona: no (read the row as a failure); Return pressed this time |
| ricer run 1 | T1 | pass | I6 xp only | yes |
| ricer run 1 | L3 workspace and back | pass | I6 xp only | yes |
| ricer run 1 | T7 tiling -> xp | **FAIL** no window changed look | I6 xp only | persona "reached" (tag 2, its reading); no door tried |
| ricer run 1 | T2 | pass | I6 xp only | yes (Keys click) |
| ricer run 1 | L2 gap and reload | pass | I6 xp only | yes (Super+s, :%s 12 -> 20, :w, Super+Ctrl+r) |
| ricer run 2 | T1, L3, T2 | pass | I6 xp only | yes |
| ricer run 2 | T7 | **FAIL** | I6 xp only | persona "reached (an assumption)"; 2 actions, no door |
| ricer run 2 | L2 | pass, **false green** (gap_inner still 12 on disk) | I6 xp only | no (clicked and typed in vim normal mode, 9 of 12 actions, no :w) |

Breaches outside known-red: Kemal 0 / 0, beginner 0, ricer 1 I6 x25, ricer 2 I6 x20, all the xp tray clock (3.27:1, twice 2.96:1). No I1, I2, I3, no tiling row.

### Noise check (Kemal and the ricer twice; both runs above)
Kemal: T1, L1, T3, T5 identical in contract across the two runs (all pass). L1 is green in both because both closed with Ctrl+d in the shell, never the close key or the key list; the run paths differ (run 1 Super+q, run 2 Super+Right). Ricer: T7 red in both runs, the same reading and the same quote; T1, L3, T2 green in both; L2 differs: run 1 knew vim (:%s, :w, reload) and passed, run 2 did not edit and the check wrongly passed. So the T7 red is not noise (4 rounds, 5 runs); the L2 spread is persona (editor knowledge) plus a harness false green.

### Fix check against r8
| Row | Result | Capture |
|---|---|---|
| ricer T7 (ti-55: window moved to xp) | **FAIL, third fix not seen: 2 of 2 runs red.** The bar word Looks is on screen in every capture and neither persona named or clicked it. Words: "I read 'the second look (the one with a start button)' as workspace 2 ... every picture shows a start button, so the wording was ambiguous" (run 1); run 2 the same reading, 2 actions | ricer1 shots 001, 006-007; ricer2 shots 002, 005; checks/ricer{1,2}-T7.txt |
| Kemal L1 (ti-56: after an unbound chord, does the persona open the key list) | **L1 contract pass x2, but the ti-56 fix is not seen: not reached.** Run 1 pressed Super+q once (unbound), looked 6 s later: no chip, key list not opened; run 2 pressed no unbound chord. Driver check on a clean session: no chip in three captures (key, 12 s gap, key; look after each). act.py puts 4.3 s between a key and the look (hint actions.jsonl) and the chip lasts 3 s, so no persona look can show it | kemal1 shots 005-006; hint shots 002-004, hint/actions.jsonl |
| helper row (ti-50: found from the launcher) | **pass (seen, not used)**: the beginner's Apps list after Super+R has "Chat with a helper (atlas)" as the first row; no persona clicked it (Kemal used the bar word and the orb) | beginner shot 006 |
| beginner T4 (ti-27 holds) | **pass**: key list of T2 open, power menu by Super+Shift+E, one Escape, `T4:power` pass, open [] | beginner shots 003-005, checks/beginner-T4.txt |
| one bar per session (sh-41) | **pass where counted**: Kemal 2 1 / 1, ricer 1 1 / 1, ricer 2 1 / 1, hint session 1 at the start; Kemal 1 and beginner not counted (driver slip); no waybar and no `chora --headless` left after `round.sh stop` | counts above |
| beginner T6 (ti-31 known) | **contract pass this round** (Return pressed, row "Nothing found for 'frobnicate'"), persona still reads it as not reached; ti-31/ti-49 stay known | beginner shots 007-008 |

### triage.py (the three r9 files only, run on a copy in a scratch dir; FORMAT OK)
```
== tiling / I6 (1)
  2026-11-25-p2r9ti-ricer.md: Xp I6: tray clock under the contrast bar (0 days)
== tiling / contract L2 (1)
  2026-11-25-p2r9ti-ricer.md: L2 contract is green when the persona changed nothing (run 2) (0 days)
== tiling / contract T7 (2)
  2026-11-25-p2r9ti-ricer.md: T7: "the second look" read as workspace 2 for the fourth round; the Looks door is not used (run 1) (0 days)
  2026-11-25-p2r9ti-ricer.md: T7 (run 2): same result, 2 actions, no door looked for (0 days)
== tiling / none (11)
  2026-11-25-p2r9ti-beginner.md: ux: Apps list: the helper row stands first, an unknown name gives an empty list, Nothing found only after Return (0 days)
  2026-11-25-p2r9ti-beginner.md: ux: the power menu preselects Lock (0 days)
  2026-11-25-p2r9ti-beginner.md: ux: a mouse-first persona starts with keys (0 days)
  2026-11-25-p2r9ti-beginner.md: ux: the key list is large and small (0 days)
  2026-11-25-p2r9ti-kemal.md: ti-56: the hint for an unbound Super chord cannot be seen by a persona; the key list is still not opened (0 days)
  2026-11-25-p2r9ti-kemal.md: ux: L1 passes by the shell, not by a close key (0 days)
  2026-11-25-p2r9ti-kemal.md: ux: the helper opens from the "atlas · idle" word or the Listening orb, one of two clicks on the orb did nothing (0 days)
  2026-11-25-p2r9ti-kemal.md: ux: locale warning in the first terminal line (0 days)
  2026-11-25-p2r9ti-ricer.md: ux: Super+s opens a modal editor; a click and typing do nothing (0 days)
  2026-11-25-p2r9ti-ricer.md: ux: Super+Shift+2 sends the window to tag 2 and leaves the view on tag 1 (0 days)
  2026-11-25-p2r9ti-ricer.md: ux: the bar word Looks is not read as "look" (0 days)
waiting 15, overdue 0, missing ux 0
OK
```
Feedback: feedback/2026-11-25-p2r9ti-{kemal,beginner,ricer}.md (each holds both runs where there are two).

### Stop rule (p2-plan.md)
**No: tiling does not meet round 9.** Not all tasks reached: ricer T7 (ti-55) red in both runs, the fourth round; the goal's "look" is still read as a workspace and the new bar word Looks is not read as the door. Kemal L1 passes by contract (both runs closed with Ctrl+d), but the ti-56 fix is unseen: the chip lasts 3 s and the persona's look comes 4.3 s or more after a key, so it is not observable by this driver; no persona opened the key list after a stray chord. Ricer 2 L2 not reached (vim). Holding: ti-50 (helper row first), ti-27 (T4), sh-41 where counted, T1, T2, T3, T5, T6 contracts, L3. No new check-backed product BLOCK or MAJOR: T7 is the open ti-55 (duplicate); the I6 rows are the xp tray clock (xp class); one new check finding: **L2 contract false green** (check.py verify_reloaded passes when only the config file digest changed by opening the editor; MAJOR-candidate for the harness, not the product). Open for the PM: (1) T7: three rounds of wording, doors and a bar word did not lead a persona to a door; the goal wording ("second look") or a door the persona sees in its first look (an explicit bar word naming another desktop) is a decision, not a fix; (2) ti-56 hint: show it longer than 4.3 s or on a persistent word to make it observable by a persona and a real person with slow eyes (3 s is short for a hint), or measure it by a check instead of a look; (3) fix verify_reloaded to compare the value of gap_inner, not the file digest; (4) Kemal's L1 now passes without the close key: the contract wants one window closed, so it cannot tell a shell exit from the close binding. No ledger edits.
