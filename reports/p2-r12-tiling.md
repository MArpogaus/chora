# p2-r12-tiling (RC 2 persona round 12, tiling)

## r12

Lane: cloud, branch claude/p2-r12-tiling at 3be27e32 (session head, t152 green; holds tiling-focus r3 d4fd349f). **Binaries of the round: sha 3be27e322542cafe6c66138d8e15687d2e71e2d2** (`round.sh snapshot`; no product or test code changed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` read; smoke on tiling+xp: `check.py end smoke` = "check.py: no contract for smoke", and `check.py end T7` right after = "task T7 not reached (no event since begin)" (named, not fixed); stopped clean. fuzzel 1.11.1 (ti-1 not in play). Sessions 1280x800 per output, 2 slots, PERSONA_CHECK=1, helper transcript seeded per session (user "Hi", agent "Hello"), haiku personas via Agent (one agent per persona run, goals sent one at a time, no coaching, caps as written), act.py only, one `begin` before each goal and `check.py end` after it (outputs in media/2026-11-26/persona-tiling-r12-checks). I did not audit the subagents' tool calls beyond their reports and act.py logs.
Layout: tiling alone for Kemal and the beginner; ricer tiling+xp (tiling HEADLESS-1, xp HEADLESS-2). Waybar count 3 per session at each start (6 with two sessions), none left after `round.sh stop`; `pgrep -fa 'chora --headless'` empty at the end.
Goal words: **T7 from the README "Tiling goals" word for word: "Send the terminal over to the xp look (the other desktop that is running)"**; the others r11's words. Order per README (ricer T1, T7, L3, T2, L2).
Caps written before the run: Kemal T1 6, L1 10, T3 9 (25; T5 scripted by the driver, Super+x and one look); beginner T1 5, T2 4, T4 5, T6 6 (20); ricer T1 6, T7 10, L3 8, T2 5, L2 12 (41). Used (act.py log): Kemal 1 23 persona actions, Kemal 2 22, beginner 17, ricer 1 22, ricer 2 29 (looks 11, 12, 9, 11, 16); no failed action, no retry past 3 (Kemal run 2 hit the cap in L1 and T3, ricer run 2 used 9 of 10 in T7). Every task of every persona ran.
Captures: media/2026-11-26/persona-tiling-r12-{kemal1,kemal2,beginner,ricer1,ricer2} (shots, actions.jsonl, calls.log, breaches.jsonl; ricer evidence trimmed to 3), media/2026-11-26/persona-tiling-r12-checks/*.txt.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal run 1 | T1 | pass | 0 | yes (Super+Return) |
| Kemal run 1 | L1 | pass | 0 | yes: Super+Return, Super+Right (workspace 2, ti-12/28), Super+1, click, Ctrl+d (shell exit); no unbound chord |
| Kemal run 1 | T3 | **FAIL** helper ends ['working'] | 0 | persona: no (did not look after sending); Listening pill click nothing, Keys list, Super+A, typed; later capture shows "Hello" |
| Kemal run 1 | T5 (scripted) | pass | 0 | n/a |
| Kemal run 2 | T1 | pass | 0 | yes |
| Kemal run 2 | L1 | **FAIL** expected 1 window closed, got [] | 0 | persona: no (cap 10; Super+Right, Super+1, Super+Tab; "would have tried Super+q or Super+w") |
| Kemal run 2 | T3 | **FAIL** helper ends ['working'] | 0 | persona: no (bar word "atlas · idle" found, first answer "Hello", second question sent at the cap) |
| Kemal run 2 | T5 (scripted) | pass | 0 | n/a |
| beginner | T1 | pass | 0 | yes (Apps, click Foot) |
| beginner | T2 | pass | 0 | yes (Keys button) |
| beginner | T4:power | pass | 0 | yes (power icon, Escape) |
| beginner | T6 | pass (Nothing found row) | 0 | persona: no (reads "Nothing found" as failure) |
| ricer run 1 | T1 | pass | I6 xp only | yes |
| ricer run 1 | T7 | **pass** | I6 xp only | persona: reached, not verified |
| ricer run 1 | L3 | **pass** | I6 xp only | yes (tag 2, tag 1) |
| ricer run 1 | T2 | pass | I6 xp only | yes (Keys click, left open) |
| ricer run 1 | L2 | **FAIL** gap_inner still 12 | I6 xp only | persona: no (editor ignored click and BackSpace; stopped after 4 of 12 actions) |
| ricer run 2 | T1 | pass | I6 xp only | yes |
| ricer run 2 | T7 | **pass** | I6 xp only | persona: reached, not verified (also clicked Looks: xp a second time, Escape) |
| ricer run 2 | L3 | **FAIL** expected the focus with tiling, holders [] | I6 xp, **I3 x9** ("the core focuses tiling of tiling, no look says it holds the focus") | yes (tag 2, tag 1) |
| ricer run 2 | T2 | pass | I6 xp only | yes (Keys, Escape) |
| ricer run 2 | L2 | **FAIL** gap_inner still 12 | I6 xp only | persona: no (typed "20" into a modal editor, shown as a pending count; stopped at 8 of 12) |

Breaches outside known-red: Kemal 0 / 0, beginner 0, ricer 1 I6 x38 (xp tray clock 3.27, xp-task "foot" 3.86), ricer 2 I6 x52 and I3 x9, the I3 only in L3 of run 2. No I1, I2.

### Noise check (Kemal and the ricer twice; both runs above)
Kemal: T1, T5 pass in both. L1 differs only by the cap (run 1 closed by the shell, run 2 never closed); Super+Right cost one action in both. T3 red in both with the same line; in both the personas found the helper (run 1 by the Keys list and Super+A, run 2 by the bar word) and the last send was the last action, so the contract ran with a question in flight; red in 2 of 2 is stable for this driver order. Ricer: T1, T7, T2 pass in both, T7 green 4 of 4 over r11 and r12; L3 passes in run 1 and fails in run 2 with a **new** line (holders [] and I3, not the r11 holder xp), so L3 is not a stable pass; L2 red in both, same line, by the editor (no focus line).

### Fix check against r11
| Row | Result | Capture |
|---|---|---|
| ricer L3 after T7 (ti-57: focus back with tiling) | **half: pass in run 1, FAIL in run 2** (holders [] and I3 x9: the core names tiling focused, no look says it holds the focus; the r11 holder xp is gone in both runs) | checks/ricer1-L3.txt, ricer2-L3.txt, ricer2 breaches.jsonl |
| ricer L2 after T7 | **FAIL 2 of 2**, no focus line now: gap_inner unchanged because the persona could not edit the modal editor (run 1: clicks and BackSpace ignored; run 2: "20" taken as a pending count) | checks/ricer{1,2}-L2.txt, ricer1 shot 011, ricer2 shot 016 |
| ricer T7 (ti-55) | **pass 2 of 2** (contract); persona still "reached, not verified" | checks/ricer{1,2}-T7.txt |
| Kemal T3 (helper found in both runs, or where the persona looked) | **found 2 of 2**: run 1 clicked the Listening pill (nothing), the Keys button, read "Super+A: Chat with a helper", pressed Super+A; run 2 clicked the bar word "atlas · idle". Contract red in both ("helper ends working", last send at the cap, see noise check) | kemal1 shots 008-011, kemal2 shots 012-013, checks/kemal{1,2}-T3.txt |
| Kemal L1 (ti-56: chip within 8 s after an unbound Super chord; does the persona open the key list) | **chip pass (driver)**: Super+x then one look in both sessions shows "Super+X does nothing here. Super+F1 shows all keys." Persona side **not reached**: no persona pressed an unbound chord; run 1 opened the key list by the Keys button for T3 | kemal1 shot 012, kemal2 shot 013 |
| helper row (ti-50) | **pass**: the Apps list has "Chat with a helper" first; beginner named it, did not click | beginner shot 002 |
| beginner T4 (ti-27) | **pass**: power icon, Escape, T4:power pass | beginner shots 006-007 |
| one bar per output (sh-41) | **pass**: 3 waybar processes per session at each start, tiling bar on HEADLESS-1 and xp taskbar on its own output in the ricer sessions, none left after stop | start counts, ricer1 shot 001 |
| beginner T6 (ti-31 known) | contract pass, persona still reads "Nothing found" as not reached (ti-31, ti-49 known) | beginner shots 008-009 |
| rtd4-1: tag 1 shows no xp taskbar | **pass**: ricer 1 first look and ricer 2 look after L3 (tag 1 empty) show the tiling bar and the dark wallpaper, no xp taskbar | ricer1 shot 001, ricer2 shot 010 |

### triage.py (the three r12 files only, run on a copy in a scratch dir; FORMAT OK)
```
== tiling / I6 (1)
  2026-11-26-p2r12ti-ricer.md: Xp I6: tray clock and "foot" task buttons under the contrast bar (0 days)
== tiling / contract L1 (1)
  2026-11-26-p2r12ti-kemal.md: L1: run 2 closes nothing within 10 actions (0 days)
== tiling / contract L2 (1)
  2026-11-26-p2r12ti-ricer.md: L2: the persona cannot change the value in the settings editor (red 2 of 2, no focus line this time) (0 days)
== tiling / contract L3 (1)
  2026-11-26-p2r12ti-ricer.md: L3: after T7 the focus comes back with tiling in run 1, but in run 2 no look holds it (I3 x9) (0 days)
== tiling / contract T3 (1)
  2026-11-26-p2r12ti-kemal.md: T3: helper contract red in both runs although both personas typed the question into the chat panel (0 days)
== tiling / none (7)
  2026-11-26-p2r12ti-beginner.md: ux: ti-50 holds, the Apps list has "Chat with a helper" as its first entry (0 days)
  2026-11-26-p2r12ti-beginner.md: ux: an unknown program name: "Nothing found" only after Return, the launcher stays open, the box looks empty (0 days)
  2026-11-26-p2r12ti-beginner.md: ux: the key list is dense and small at 800 px; "Tag" and "Tile" are not explained; the power menu preselects Lock (0 days)
  2026-11-26-p2r12ti-kemal.md: ti-56: chip seen by the driver; no persona pressed an unbound chord (0 days)
  2026-11-26-p2r12ti-kemal.md: ux: Super+Right switches workspace instead of moving focus; the Listening pill does nothing on click (0 days)
  2026-11-26-p2r12ti-ricer.md: T7 passes again; the persona cannot see the xp desktop (0 days)
  2026-11-26-p2r12ti-ricer.md: ux: the key list stays open until the next goal's first action; the settings editor is a raw vim-style file (0 days)
waiting 12, overdue 0, missing ux 0
OK
```
Feedback: feedback/2026-11-26-p2r12ti-{kemal,beginner,ricer}.md (each holds both runs where there are two).

### Stop rule (p2-plan.md)
**No: tiling does not meet round 12.** Not all tasks reached with a green contract: Kemal T3 red 2 of 2 and L1 red in run 2, ricer L3 red in run 2, ricer L2 red 2 of 2. New check-backed finding: ricer run 2 L3, after T7, "expected the focus with tiling, holders []" with I3 x9 ("the core focuses tiling of tiling, no look says it holds the focus"), a MAJOR candidate for the PM to classify (ti-57 half fixed: run 1 green, and the r11 holder xp is gone in both runs; cause open: run 2 had an extra Looks: xp menu opened and shut in T7, a reading). L2 now fails without a focus line: the persona cannot edit the modal settings editor (a UX reading, check is the contract; a ti-12-class item or harness backlog for the PM). T3 red is a driver timing effect as much as a product one (the last send was the last action; one capture shows the question 40 s later still unanswered in run 2, driver reading). Fix checks holding: ti-55 T7 4 of 4, ti-56 chip (driver), ti-50, ti-27, sh-41, rtd4-1, T1, T2, T4, T5, T6 contracts. Open for the PM: (1) classify L3 run 2 (I3 holders []), (2) L2 editor, (3) T3 timing in the driver, (4) ti-56 persona side still not observed, (5) T7 stays "reached, not verified" for a persona. No ledger edits.
