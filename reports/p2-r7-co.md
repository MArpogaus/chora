## r7
Lane: cloud, branch claude/p2-r7-co at bc153929 (run only, no code changes). Step 0: unshallow, `cargo check --workspace` and `cargo build --workspace` green; `round.sh --help` ok; smoke `begin` + `check.py end smoke` on construct ("no contract for smoke"). The all-looks `round.sh smoke` showed one I6 on xp (tray clock 3.27:1, known family) and left the xp session up; I stopped it by hand. **Binaries of the round: sha bc153929e70c7316af5ecd2877e2b814214a373a** (/tmp/persona-round/bin, one snapshot).
Setup: construct 1280x800, 2 slots (explorer and Max together, demo after), PERSONA_CHECK=1, helper transcript seeded (one agent entry) in all three sessions, one `begin` per goal and `check.py end` after it, one haiku Agent per goal. Probe started before T5 (`round.sh probe max`).
Caps written before the run (/tmp/persona-round/caps.txt; sum to 45 actions, 20 looks): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 11/5, C3 12/5, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6. Used: explorer 14 actions, 8 looks; Max 34, 19; demo 31, 18 (actions 32 to 96 of its file are my hand runs). **No cap breached**, no retries past 3, no coaching. Every task of every persona ran.
Non-I6 breaches: **none in any persona run** (I6 rows: explorer 234, Max 554, demo 506 incl. hand runs, the known xp contrast family). Captures: media/2026-11-23/persona-r7-{explorer,max,demo}/ (demo: shots of the persona part only).

### Explorer
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass (cube in a face) | 0 | no (row "A cube of desktops" shows a plain desktop, Super+C nothing; 4th round) |
| T1 | pass | 0 | yes (Start, Terminal) |
| T2 | pass | 0 | yes (pill "Super F1 All keys") |

### Max
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C2 | pass (no pull) | 0 | no (terminal lost behind the xp desktop after Super+C) |
| C3 | pass | 0 | no (Super+Return does nothing; one terminal; fly and Tab not reached) |
| T5 | pass (probe got no Super key) | 0 | no by its eyes (probe window has no title, named chora-probe; Super+t opened a terminal: chord stayed with the desktop) |
| T3 | pass (focus back) | 0 | yes (Hello, then typing reached the chat box) |

### Demo watcher
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | 0 | no (plain desktop, 4th round) |
| C2 | pass (no pull) | 0 (r6: I3 x5) | no (drag onto pager button 2: window stays, tooltip only) |
| T4 lock | pass (calls.log `swaylock -f` x2) | 0 (r6: I2 x2, I3 x4) | no by its eyes (no lock window seen, ran out of looks before the key) |

### Fix check against r6 (co-focus r1 to r3)
| Row | Result | Capture / state dump |
|---|---|---|
| co-48 explorer T1 (terminal from Start in the xp face holds the focus) | **pass** (contract T1 pass; r6 FAIL), persona reached | persona-r7-explorer shots 006 |
| co-48 Max C3 | **pass** contract (r6 FAIL, holders xp); persona did not get a second pane (Super+Return nothing), so two panes by the persona not reached | persona-r7-max shots 010 |
| co-49 Max T3 (focus back after the helper answers) | **pass** (contract T3 pass, r6 FAIL focus None); persona reached, typing reaches the chat box | persona-r7-max shots 019 |
| co-50 demo C2 (keys after a drag onto a pager button; I3 rows 0) | persona: contract pass, **I3 rows 0** (r6 x5); the persona pressed no key after the drag, so the key itself is covered only by the hand runs | persona-r7-demo shots 010-012 |
| co-51 demo T4 (Start, Lock; next key after the lock window closes; I2, I3 rows 0) | persona: contract pass, I2 and I3 rows 0 in the persona part (r6 I2 x2, I3 x4); persona did not get to the key. Hand runs: see below | persona-r7-demo shots 013-018 |

No fail, so no state dump line.

### co-50 and co-51 by hand, 5x each (driver, demo session, act.py with the checker on; scripts /tmp/persona-round/hand.sh)
- co-50 (Start, Terminal, drag the title onto pager button 2, key a): **5/5 with I3 rows 0 and no other non-I6 row**, contract C2 pass each time (r6 rate 5/5 red). Caveat: I read the end state through the checker only; the windows were not closed between runs.
- co-51 (Start, Lock, wait 12 s, key a): I3 rows 0 in **5/5**, contract T4:lock pass 5/5, `swaylock -f` called each time. **I2 appeared in 2/5** (runs 1 and 3: "xp shows s9 / s11 on VIRTUAL-1, which has no box of a size there", at the Lock click). Not clean: the session held the windows moved by the five co-50 runs before it, so the row may come from those (confound), but I2 is the invariant co-51 names. For triage: reproduce co-51 on a fresh session.

### triage.py (`python3 triage.py --format feedback/2026-11-23-p2r7co-*.md`: FORMAT OK; the findings of the three files, 14 new, none MISSING UX)
```
construct / I2:  p2r7co-demo: I2 after Start, Lock in the xp face (hand runs 2 of 5)
construct / I6:  p2r7co-demo, -explorer, -max: xp face contrast rows (I6)
construct / contract T1:  p2r7co-explorer: T1 reached: Start, Terminal
construct / contract T2:  p2r7co-explorer: ux: key list was found
construct / contract T3:  p2r7co-max: ux: helper chat reached through the LISTENING pill only
construct / contract T4:lock:  p2r7co-demo: T4 lock: Lock closes the menu and no lock window is drawn
construct / contract T5:  p2r7co-max: T5: the probe window is not recognisable as "Probe window"
construct / none:  p2r7co-demo: C1 the cube row shows a plain desktop (fourth time)
                   p2r7co-demo: C2 dragging a terminal onto the pager button does not move it
                   p2r7co-explorer: C1 the row "A cube of desktops" enters face 0 with no overview (fourth time)
                   p2r7co-max: C2 the terminal is lost behind the xp desktop after Super+C
                   p2r7co-max: C3 Super+Return opens nothing (again)
(whole inbox: waiting 310, overdue 266, missing ux 0)
```

### Stop rule (p2-plan.md)
Round 7 is **not met** for construct: all contracts are green (co-48, co-49, co-50 by contract and by the 5x hand runs hold; r6 had 3 red), but T1-T7 and C1-C3 are not all reached by the personas: C1 (explorer, demo), C2 (Max, demo), C3 (Max), T5 and T4 lock by eyes are not. New check-backed BLOCK or MAJOR: none from the persona runs; one candidate, I2 on Start, Lock in 2 of 5 hand runs (co-51 not clean, confounded by earlier windows). The reach gaps are UX rows with check none (C1 cube row, Super+Return on a face, pager drag, lock window not seen). No ledger edit here.
