# p2-r13-co: RC 2 persona round 13 on construct

## r13
Lane: cloud, branch claude/p2-r13-co at 62a04be0 (train140 green). Run only, no code change. Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` ok; smoke `begin smoke` + `check.py end smoke` ("no contract for smoke"). **Binaries of the round: sha 62a04be03df218c9f745dedd7baaf689e231bcad** (/tmp/persona-round/bin, one snapshot).

Setup as r12: construct 1280x800, PERSONA_CHECK=1, helper transcript seeded (one agent entry), 2 slots, one `begin` per goal, `check.py end` after it (output kept in media/2026-11-25/persona-r13-ends/), one haiku Agent per goal (act.py only), probe started once per Max run before T5. Task words fresh in a user's words (r12's were not kept), same in both runs. Caps written before the run (media/2026-11-25/persona-r13-hand/caps.txt): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 14/6, C3 9/4, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6. Used: explorer 17 actions, 10 looks; Max 1 38, 19; Max 2 29, 16; demo 1 30, 15; demo 2 32, 17. **No cap breached**, no retries past 3, no coaching. Every task of every persona ran, all 17 contract lines read: **16 pass, 1 FAIL (Max run 2 C3)**. One tool slip: demo 1 C2 made one read_documentation call (no screen effect). Captures: media/2026-11-25/persona-r13-captures/; hand runs media/2026-11-25/persona-r13-hand/ (scripts hand.sh, logs).
Non-I6 breaches: co-44 I2 (every persona), one I5 (label overlap, co-69, Max 2 drag), demo 2: I2 x2 + I3 x2 at the Lock clicks (co-51 family). No other I3.

### Per task
| Persona / run | Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|---|
| Explorer | C1 | pass | co-44 | yes (row, Return, Alt+Escape; 7) |
| Explorer | T1 | pass | co-44 | yes (Programs, Foot; 5) |
| Explorer | T2 | pass | co-44 | yes (Super+F1, Escape; 5) |
| Max 1 | C2 | pass | co-44 | **no** (drag on the face orbited the cube; look cap) |
| Max 1 | C3 | pass | co-44 | **no** (stepped into the existing terminal, no new one) |
| Max 1 | T5 | pass (probe got no Super key) | co-44 | **no** (Tab list, flew to chora-probe, no look left) |
| Max 1 | T3 | pass | co-44 | yes (Super+a, "Hello") |
| Max 2 | C2 | pass | co-44, I5 | yes by drag (found, not told; pull key shown only after the re-dock) |
| Max 2 | C3 | **FAIL** "expected the focus with construct, holders ['xp']" | co-44 | yes by its words (Foot from Programs); same as r12 (co-77 family) |
| Max 2 | T5 | pass | co-44 | **no** (faces 1-4 turned, dark panel at the edge of face 4 not clicked) |
| Max 2 | T3 | pass | co-44 | yes (Helper in the top bar, "Hello") |
| Demo 1 | C1 | pass | co-44 | yes (6) |
| Demo 1 | C2 | pass | co-44 | **no** (Foot from Programs = pane; Alt+Escape: only Step back and All keys, no pull key; cap) |
| Demo 1 | T4 lock | pass (swaylock -f) | co-44 | no by eyes (co-57 harness stand-in); act.py rejects "Esc" (harness) |
| Demo 2 | C1 | pass | co-44 | yes (6) |
| Demo 2 | C2 | pass | co-44 | **yes, told** (Return into the face, Super+T, bar named "Super Alt 4 Pull out", Super+Alt+4 pulled, Super+Alt+2 gave it back) |
| Demo 2 | T4 lock | pass (swaylock -f) | co-44, I2 x2 + I3 x2 at Lock (co-51 family) | no by eyes (co-57) |

### Noise check (Max and the demo watcher twice)
Max: C2 not reached then reached (drag); C3 not reached then reached with the contract FAIL (run 1 stood in an existing terminal, so its pass is of a window that was not new); T5 not reached both; T3 reached both. Demo: C1 reached both; **C2 not reached then reached** (the two runs took different ways: run 1 started Foot from Programs with the cube stood back, which makes a pane; run 2 stepped into the face and pressed Super+T); T4 contract pass both, not reached by eyes both. The persona verdict varies between runs on C2, C3; the contracts differ only on Max C3.

### Fix check
| Row | Result | Capture / state dump |
|---|---|---|
| co-75 (Super+x in a face closes the focused face window, never a parked pane) | **holds by hand 5/5**: row Terminal (pane), Super+c, Return (face 0), Super+t, Super+x: surfaces 2 -> 1, the pane stays (panes 1), focus xp-icons. **Persona: not reached** (no persona pressed Super+x) | hand-co75.log |
| co-76 (in a pane flown into, after the hints something leads to the key list or out) | **holds by hand 5/5** (9 s after the click on the Terminal row: bar "Super F1 All keys / Alt Esc Step back", hint ids step-back and keys have rects). Persona: explorer T1 saw both after Foot; Max 1 T5 and explorer T2 used the Super F1 hint to open the list | hand-co76.log, persona-r13-captures/explorer/shots/007.png |
| co-79 (stepped out by Alt+Escape from a pane, a pull key is told; C2 reached by Max and demo) | **partial**. By the design (docs/cube-modes.md, co-68) the key is told where it acts: in a face with the keyboard in a window. Hand 5/5 for that state (row, Return, Super+t: bar "Super F1 All keys, Super Alt 4 Pull out, Alt Esc Step back", Super+Alt+4 pulls: panes [foot], hand-co79c.log). **FAIL 0/5 for the state of the ledger text, Alt+Escape out of a pane**: (a) pane flown into from the Terminal row: after Alt+Escape mode use, core focus construct-look and s1, bar empty (hand-co79a.log); (b) Foot from Programs with the cube stood back: after Alt+Escape mode navigate, bar "Click Fly in, Return, Ctrl Alt arrows, Super F1 All keys", no pull key, Super+Alt+4 does nothing (hand-co79b.log). Personas: demo 2 reached C2 **told, in a face** (pass); demo 1 stood in the pane state, not told, not reached (state at the end after Super+Alt+2: mode use front 0, panes [], construct holder none, xp holder xp s1, core focus s1; hints "Super Alt 4 Pull out ... Alt Esc Step back"); Max reached C2 only by drag (run 2), not told | demo1x-C2 dump, demo2 C2 dump (persona-r13-ends/demo1-C2.txt, demo2-C2.txt), shots demo1x/011.png |
| co-70 (the told pull acts) | holds: demo 2 pulled by the told key; hand co79c 5/5 told and pulls | |
| co-71 (key sheet closes) | holds: explorer T2 and Max 1 T5 closed it by Escape | |
| co-64, co-67 | not exercised (no pane-beside drag by a click hint; no program started while the cube stands back whose loss is checked, apart from demo 1 Foot) | |
| co-65 | partly: explorer T1 Foot from Programs full size, pass; Max 2 C3 Foot opened in the XP face, contract FAIL (co-77) | |

### Hand runs, 5x each (hand.sh; binaries 62a04be0; fresh compositor per run, fuzzel cache cleared)
| Row | Steps | Rate |
|---|---|---|
| co-75 | Terminal row, Super+c, Return, Super+t, Super+x | face window closed, pane stays 5/5 |
| co-76 | Terminal row, wait 9 s | step-back and keys hints present 5/5 |
| co-79 (face, keyboard in a window) | row, Return, Super+t, bar, Super+Alt+4 | told 5/5, pulls 5/5 |
| co-79 (Alt+Escape from a pane, walk) | Terminal row, Alt+Escape | told 0/5, bar empty |
| co-79 (Alt+Escape from Foot started from Programs, cube stood back) | row, Programs, Foot, Alt+Escape | told 0/5 |

### New rows proposed (no ledger edit here)
1. **co-79 remains open for the path "step out of a pane"**: the demo watcher who starts Foot from Programs (a pane) and leaves it by Alt+Escape is told no pull key (0/5 by hand, demo 1 persona); the fix holds only for a face with the keyboard in a window (5/5, demo 2 reached). Check: hand runs; no gate cell for the pane path. Design question: the pane has nothing in a face to pull (co-68), so the way for this person is not the pull key; what tells them to step into a face first? MAJOR (reach), not new but not closed.
2. Max C3 FAIL repeats (co-77, known MINOR); T5 probe still not found (co-66).
3. act.py rejects "Esc" while the key list and bar print "Esc" (harness).

### triage.py (`python3 triage.py --format feedback/2026-11-25-p2r13co-*.md`: FORMAT OK; the r13 lines of the full output)
```
2026-11-25-p2r13co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
2026-11-25-p2r13co-demo.md: Lock click: I2 xp shows s2, s3 with no box, I3 (co-51 family) (0 days)
2026-11-25-p2r13co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
2026-11-25-p2r13co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
2026-11-25-p2r13co-demo.md: xp face contrast rows (I6) (0 days)
2026-11-25-p2r13co-explorer.md: xp face contrast rows (I6) (0 days)
2026-11-25-p2r13co-max.md: xp face contrast rows (I6) (0 days)
2026-11-25-p2r13co-max.md: Max2 C3: Foot from Programs in a face with an earlier window: focus not with construct (contract red; co-77/co-72 family) (0 days)
2026-11-25-p2r13co-demo.md: C2 run 1: Foot from Programs is a pane; Alt+Escape from it leaves a pane alone at the screen edge with no pull key told (0 days)
2026-11-25-p2r13co-demo.md: C2 run 2: told in the face and pulled (0 days)
2026-11-25-p2r13co-demo.md: T4: Lock gives no lock the persona can read; act.py rejects the key name Esc (0 days)
2026-11-25-p2r13co-demo.md: C1: Return and Alt+Escape found from the bar (0 days)
2026-11-25-p2r13co-explorer.md: C1: cube, step in, step out reached (0 days)
2026-11-25-p2r13co-explorer.md: T1: terminal through Programs reached (0 days)
2026-11-25-p2r13co-explorer.md: T2: key list by Super+F1 from the hint bar (0 days)
2026-11-25-p2r13co-max.md: C2 run 1: a plain drag on the cube turns it, the window does not leave (0 days)
2026-11-25-p2r13co-max.md: C2 run 2: pane moved by drag, back onto the XP face; pull key shown only afterwards (0 days)
2026-11-25-p2r13co-max.md: C3 run 1: clicking the Terminal label does nothing; a click on the panel flies into the old terminal (0 days)
2026-11-25-p2r13co-max.md: T5: the Probe window is not found by faces or names, only by the Tab list (co-66 family) (0 days)
2026-11-25-p2r13co-max.md: T3: helper only by Super+A, found in the key list; a top bar Helper entry worked in run 2 (0 days)
```

### Stop rule (p2-plan.md)
Round 13 is **not met** for construct. Contracts: 16 of 17 green; the red is Max run 2 C3 (a known co-77 family, not new). Reached by the personas: C1, T1, T2, T3 yes; C2 reached by Max 2 and demo 2, not by Max 1 and demo 1; C3 by Max 2 (contract red) and not by Max 1; T5 not reached in two runs (co-66); T4 by eyes not (co-57 harness). **No new check-backed BLOCK or MAJOR**; the fix checks: co-75 holds by hand 5/5 (persona not reached), co-76 holds by hand 5/5 and on screen for personas, co-79 holds for a face with a window (demo 2 reached it) and not for the step out of a pane (0/5, demo 1). No ledger edit here.
