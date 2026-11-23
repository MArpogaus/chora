## r6
Lane: cloud, branch claude/p2-r6-co at 86e619f3 (run only, no code changes). Step 0: unshallow, `cargo check --workspace` and `cargo build --workspace` green; `round.sh --help` ok; smoke `begin smoke` and `check.py end smoke` ("no contract for smoke", as r4 and r5). **Binaries of the round: sha 86e619f361633f5b4222cf1a2c9f909eae8332fd** (/tmp/persona-round/bin, one snapshot).
Setup: construct 1280x800, 2 slots (explorer and Max together, demo after), PERSONA_CHECK=1, helper transcript seeded (one agent entry) in all three sessions, one `begin` per goal and `check.py end` after it, one haiku Agent per goal in the same session. `round.sh probe max` started just before T5 (a first start before C2 was stopped and the session restarted, so the probe window is not in C2 or C3).
Caps written before the run (sum to the persona total, 45 actions and 20 looks): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 11/5, C3 12/5, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6 (looks 20). Used: explorer 17 actions, 10 looks; Max 31, 16; demo 31, 17. **No cap breached** (r4 and r5 slipped on looks). No retries past 3, no coaching. Every task of every persona ran (Max T5 and T3 included).
"Breaches" are those `check.py end` prints; the I6 rows of the xp face (tray clock under the exempt floor 3.0, title 2.15:1, taskbar button 3.86:1, menu frame) are the known xp contrast family (282, 485, 515 rows over the runs) and are counted separately.

### Explorer
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass (cube in a face) | 0 | no (row "A cube of desktops" enters face 0, Super+C x2 nothing visible) |
| T1 | **FAIL** no new focused window of construct; new s1 | 0 (r5: I3 x2) | yes (Start, Terminal; Super+Return nothing) |
| T2 | pass | 0 (r5: I3 x4) | yes (clicked the pill "Super F1 All keys"; the list opened) |

### Max
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C2 pull | pass (no pull) | 0 | no (terminal lost behind the xp desktop; no handle) |
| C3 two panes, fly, Tab | **FAIL** expected the focus with construct, holders ['xp'] | 0 | no (one floating foot, Super+Return nothing, no second terminal, no Tab) |
| T5 | pass (probe got no Super key) | 0 | yes (Super+t opened a terminal: the chord stayed with the desktop; the probe window has no title bar or mark) |
| T3 | **FAIL** the focus is with None, not back with construct | 0 | yes (Hello shown) |

### Demo watcher
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | 0 | no (plain desktop, no cube) |
| C2 | pass (no pull) | **I3 x5** "the focused s1 is no shown window of xp" | no (drag onto pager button 2: tooltip only, window stays) |
| T4 lock | pass (calls.log `swaylock -f` x2) | **I2 x2, I3 x4** (s2, s3 on VIRTUAL-2 no box; xp holds, core focuses nothing; one more s1 row) | no by its eyes (no lock screen) |

### Fix check against r5
| Row | Reached | Holds | Capture |
|---|---|---|---|
| explorer T1 (terminal in a face) | yes, persona reached (Start, Terminal) | persona yes; contract **still FAIL** (terminal owned by the xp child: gap or bug for triage) | persona-r6-explorer shots 008, actions 8 to 14 |
| explorer T2 key list by mouse in a face (co-42) | yes: the pill "Super F1 All keys" is there and a click opens the list | **pass** (contract pass, persona reached, r5 FAIL) | persona-r6-explorer shots 009, 010 |
| I3 double focus on clicks in the xp face (co-43, co-46) | Start, Terminal, pill, rows clicked in the face by all three personas | **holds**: 0 "construct and xp each say they hold the focus" in 79 actions (r5 6 + demo from its first click). New other I3 rows in the demo: focus left on a window that is not shown after the pager click (s1, 5 rows) and "xp holds sN, the core focuses nothing" (2 rows) | persona-r6-demo evidence 019 to 030, breaches.jsonl |
| Max C3 (two panes, fly, Tab) | no | **FAIL** (r5 contract pass): holders ['xp']; persona got no second pane (Super+Return does nothing, Start gives a floating foot); fly and Tab not reached | persona-r6-max shots 010, evidence 020 to 024 |

### triage.py (`python3 triage.py --format feedback/2026-11-23-p2r6co-*.md`: FORMAT OK; findings listed for the three files, 16 new, none MISSING UX)
```
FORMAT OK
  2026-11-23-p2r6co-demo.md: I2 and I3 on clicks of T4: xp shows s2 and s3 on VIRTUAL-2 with no box (0 days)
  2026-11-23-p2r6co-demo.md: I3 after a window is dragged onto a desktop button: the focus stays with a window that is not shown (0 days)
  2026-11-23-p2r6co-demo.md: xp face contrast rows (I6) (0 days)
  2026-11-23-p2r6co-explorer.md: xp face contrast rows (I6) (0 days)
  2026-11-23-p2r6co-max.md: xp face contrast rows (I6) (0 days)
  2026-11-23-p2r6co-max.md: C3 red: the focus is with xp, not construct (0 days)
  2026-11-23-p2r6co-explorer.md: T1 red: the terminal started from Start in the xp face is owned by xp, not by construct (0 days)
  2026-11-23-p2r6co-max.md: T3 red: after the helper answered the focus is with nobody (0 days)
  2026-11-23-p2r6co-demo.md: C2: dragging a window onto the pager button does not give it away (0 days)
  2026-11-23-p2r6co-demo.md: C1: the cube row shows a plain desktop (third time) (0 days)
  2026-11-23-p2r6co-demo.md: T4 lock: the stand-in is called twice but no lock screen is drawn (0 days)
  2026-11-23-p2r6co-explorer.md: C1: the row "A cube of desktops" enters face 0 with no overview (again) (0 days)
  2026-11-23-p2r6co-explorer.md: Super+Return does nothing on the face (0 days)
  2026-11-23-p2r6co-explorer.md: ux: key list was found (0 days)
  2026-11-23-p2r6co-max.md: C2: no way found to give the window away (0 days)
  2026-11-23-p2r6co-max.md: T5: the probe window has no title bar and no focus mark (0 days)
```

### Stop rule (p2-plan.md)
Round 6 is **not met** for construct: T1-T7 and C1-C3 are not all reached (C1 by no persona, C2 by none, C3 not, T4 lock not by eyes; contracts C3, T1, T3 red) and there are new check-backed candidates: contract C3 (focus with xp), contract T3 (focus None after the answer), I3/I2 on the pager drag and on Start Lock in an xp face (demo). Triage decides which are new BLOCK or MAJOR against co-42, co-43, co-46, co-47 (co-42, co-43, co-46 hold in this round). No ledger edit here.
