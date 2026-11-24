# p2-r11-co: RC 2 persona round 11 on construct

## r11
Lane: cloud, branch claude/p2-r11-co at be6e62ec (co-pull r1 head). Run only, no code change. Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` ok; smoke `begin smoke` + `check.py end smoke` on construct ("no contract for smoke"). **Binaries of the round: sha be6e62eca5fe203cbddb76b4f34cc6b6d06c6832** (/tmp/persona-round/bin, one snapshot).

Setup as r10: construct 1280x800, PERSONA_CHECK=1, helper transcript seeded (one agent entry), explorer and Max together, demo after; one `begin` per goal, `check.py end` after it (output kept to file), one haiku Agent per goal (act.py only), probe started once before Max T5. Caps written before the run (media/2026-11-24/persona-r11-hand/caps.txt): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 14/6, C3 9/4, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6 (totals 45/20). Used: explorer 15 actions, 9 looks; Max 29, 16; demo 29, 15 (C2 15/7 at its cap). **No cap breached**, no retries past 3, no coaching, no tool outside act.py and Read. Every task of every persona ran, all 10 contract lines read: C1, T1, T2, C2, C3, T5, T3, C1, C2, T4:lock all **pass** (media/2026-11-24/persona-r11-ends/).
Non-I6 breaches: co-44 I2 (explorer 18, Max 57, demo 20 rows), **I3 x2 in Max C3 (new, below)**, I2 "construct shows s3 on HEADLESS-1, no box" x1 at a Lock click (demo, co-51 family). No I5 this round. Captures: media/2026-11-24/persona-r11-{explorer,max,demo}/, hand runs persona-r11-hand/.

### Explorer
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | co-44 | yes (row, Return, Alt+Escape; 7 actions) |
| T1 | pass | co-44 | yes (Programs, Foot; 5 actions) |
| T2 | pass | co-44 | yes (hint bar "Super F1 All keys"; 3 actions) |

### Max
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C2 | pass | co-44 | no by its words (Terminal beside the cube after Super+c, "Click Fly in" hint; it dragged the pane onto a face and could not tell if it is in; end state: xp holds s1, so it is in the face; the pull key was not named and never pressed) |
| C3 | pass | co-44, **I3 x2** ("the focused s2 is no shown window of xp") | partly (Foot started from Programs, appeared on a face, contents unreadable; the persona opened the key sheet and read Alt+Esc, did not step in) |
| T5 | pass (probe got no Super key) | co-44 | no (the key sheet opened in C3 stayed open through Esc, click and its Close; the probe unlabeled; 7 actions, 4 looks) |
| T3 | pass | co-44 | yes (Super+A, hello, "Hello") |

### Demo watcher
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | co-44 | yes (row, Return, Alt+Escape; 7 actions) |
| C2 | pass | co-44 | no (Return, Super+t put foot in the XP face, Esc x2 did nothing, Alt+Escape stood back with the window inside the face; no pull key on the bar; never pulled; cap) |
| T4 lock | pass (`swaylock -f` x3) | co-44, I2 x1 at a Lock click (co-51 family) | no by its eyes (highlight only, co-57 known) |

### Fix check
| Row | Result | Capture / state dump |
|---|---|---|
| co-68 (hint names the pull only where it acts) | **partly**. Pane beside the cube and the keyboard in the pane (r10 states): `Pull out` not named, key silent (5/5 by hand, reach.sh co64 below). A face window with the keyboard and a pane beside: told 5/5, key pulls it 5/5 (see rcp1-1). **But**: a face window with the keyboard and no pane beside (Return, Super+t, Alt+Escape): the bar shows only "Return, turns, All keys", `Pull out` not told, yet Super+Alt+4 **does pull** the window 5/5 (s1 owner xp -> construct, rect 290,150,700,500, camera moved): the key acts where it is not told | persona-r11-hand hr11-co68-co68-*.log; state, run 1: stood back `mode navigate front 0, camera [0,0], panes [], holders construct / xp none, s1 owner xp`; after the key `panes [foot], s1 owner construct visible` |
| rcp1-1 (told pull on screen, face window with keys: hint shown, key pulls) | **pass 5/5** by hand with a pane beside (row Terminal, Super+c, Return, Super+t, Alt+Escape): bar `Click Fly in ... Super Alt 4 Pull out`; Super+Alt+4 pulls the face window s2 (owner construct, visible), no breach but co-44. Not met by a persona | persona-r11-hand hr11-rcp11-co681.log; state: stood back, panes [foot], s1 construct, s2 xp; after: panes [foot, foot-2], s2 owner construct, rect [290,150,700,500], holders construct s2 |
| rcp1-2 (C2 reach of Max and demo) | **confirmed, C2 not reached by a persona**. Max found the pane beside the cube by Super+c and moved it by drag (the click hint told "Click Fly in"; the pull key was not named, so was not pressed). Demo went into the face, started foot there, found Alt+Esc by the bar, stood back, saw no pull key told and never pulled. By which way: found (drag, Super+c), never told by the bar. The key sheet group "GIVE A WINDOW" exists but was not read | persona-r11-max 001-005, persona-r11-demo 005-011 |
| co-64 (way told on screen, 5x by hand, reach.sh co64) | **told: click yes 5/5, pull key no 5/5** (by design after co-68: pane beside, keyboard in the pane); the drag moves the pane 5/5 (s1 at [1125 400], cube front 0). The script's "click and pull named" flag is 0/5 for that reason | persona-r11-hand hr11-co64.log |
| co-65 | holds (Foot from Programs hint bar `Alt Esc Step back`, demo, Max read it in the key sheet) | max 006-009 |
| co-67 | holds (Foot started while stood back showed beside the cube, Max C3 on a face; demo Alt+Escape showed foot in the face) | |
| co-61 | holds (explorer, demo, 3 persona C1 runs) | explorer 004, demo 004 |
| co-62, co-52, co-54 | hold (rows enter, pane beside reachable by drag, no lost window in any contract) | |
| co-51 | I2 x1 at Lock (r10 x3), I3 x0 in T4, contract T4:lock pass | demo T4 evidence |

### Hand runs, 5x each (driver, act.py with the checker on, each from a fresh compositor with no earlier windows; scripts persona-r11-hand/co68.sh and reach.sh; binaries be6e62ec; fuzzel cache cleared per run)
| Row | Steps | Rate | Notes |
|---|---|---|---|
| co-68 (no pane) | row, Return, Super+t, Alt+Escape, Super+Alt+4 | key pulls 5/5; told 0/5 | see the table above; breaches: none but co-44 |
| rcp1-1 (pane beside) | row Terminal, Super+c, Return, Super+t, Alt+Escape, Super+Alt+4 | told 5/5, pulls 5/5 | after the pull the camera flew to the new pane (`Step back` bar) |
| co-64 | row Terminal, Super+c, hint, drag on the pane | click named 5/5, pull not named 5/5 (as designed), drag moves 5/5 | |

### New rows proposed (no ledger edit here)
1. **Pull key acts but is not told in a face window with the keyboard and no pane beside the cube (rcp1-2 family; MINOR, check: hand run co68.sh 0/5 told, 5/5 acts)**. The hint bar only names it in the pane set (`pane_hints`), so the one state where a person has just stepped back from a face and wants to float the window never shows the key; demo C2 not reached for that reason. Candidate: raise to MAJOR if C2 stays unreached next round.
2. **I3 "the focused s2 is no shown window of xp" in Max C3 (candidate MINOR, check I3)**, at the Foot start from Programs while a pane window existed.
3. **The key sheet opened in C3 stays open through Esc, a click and its Close button (Max T5, check: none, one capture read: persona-r11-max shots 010-013)**; the co-47 family (keys after cube close) may cover it.

### triage.py (`python3 triage.py --format feedback/2026-11-24-p2r11co-*.md`: FORMAT OK; the r11 lines of the full output, construct groups)
```
== construct / I2 (18)
  2026-11-24-p2r11co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r11co-demo.md: T4: Lock gives the persona no lock it can read (0 days)
  2026-11-24-p2r11co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r11co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
== construct / I3 (7)
  2026-11-24-p2r11co-max.md: C3: I3 focused s2 is no shown window of xp after Foot from Programs (0 days)
== construct / I6 (18)
  2026-11-24-p2r11co-demo.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r11co-explorer.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r11co-max.md: xp face contrast rows (I6) (0 days)
== construct / contract C1 (11)
  2026-11-24-p2r11co-demo.md: C1: cube, step in, step out reached (0 days)
  2026-11-24-p2r11co-explorer.md: C1: cube, step in, step out reached (0 days)
== construct / contract C2 (9)
  2026-11-24-p2r11co-max.md: C2: pane beside the cube, drag back into a face; the persona cannot tell if it worked (0 days)
== construct / contract T1 (8)
  2026-11-24-p2r11co-explorer.md: T1: terminal through the Programs menu (0 days)
== construct / contract T2 (8)
  2026-11-24-p2r11co-explorer.md: T2: key list found by the hint bar (0 days)
== construct / contract T3 (6)
  2026-11-24-p2r11co-max.md: T3: helper chat by Super+A (0 days)
== construct / none (67)
  2026-11-24-p2r11co-demo.md: C2: no pull key told in a face with a window; Esc does not step back (0 days)
  2026-11-24-p2r11co-demo.md: C2: Esc inside the face does not step back, the hint says Alt Esc, the key sheet says Esc (0 days)
  2026-11-24-p2r11co-max.md: T5: the key sheet left open by C3 does not close with Esc or Close; the probe pane cannot be found (0 days)
```

### Stop rule (p2-plan.md)
Round 11 is **not met** for construct. All ten contracts are green. Reached by the personas: C1 (explorer, demo), T1, T2, T3. Not reached: **C2** (Max: found the pane by Super+c and the drag, unsure; demo: the pull key is not on the bar in a face window with the keyboard and no pane, never pulled), C3 (Max: Foot started, not entered, no check red), T5 (the key sheet of C3 would not close; the probe unlabeled, co-66 family), T4 lock by eyes (co-57 known). Fix checks: co-68 holds where it was reproduced (no false hint with the pane beside and the keyboard in it, 5/5), rcp1-1 passes 5/5 by hand, rcp1-2 confirmed, co-64 click told 5/5, co-65, co-67, co-61, co-62, co-52, co-54 hold. New check-backed BLOCK or MAJOR: **none**; two new check-backed MINOR candidates (the pull key acts but is not told with no pane, I3 at Foot start) and one check-none UX gap (the key sheet does not close). No ledger edit here.
