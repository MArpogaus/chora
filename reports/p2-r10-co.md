# p2-r10-co: RC 2 persona round 10 on construct

## r10
Lane: cloud, branch claude/p2-r10-co at f1457035 (co-reach r4 head). Run only, no code change. Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` ok; smoke `begin smoke` + `check.py end smoke` on construct ("no contract for smoke"). **Binaries of the round: sha f145703534254945535f2a0e652d2d07ff7e6abe** (/tmp/persona-round/bin, one snapshot).

Setup: construct 1280x800, PERSONA_CHECK=1, helper transcript seeded (one agent entry) in all three sessions, explorer and Max together, demo after; one `begin` per goal, `check.py end` after it (output kept to file, no tail), one haiku Agent per goal (act.py only). Probe started once before Max T5 (`round.sh probe max`). Caps written before the run (media/2026-11-24/persona-r10-hand/caps.txt, as r9): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 11/5, C3 12/5, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6 (totals 45/20). Used: explorer 15 actions, 9 looks; Max 31, 17; demo 28, 15 (demo C1 7/4, C2 14/7, T4 7/4). **No cap breached**, no retries past 3, no coaching, no persona tool outside act.py and Read. Every task of every persona ran. All 10 contract lines were read (none lost this round): C1, T1, T2, C2, C3, T5, T3, C1, C2, T4:lock all **pass** (media/2026-11-24/persona-r10-ends/).
Non-I6 breaches: co-44 I2 only (explorer 18, Max 32, demo 46), in demo T4 three I2 "construct shows s2/s3/s4 on HEADLESS-1, no box" at the three Lock clicks (co-51 family), and **I5 x5 in Max** during T5 (new, below). I3: 0. Captures: media/2026-11-24/persona-r10-{explorer,max,demo}/, hand runs persona-r10-hand/.

### Explorer
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | co-44 only | yes (row, Return, Alt+Escape; 7 actions) |
| T1 | pass | co-44 only | yes (Programs, Foot; 5 actions) |
| T2 | pass | co-44 only | yes (hint bar "Super F1 All keys", Super+F1; 3 actions) |

### Max
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C2 | pass | co-44 only | no (Terminal pane beside the cube after Super+c, hint "Super Alt 4 Pull out" visible; the persona turned the cube twice, never pressed the pull key and could not tell if the pane already floats; 9 actions) |
| C3 | pass | co-44 only | yes (Programs, Foot; hint bar "Alt Esc Step back"; Alt+Escape showed Foot beside the cube) |
| T5 | pass (probe got no Super key) | co-44, **I5 x5** (labels 'Terminal' and 'Chora probe' overlap at [179,521,58,20]) | no (found the grey "Chora probe" pane, flew into it, saw a blank surface, out of looks before any Super chord) |
| T3 | pass | co-44 only | yes (LISTENING pill, panel ATLAS, hello, Hello) |

### Demo watcher
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | co-44 only | yes (row, Return, Alt+Escape; 7 actions) |
| C2 | pass | co-44 only | no (Programs, Foot, a drag turned the cube to face 2 with Foot pane beside it, Super+Alt+4 "Pull out": no visible change) |
| T4 lock | pass (`swaylock -f` x3) | **I2 x3** at the Lock clicks (co-51 family) | no by its eyes (highlight only; the stand-in draws no readable lock, co-57 known) |

### Fix check
| Row | Result | Capture / state dump |
|---|---|---|
| co-64 (Max, demo C2) | **way told on screen: pass** (hint bar `Click Fly in`, `Super Alt 4 Pull out` seen by Max and demo while a pane was beside the cube). The pull act itself did **not** work for either persona: Max pressed Super+Alt+4 inside a full-screen pane (no change), demo pressed it with the stood-back cube on face 2 and no focused face window (no change; rco3-2 "no message"). Contract C2 pass, C2 not reached by both. State at the end of demo C2: cube mode `navigate`, front 1, construct `{"holder": "construct"}`, xp `{"holder": null}`, camera [0, 0], pane foot at [-1562, 3136], surface s1 beside the cube. | persona-r10-demo shots 009-011; persona-r10-max shots 004-005, 009 |
| co-65 (Max C3) | **pass**: Foot from Programs opened in the pane, top bar hidden (by design), hint bar `Alt Esc Step back`, Alt+Escape stepped out and Foot showed beside the cube | persona-r10-max shots 008-010 |
| co-67 (Max C3 path) | pass: the cube stood back, Foot started, after Alt+Escape Foot on screen beside the cube (shot 010); 5/5 by hand below | persona-r10-max shot 010 |
| co-61 | holds (explorer, demo, 5/5 persona C1 steps) | shots 004 |
| co-62 | holds (hand co-64 runs, click path) not exercised by a persona click on a pane | |
| co-51 | I2 x3 at Lock (as r9 x1), I3 x0, contract T4:lock pass | persona-r10-demo evidence |
| co-52, co-54 | hold (row shows the cube stood back; Foot, Terminal and the probe beside the cube) | |

### co-64, co-65, co-67 by hand, 5x each (driver, act.py with the checker on, each run from a fresh compositor with no earlier windows: one `round.sh start` per run; scripts media/2026-11-24/co-reach-r3/reach.sh and co-reach-r4/co67.sh; binaries f1457035; fuzzel cache cleared per run)
| Row | Steps | Rate | Notes |
|---|---|---|---|
| co-64 | row Terminal, Super+c, 6 s, read the hint bar, drag on the pane | **5/5 pass** | every run: mode `navigate`, pane s1 at [208 400], hint bar `Click Fly in ... Super Alt 4 Pull out` named; the drag moves the pane (s1 at [1125 400]), the cube stays at front 0, camera [0, 0] |
| co-65 | row, Programs, Foot, 6 s, Escape, Alt+Escape | **5/5 pass** | every run: in foot (construct holder s1), top bar hidden, hint bar `Super F1 All keys, Alt Esc Step back`; Escape reaches foot; Alt+Escape steps out (camera [0, 0], top bar back) |
| co-67 | row, Super+c, start Foot (Super+t in 3 runs, Programs in 2), Alt+Escape, Return | **5/5 pass** | every run: after Alt+Escape s1 rect [17, 283, 383, 233] on screen beside the cube, after Return (mode use) the same rect; pane at [-1562, 3136]; keyboard stays with s1 |
Non-I6 rows of all hand runs: co-44 I2 only.

### triage.py (`python3 triage.py --format feedback/2026-11-24-p2r10co-*.md`: FORMAT OK; the r10 lines of the full output, construct groups)
```
== construct / I1 (3)
== construct / I2 (14)
  2026-11-24-p2r10co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r10co-demo.md: Lock: I2 at each click on Lock (co-51 family) (0 days)
  2026-11-24-p2r10co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r10co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
== construct / I3 (6)
== construct / I5 (1)
  2026-11-24-p2r10co-max.md: T5: the labels of two panes beside the cube overlap (0 days)
== construct / I6 (15)
  2026-11-24-p2r10co-demo.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r10co-explorer.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r10co-max.md: xp face contrast rows (I6) (0 days)
== construct / I7 (10)
== construct / contract C1 (9)
  2026-11-24-p2r10co-demo.md: C1: cube, step in, step out reached (0 days)
  2026-11-24-p2r10co-explorer.md: C1: cube, step in, step out reached (0 days)
== construct / contract C2 (8)
== construct / contract C3 (4)
  2026-11-24-p2r10co-max.md: C3: Foot from Programs fills the view; the way out is named (0 days)
== construct / contract T1 (7)
  2026-11-24-p2r10co-explorer.md: T1: terminal through the Programs menu (0 days)
== construct / contract T2 (7)
  2026-11-24-p2r10co-explorer.md: T2: key list found by the hint bar (0 days)
== construct / contract T3 (5)
  2026-11-24-p2r10co-max.md: T3: helper chat through the LISTENING pill (0 days)
== construct / contract T4 (1)
== construct / contract T4:lock (5)
  2026-11-24-p2r10co-demo.md: T4: Lock gives the persona no lock it can read (0 days)
== construct / contract T5 (7)
== construct / none (64)
  2026-11-24-p2r10co-demo.md: C2: "Super Alt 4 Pull out" does nothing visible, the plain drag turns the cube (0 days)
  2026-11-24-p2r10co-max.md: C2: "Pull out" is shown but does nothing for a pane already beside the cube (0 days)
  2026-11-24-p2r10co-max.md: T5: the probe pane shows a blank grey surface (0 days)
```

### Stop rule (p2-plan.md)
Round 10 is **not met** for construct. All ten contracts are green (C1, C2, C3, T1-T5 pass in their runs). Reached by the personas: C1 (explorer, demo), T1, T2, T3, **C3 (Max: co-65 fixed)**. Not reached: C2 (Max and demo: the pull key `Super Alt 4` named in the hint bar does nothing visible for the persona; the plain drag turns the cube or moves the pane), T5 (the probe pane is blank for the persona, out of looks), T4 lock by eyes (co-57 stand-in, known). Fix checks: co-64 told on screen (hint bar) but the pull act is not reached by a persona (5/5 by hand for the hint and drag), co-65 pass (persona and 5/5), co-67 pass (5/5 by hand, persona path seen), co-61, co-62, co-51, co-52, co-54 hold. New check-backed BLOCK or MAJOR: **none**; one new check-backed row: I5 labels of two panes overlap at the cube (Max T5), candidate MINOR. UX gaps with check none: the pull key hint shows while the pull cannot act (rco3-2 family, now met by two personas: candidate to raise), the probe pane blank. No ledger edit here.
