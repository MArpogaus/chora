## r5
Lane: cloud, branch claude/p2-r5-co at 892d035c, run only (no code changes). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` ok; smoke `round.sh begin smoke` took its begin, `check.py end smoke` says "no contract for smoke" (as r4). **Binaries of the round: sha 892d035cf67beb7406c1941afb839e928049b122**, snapshot /tmp/persona-round/bin.
Setup: round.sh snapshot once, construct 1280x800, 2 slots (explorer and Max together, demo after Max), PERSONA_CHECK=1, helper transcript seeded (one agent entry), `round.sh probe max`, one `begin` per goal and `check.py end` after it (one haiku Agent per goal, same session). Limits: actions 21, 44, 25 (cap 45); looks 12, **22** (cap 20: breached by the driver again, the per-goal caps of C2 and C3 added up to more than the persona total; r4 same slip), 14. No retries past 3, no coaching. "Breaches" below are those that `check.py end` counts (outside known-red.toml; the I6 rows of the xp face are listed there).
**Not run: Max T5 and T3** (his action cap was spent by C2 and C3; not restarted rather than breaching the cap further).

### Explorer (21 actions, 12 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | 0 (r4: I7 x4) | no (row goes into face 0, no overview seen; Super+C, pager nothing) |
| T1 | **FAIL** no new focused construct window (the terminal is owned by the xp child) | I3 x2 | yes (Start, Terminal in the face) |
| T2 | **FAIL** key list never open; `xp-dlg-help` open | I3 x4 | by its eyes yes (xp Help and Support, Keyboard shortcuts); not the construct list |

### Max (44 actions, 22 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C2 pull | pass (no pull) | 0 | no (Super+Alt+1 "Give window to Tiling" x3, nothing; no second look) |
| C3 two terminals, fly, Tab | pass (two panes in the room) | 0 | no (full-screen terminals; Return did not fly, Tab shows nothing inside a terminal) |
| T5 | not run | - | - |
| T3 | not run | - | - |

### Demo watcher (25 actions, 14 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | 0 | no (row goes into face 0, no cube seen) |
| C2 | pass (no pull) | I3 x6 | no (icons select, nothing opens; Start not tried) |
| T4 lock | pass (calls.log `swaylock -f` x2) | I3 x7 | no by its eyes (Start, Lock: no lock screen seen; Super+L nothing) |

### Fix check against r4
| Row | Reached | Holds | Capture |
|---|---|---|---|
| T2 key list by mouse and by F1 (co-41, co-keys r3) | explorer by mouse in a face: no (xp Help opened, no Keys door in a face, F1 and Super+F1 not pressed); Max: Keys item in the walk room opens the list by mouse, Super+F1 opens it from a full-screen Foot pane | **FAIL** for the explorer (contract T2 red, as r4); by key and by the walk-room item pass (list shows Super+T, Return, Tab, Alt+Esc) | persona-r5-explorer shots 016-021; persona-r5-max shots 003-005, 011-013 |
| r4 I7 at the first click (known, harness) | yes | gone: 0 breaches in C1 (explorer, demo) | persona-r5-explorer/breaches.jsonl |
| r4 explorer T2 red | see T2 | **still red** | above |
| r4 Max C3 not reached (Foot fills the view) | yes | partial: two panes now exist (contract pass), persona still not reached | persona-r5-max shots 008-022 |
| r4 demo T4 lock (click highlights only) | yes | contract pass again; Start, Lock now acts (stand-in called), no lock screen seen | persona-r5-demo/calls.log |
| r4 Max I3 pane dragged out; co-33; co-35; r4 C2 xp child I1/I3 | pane drag and Tab after the flight not tried; no pull | not reached; **new: I3 "construct and xp each say they hold the focus" on every click inside the xp face (demo from its first click, no window open; explorer 6 rows); r4 demo had 0 in 24 actions** | persona-r5-demo/breaches.jsonl, evidence/009-022; persona-r5-explorer/evidence |

### triage.py (the three files; `python3 triage.py --format feedback/2026-11-21-p2r5co-*.md`: FORMAT OK; the full run lists all files, the r5-co lines are)
```
