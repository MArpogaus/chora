## r8
Lane: cloud, branch claude/p2-r8-co at 8c411416 (run only, no code changes; contains fe369799 co-reach). Step 0: unshallow, `cargo check --workspace` and `cargo build --workspace` green; `round.sh --help` ok; smoke `begin` + `check.py end smoke` on construct ("no contract for smoke"). **Binaries of the round: sha 8c411416eb1e77b0966bbb3951aafe311e03be35** (/tmp/persona-round/bin, one snapshot).
Setup: construct 1280x800, 2 slots (explorer and Max together, demo after), PERSONA_CHECK=1, helper transcript seeded (one agent entry) in all three sessions, one `begin` per goal and `check.py end` after it, one haiku Agent per goal (act.py only). Probe started before T5 (`round.sh probe max`). Caps written before the run (/tmp/persona-round/caps.txt, as r7; sum to 45 actions, 20 looks): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 11/5, C3 12/5, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6. Used: explorer 21 actions, 12 looks; Max 31, 17; demo 36, 19. **No cap breached**, no retries past 3, no coaching. Every task of every persona ran.
Non-I6 breaches in the persona runs: the known co-44 I2 rows (explorer 28, Max 40, demo 4) and, in demo T4 only, **I2 x3 and I3 x1 at the three Lock clicks** (co-51). Nothing else. I6 rows: explorer 163, Max 72, demo the known xp family. Captures: media/2026-11-24/persona-r8-{explorer,max,demo}/ (explorer and Max `evidence/` dropped: I6 and co-44 only), hand runs media/2026-11-24/persona-r8-hand/.

### Explorer
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass (cube stood back, face 0) | 0 (co-44 I2 x28) | yes (row, Return, F1, Alt+Escape, Escape) |
| T1 | pass | 0 | yes (Programs, Foot) |
| T2 | pass | 0 | yes (Super+F1) |

### Max
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C2 | pass | 0 (co-44 I2 x40) | no (terminal panel visible left of the cube; turn left only cycles XP faces, a click on the panel does nothing; Tab not tried) |
| C3 | pass | 0 | no (four cube turns, ran out of looks; Super+Return never pressed) |
| T5 | pass (probe got no Super key) | 0 | no (probe window not found on the cube) |
| T3 | pass | 0 | yes (LISTENING pill, Hello, answer) |

### Demo watcher
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | 0 (co-44 I2 x4) | no (row shows the cube, Return steps in; no way out found: Escape, Super+C, pager arrow) |
| C2 | pass | 0 (I3 rows 0) | no (Super+Return nothing; drag to pager button 2 does not move the window; co-55, co-56 known) |
| T4 lock | pass (`swaylock -f` x3) | **I2 x3, I3 x1** | no by its eyes (no lock window read; no key pressed) |

### Fix check against r7 (co-reach r1)
| Row | Result | Capture / state dump |
|---|---|---|
| co-52 explorer C1 | **pass** (contract C1 pass; the row shows the cube stood back, `Step in` in the hint bar; r7 plain desktop) | persona-r8-explorer shots 003-004 |
| co-52 demo C1 | **pass** (row shows the cube, Return steps in); step out not reached (UX, co-47 family) | persona-r8-demo shots 004-005 |
| co-54 Max C2 | **pass** by contract C2 and by the scene: terminal visible beside the cube after Super+C; persona **not reached** (cannot get to it) | persona-r8-max shots 005 |
| co-51 demo T4 | **FAIL** in the persona part: I2 x3 (s2, s3, s4 "no box of a size") and I3 x1 ("xp says it focuses s3, the core focuses s1") at the three Lock clicks; contract T4:lock pass. Dump after: construct `{'holder': None}`, xp `{'holder': 'xp', 'surface': 's1'}`, cube use, front 3. The persona pressed no key afterwards | persona-r8-demo evidence/, breaches.jsonl |
| co-48 | holds (explorer T1 contract pass, Max C3 contract pass) | |
| co-49 | holds (Max T3 contract pass, reached) | |
| co-50 | holds (demo C2 contract pass, I3 rows 0) | |

### co-52, co-54, co-51 by hand, 5x each (driver, act.py with the checker on, each run from a fresh compositor with no earlier windows: one `round.sh start` per run; script /tmp/persona-round/hand.sh)
| Row | Steps | Rate | Notes |
|---|---|---|---|
| co-52 | row "A cube of desktops", 6 s, Super+C, 3 s | **5/5 pass** (cube mode navigate, front 0, construct holds the keys; contract C1 pass; r1 before the fix 5/5 red) | non-I6 rows: co-44 only |
| co-54 | row Terminal, 5 s, Super+C, 6 s | **5/5 pass** by contract C2 (window not gone; scene: cube stood back, pane room yes); capture of run 5 shows the terminal beside the cube. My per-window dump failed (no `title` key in the construct windows), so shown/focus of the pane were read from the contract and the capture, not the dump. Reachable by Tab fly **not tried** here | non-I6 rows: co-44 only |
| co-51 | row, Return (step in), Start, Lock, 12 s, key a | **I3 0/5; I2 5/5** ("xp shows s1 on VIRTUAL-1, which has no box of a size there" at the Lock click); contract T4:lock pass 5/5; `swaylock -f` x1 each; after key a: construct `{'holder': None}`, xp `{'holder': 'xp'}` | fresh session each run, so the r7 confound is gone: the I2 comes at every Lock click as co-reach r1 found (checker snapshot slower than the 8 s stand-in). The I3 row seen once in the persona part (Lock clicked three times, three stand-in windows at once) did not come in the hand runs |

### triage.py (`python3 triage.py --format feedback/2026-11-24-p2r8co-*.md`: FORMAT OK; 17 findings of the three files, none MISSING UX; the lines of the r8 files from the full output)
```
== construct / I2 (6)
  2026-11-24-p2r8co-demo.md: Start, Lock: I2 and I3 at the Lock click (co-51) (0 days)
  2026-11-24-p2r8co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r8co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r8co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
== construct / I6 (9)
  2026-11-24-p2r8co-demo.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r8co-explorer.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r8co-max.md: xp face contrast rows (I6) (0 days)
== construct / contract C1 (5)
  2026-11-24-p2r8co-explorer.md: C1: cube, step in, step out reached (0 days)
== construct / contract T1 (5)
  2026-11-24-p2r8co-explorer.md: T1: terminal through the Programs menu, full screen (0 days)
== construct / contract T2 (5)
  2026-11-24-p2r8co-explorer.md: T2: key list found by Super+F1 (0 days)
== construct / contract T3 (3)
  2026-11-24-p2r8co-max.md: T3: helper chat through the LISTENING pill (0 days)
== construct / contract T5 (6)
  2026-11-24-p2r8co-max.md: T5: the probe window is not found on the cube (0 days)
== construct / none (58)
  2026-11-24-p2r8co-demo.md: C1: no way out of a face is shown (0 days)
  2026-11-24-p2r8co-demo.md: C2: Super+Return and the pager drag do nothing (0 days)
  2026-11-24-p2r8co-demo.md: T4: Lock shows no lock the persona can read (0 days)
  2026-11-24-p2r8co-max.md: C2: terminal stays visible but the persona cannot reach it (0 days)
  2026-11-24-p2r8co-max.md: C3: a second terminal pane not reached (0 days)
waiting 327, overdue 310, missing ux 0
```

### Stop rule (p2-plan.md)
Round 8 is **not met** for construct. Contracts all green (C1-C3, T1-T5 pass; co-48, co-49, co-50 hold; co-52 and co-54 fixed by contract, co-52 5/5 by hand, co-54 5/5 by contract). T1-T7 and C1-C3 are not all reached by the personas: reached are C1 (explorer), T1, T2, T3; not reached are C1 (demo, no way out of a face), C2 (Max: terminal visible but not reachable; demo: Super+Return and pager drag, co-55 and co-56 known), C3 (Max), T5 (probe not found on the cube), T4 lock by eyes (co-57 known). New check-backed BLOCK or MAJOR: **none new**; co-51 stays a harness-side I2 (5/5 fresh runs, as co-reach r1 classed it NIT) and its I3 row (1 in the persona part, 0 in 5 hand runs) is a candidate for one capture read. The gaps are UX rows with check none (no way out of a face shown; Tab fly and Super+Return not discoverable from the cube). No ledger edit here.
