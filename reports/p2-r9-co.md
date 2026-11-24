## r9
Lane: cloud, branch claude/p2-r9-co at b2fa7792 (co-reach r2 head; run only, no code changes). Step 0: unshallow, `cargo check --workspace` and `cargo build --workspace` green; `round.sh --help` ok; smoke `begin` + `check.py end smoke` on construct ("no contract for smoke"). **Binaries of the round: sha b2fa7792198e7731c2b2ecc4b9f97d22469571a5** (/tmp/persona-round/bin, one snapshot).
Setup: construct 1280x800, 2 slots (explorer and Max together, demo after), PERSONA_CHECK=1, helper transcript seeded (one agent entry) in all three, one `begin` per goal and `check.py end` after it, one haiku Agent per goal (act.py only). Probe started before T5 (`round.sh probe max`; a first probe started too early was killed before C2). Caps written before the run (media/2026-11-24/persona-r9-hand/caps.txt, as r8): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 11/5, C3 12/5, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6. Used: explorer 15 actions, 9 looks; Max 33, 17; demo 32, 17. **No cap breached**, no retries past 3, no coaching. Every task of every persona ran. Driver slip: for the first two goals (explorer C1, Max C2) I cut the `check.py end` output with `tail` and lost the task line (single-use `end`); the contract of those two is **not read** (the breaches section shows I6 only). Persona slip: the T5 persona made one read_documentation call (no screen effect).
Non-I6 breaches: only co-44 I2 (explorer 20, Max 60, demo 50) and in demo T4 one I2 at the lock (co-51 family, below). I3: 0. Captures: media/2026-11-24/persona-r9-{explorer,max,demo}/, hand runs persona-r9-hand/, contract lines persona-r9-ends/.

### Explorer
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | not read (driver slip) | co-44 only | yes (row, Return, Alt+Escape) |
| T1 | pass | co-44 only | yes (Programs, Foot) |
| T2 | pass | co-44 only | yes (All keys in the hint bar) |

### Max
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C2 | not read (driver slip) | co-44 only | no (drag moved the pane across the cube, read as turning the cube; no click on the pane; never typed) |
| C3 | pass | co-44 only | no (Foot opened full screen in a face, top bar hidden, Escape nothing, out of looks) |
| T5 | pass (probe got no Super key) | co-44 only | no (probe window not found on four faces) |
| T3 | pass | co-44 only | yes (LISTENING pill, Hello) |

### Demo watcher
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | co-44 only | **yes** (row, Return, Alt+Escape; shot 003 hint bar "Alt Esc Step back") |
| C2 | pass | co-44 only | no (no float key named, Tab only flies, drag in the face does nothing; Super+drag not possible with act.py) |
| T4 lock | pass (`swaylock -f` x2) | **I2 x1** at the lock (s2 on HEADLESS-1, 9 s after Super+Shift+Escape; capture evidence/028-key.png) | no by its eyes (no lock window in a look; Super+Shift+Escape with the key list open showed nothing) |

### Fix check against r8 and co-reach r2
| Row | Result | Capture / state dump |
|---|---|---|
| co-61 demo C1 | **pass** (persona stepped out with Alt+Escape; hint bar names Step back) | persona-r9-demo shots 003, 004 |
| co-61 explorer C1 | pass by the persona (stepped out); contract not read | persona-r9-explorer shots 003, 004 |
| co-62 Max C2 | **not reached**: the persona never clicked the pane (it dragged it; shot 004 shows the pane moved from left to right of the cube); no state to dump | persona-r9-max shots 003-005 |
| co-52 | holds (explorer, demo: row shows the cube stood back; hand runs) | |
| co-54 | holds (terminal pane beside the cube after Super+C: Max shot 003; C3 scene "pane room yes") | |
| co-51 | I2 x1 in demo T4 (down from x3), I3 x0; the contract T4:lock pass; harness family as co-reach r1 | persona-r9-demo evidence/ |

### co-61 and co-62 by hand, 5x each (driver, act.py with the checker on, each run from a fresh compositor with no earlier windows: one `round.sh start` per run; script media/2026-11-24/persona-r9-hand/reach.sh as of co-reach r2)
| Row | Steps | Rate | Notes |
|---|---|---|---|
| co-61 | row "A cube of desktops", 6 s, Return, 5 s, dump | **5/5 pass** | every run: mode `use`, construct `{"holder": "construct"}`, xp `{"holder": null}`, hint bar `Super | F1 | All keys | Alt | Esc | Step back`; non-I6 rows: co-44 I2 x2 per run |
| co-62 | row Terminal, 5 s, Super+c, 6 s, click the pane centre (208,400), 3 s, type | **5/5 pass** | every run: before the click mode `navigate`, pane at [208 400]; after it construct `{"holder": "construct", "surface": "s1"}`, xp `{"holder": null}`, camera [-1191 2391]; `echo reach` typed; non-I6 rows: co-44 I2 x8 per run |

### triage.py (`python3 triage.py --format feedback/2026-11-24-p2r9co-*.md`: FORMAT OK; the r9 lines of the full output)
```
== construct / I1 (3)
== construct / I2 (10)
  2026-11-24-p2r9co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r9co-demo.md: Lock: I2 after Super+Shift+Escape (co-51 family) (0 days)
  2026-11-24-p2r9co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r9co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
== construct / I3 (6)
== construct / I6 (12)
  2026-11-24-p2r9co-demo.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r9co-explorer.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r9co-max.md: xp face contrast rows (I6) (0 days)
== construct / I7 (10)
== construct / contract C1 (7)
  2026-11-24-p2r9co-demo.md: C1: the face names its way out (co-61 fix seen) (0 days)
  2026-11-24-p2r9co-explorer.md: C1: cube, step in, step out reached (0 days)
== construct / contract C2 (8)
== construct / contract C3 (3)
== construct / contract T1 (6)
  2026-11-24-p2r9co-explorer.md: T1: terminal through the Programs menu (0 days)
== construct / contract T2 (6)
  2026-11-24-p2r9co-explorer.md: T2: key list found through All keys (0 days)
== construct / contract T3 (4)
  2026-11-24-p2r9co-max.md: T3: helper chat through the LISTENING pill (0 days)
== construct / contract T4 (1)
  2026-11-24-p2r9co-demo.md: T4: Lock shows no lock the persona can read (0 days)
== construct / contract T4:lock (4)
== construct / contract T5 (7)
  2026-11-24-p2r9co-max.md: T5: the test window is not found on four cube faces (0 days)
== construct / none (61)
  2026-11-24-p2r9co-demo.md: C2: the terminal cannot be pulled out of its face by the persona (0 days)
  2026-11-24-p2r9co-max.md: C2: a drag on the pane beside the cube reads as turning the cube (0 days)
  2026-11-24-p2r9co-max.md: C3: Programs opens Foot in the face, full screen; Escape does not step out (0 days)
waiting 359, overdue 310, missing ux 0
```

### Stop rule (p2-plan.md)
Round 9 is **not met** for construct. All read contracts are green (C1, C2 demo, C3, T1-T5 pass; two contract lines, explorer C1 and Max C2, were lost by the driver). T1-T7 and C1-C3 are not all reached by the personas: reached are C1 (explorer, **demo: co-61 fixed**), T1, T2, T3; not reached are C2 (Max: pane dragged, no click; demo: no way to float found), C3 (Max: Foot full screen in a face), T5 (probe not found on the faces), T4 lock by eyes (co-57 known). Fix checks: co-61 pass (persona and 5/5 by hand), co-62 5/5 by hand but not exercised by the persona, co-51 down to one I2 and no I3, co-52 and co-54 hold. New check-backed BLOCK or MAJOR: **none**. The open gaps are UX rows with check none: the person is not told that Super+C already shows a window as a pane that can be clicked or dragged (Max C2, demo C2), a program started from the cube opens full screen in a face with the top bar hidden (Max C3), and the key list is the only place a float key could be named. No ledger edit here.
