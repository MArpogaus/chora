# p2-r7-tiling (RC 2 persona round 7, tiling)

## r7

Lane: cloud, branch claude/p2-r7-tiling at ad0093ba (tiling-reach 4e257c06 merged with session head ce866b0d). **Binaries of the round: sha ad0093ba3b072e3704c682758223d872f47c3eb0** (`round.sh snapshot`, /tmp/persona-round/bin). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` read; smoke (`round.sh begin smoke`, `check.py end smoke`: "no contract for smoke", expected; one waybar). fuzzel **1.11.1** on this host (ti-1 not in play). Sessions 1280x800, 2 slots (Kemal and beginner tiling together, then ricer tiling+xp), PERSONA_CHECK=1, helper transcript seeded (user "Hi", agent "Hello") before each persona, haiku personas via Agent, act.py only, one `begin` before each goal and `check.py end` after it, goals one at a time by SendMessage, no coaching.
Goal words: r6's words were not in the repo (the r6 report only has the T7 text); I wrote the goals in the same plain words from the r6 feedback steps and used the r6 T7 text verbatim ("send the terminal over to the second look (the one with a start button)"). Not word for word for the other goals; no goal names a row, a key or a desktop name.
Caps written before the run (sum <= 45 each): Kemal T1 6, L1 10, T3 9 (25, T5 scripted 5 more); beginner T1 5, T2 4, T4 5, T6 6 (20); ricer T1 6, L3 8, T7 10, T2 5, L2 12 (41). Used (act.py log): Kemal 25 (21 own + 4 scripted keys), 12 looks; beginner 17, 9 looks; ricer 26, 13 looks. No failed action, no retry past 3.
Captures: media/2026-11-24/persona-tiling-r7-{kemal,beginner,ricer} (shots, actions.jsonl, breaches.jsonl, calls.log, evidence), -checks (check.py end output per task). Feedback: feedback/2026-11-24-p2r7ti-{kemal,beginner,ricer}.md. Slips: none (no persona Bash outside act.py; no sandbox block). The T4 stand-ins ran only wpctl/systemctl probes and the fuzzel stand-in rows (no lock, no power call). End of run: `pgrep -fa 'chora --headless'` empty, no waybar left.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal | T1 Super+Return | pass | 0 | yes (2 actions + look) |
| Kemal | L1 two terminals, focus, close one | FAIL: expected 1 window closed, got [] | 0 | no: second terminal opened; Super+Right (workspace) and Super+l (split) tried; the key list not opened; stopped at 9 of 10 |
| Kemal | T3 Super+a, ask | FAIL: the helper never worked | 0 | no: Super+space, typed "atlas" (nothing found), Escape, Super+a on the 6th action, cap 9 spent before typing |
| Kemal | T5 (scripted, probe, Escape, Super+x/q/w) | pass | 0 | n/a |
| beginner | T1 Apps click, Foot | pass | 0 | yes (fuzzel 1.11.1) |
| beginner | T2 Keys button | pass | 0 at the end | yes (2 actions + look) |
| beginner | T4 power | FAIL: expected open [], got ['tiling-keys'] (key list of T2 open; power menu opened and left open by the persona; no systemctl or loginctl call) | I6 x2 here | yes (its reading: "leave without choosing") |
| beginner | T6 frobnicate | pass | I6 x114 (key list rows, see below) | no by the persona: `Nothing found for 'frobnicate'` is the first row after Return; it expected a program or a notice (ti-31 known) |
| ricer | T1 Super+Return | pass | I6 xp only | yes |
| ricer | L3 workspace and back | pass | I6 xp only | yes (Super+2, Super+1) |
| ricer | T7 tiling -> xp | FAIL: no window changed look | I6 xp only | no (persona says reached): "second look" read as workspace 2, Super+Shift+2, Super+2; neither the Apps list nor the key list opened |
| ricer | T2 key list | pass | I6 xp only | yes (click Keys; read "Super+Alt+2-4: move window to another desktop" unasked) |
| ricer | L2 gap and reload | pass | I6 xp only | yes: Super+S, :%s gap_inner 12 -> 24, :w, Super+Ctrl+R; gap between windows grew about 15 to 25 px |

Breaches outside known-red: Kemal 0; beginner I6 x116 (all key list: 'Tags' and about 28 row labels or keys 1.00:1, 'Move focus' 1.34:1, 'Window before' 3.77:1; the list stayed open under the launcher and the power menu); ricer I6 x59, all xp (tray clock 26, Recycle Bin 21, My Computer 12). No I1, no I2, no I3, no I5.

### Fix check against r6
| Row | Result | Capture |
|---|---|---|
| ricer T7 (window moved to xp by the launcher row or Super+Alt key) | **not reached, fail**: the persona did not look for a door. Its words: "the second look read as workspace 2, the second screen I captured earlier; the taskbar start button is in every look, so I did not use it to pick a screen". The rows "Move window to the ... desktop" and the key row were not on its path; at T2 it read `Super+Alt+2-4 moves a window to another desktop` in the key list. The words work once the list is open; the goal still does not lead there. Launcher row not tried | ricer shots 006-009, checks/ricer-T7.txt |
| one bar (waybar processes per session at start and end, exactly 1) | **pass**: Kemal 1 / 1, beginner 1 / 1, ricer 1 / 1 (counted by XDG_RUNTIME_DIR of /proc/PID/environ); none left after `round.sh stop` (smoke also 1) | ps counts in this report; bar visible in kemal shot 001 |
| beginner T6 (ti-31 known) | **not fixed, known**: contract pass; the list is silent while typing, `Nothing found for "frobnicate"` after Return (fuzzel stand-in call row in calls.log); persona answered not reached | beginner shots 008-009, calls.log |
| beginner T4 (ti-27) | **fail again**: the key list of T2 is still open at the end of T4 (`open ['tiling-keys']`); this time the persona also left the power menu open (my goal said "leave them without choosing anything"); no power call ran | beginner shots 006-007, checks/beginner-T4:power.txt |

New in r7: Kemal L1 and T3 went red (r6 pass). Both are persona path and cap: it did not open the key list for L1 (r6 did, from the Keys button) and it spent its T3 cap in the Apps list looking for "atlas". No product red behind them (no breach, no focus or lost-window invariant). The beginner I6 count on the open key list grew from 2 to 116 lines (rows read 1.00:1 while a popup is over the list); cause not checked (the rows may be read through the popup), verify before any ti-30 claim.

### triage.py (the three r7 files only, run on a copy; FORMAT OK; the script is triage.py at the root of claude/session-log, there is no tools/triage.py)
```
== tiling / I6 (2)
  2026-11-24-p2r7ti-beginner.md: Key list rows under a popup read 1.00:1 (I6, many rows, up from 2 in r6) (0 days)
  2026-11-24-p2r7ti-ricer.md: Xp I6: tray clock and desktop icon labels under the contrast bar (0 days)
== tiling / contract L1 (1)
  2026-11-24-p2r7ti-kemal.md: L1: the persona did not find the focus key and closed nothing, contract red (0 days)
== tiling / contract T3 (1)
  2026-11-24-p2r7ti-kemal.md: T3: the helper question was never sent, the launcher has no helper row for the word "atlas" (0 days)
== tiling / contract T4:power (1)
  2026-11-24-p2r7ti-beginner.md: Power menu opened and left open, key list of the goal before still open: T4 contract red (0 days)
== tiling / contract T7 (1)
  2026-11-24-p2r7ti-ricer.md: T7: "the second look" read as workspace 2, no handover, the new rows not tried (0 days)
== tiling / none (7)
  2026-11-24-p2r7ti-beginner.md: Launcher: "Nothing found" appears only after Return and the persona read it as no answer (0 days)
  2026-11-24-p2r7ti-beginner.md: ux: the key list says "lock, log out, screenshot", the power menu has Restart and Shut down too (0 days)
  2026-11-24-p2r7ti-kemal.md: ux: the launcher has no row a person finds for "helper" or its name (0 days)
  2026-11-24-p2r7ti-kemal.md: ux: locale warning in the first terminal line (0 days)
  2026-11-24-p2r7ti-ricer.md: The goal word "look" is the persona's word for a workspace, the same as in r6 (0 days)
  2026-11-24-p2r7ti-ricer.md: Super+Shift+2 sends the window to tag 2 and leaves the view on tag 1 (0 days)
  2026-11-24-p2r7ti-ricer.md: ux: the editor saves with no word on the status line (0 days)
waiting 13, overdue 0, missing ux 0
OK
```

### Stop rule (p2-plan.md)
**No: tiling does not meet round 7.** Not all tasks reached: ricer T7 (second round: the goal word "look" is read as a workspace and the persona never opens a list that holds the new rows), Kemal L1 and T3 (reach and cap, r6 passed), beginner T6 (ti-31, fuzzel). beginner T4 is red by the key list left open (ti-27). One bar holds (ti-47 pass in all three sessions). No new check-backed product BLOCK or MAJOR: the contract reds L1, T3, T4, T7 are persona reach and left-open state, with no invariant behind them; the I6 rows are ti-30 / xp classes (the beginner count of 116 wants a verify). Open for the PM: (1) T7 reach is not fixed by the rename: a task in the words "second look" does not find the door; the xp and tiling bars read as one desktop (ux, medium); (2) the same persona and the same goals gave L1 pass in r6 and red in r7, so a haiku persona's path varies; a 2nd run of Kemal L1 would tell noise from a regression; (3) key list left open at T4 (ti-27) and I6 on it. No ledger edits.
