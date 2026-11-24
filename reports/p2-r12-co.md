# p2-r12-co: RC 2 persona round 12 on construct

## r12
Lane: cloud, branch claude/p2-r12-co at 280b9dcf (co-pull r2 head). Run only, no code change. Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` ok; smoke `begin smoke` + `check.py end smoke` on construct ("no contract for smoke"). **Binaries of the round: sha 280b9dcf5821e41784aeeb7cfe0b36041e201989** (/tmp/persona-round/bin, one snapshot).

Setup as r11: construct 1280x800, PERSONA_CHECK=1, helper transcript seeded (one agent entry), explorer and Max together, demo after, one `begin` per goal, `check.py end` after it (output in media/2026-11-24/persona-r12-ends/), one haiku Agent per goal (act.py only), probe started once before Max T5. Task words are written fresh in a user's words (the r11 wording was not kept in the log). Caps written before the run (media/2026-11-24/persona-r12-hand/caps.txt, same as r11): explorer C1 12/6, T1 14/6, T2 19/8; Max C2 14/6, C3 9/4, T5 10/4, T3 12/6; demo C1 15/7, C2 15/7, T4 15/6. Used: explorer 20 actions, 11 looks; Max 33, 17; demo 31, 17. **No cap breached**, no retries past 3, no coaching, no tool outside act.py and Read. Every task of every persona ran, all 10 contract lines read: **9 pass, 1 FAIL (Max C3)**. Captures: media/2026-11-24/persona-r12-{explorer,max,demo}/, hand runs persona-r12-hand/.
Non-I6 breaches: co-44 I2 (explorer 26, Max 42, demo 10 rows), I2 "xp shows s3 on VIRTUAL-1, no box" x1 at a Lock click (demo, co-51 family). No I3, no I5.

### Explorer
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | co-44 | yes (row, Return, Alt+Escape; 8 actions) |
| T1 | pass | co-44 | yes (Programs, Foot; 5) |
| T2 | pass | co-44 | yes (Super+F1 from the hint bar, Escape closed the sheet; 7) |

### Max
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C2 | pass | co-44 | by its words with a caveat: pane moved by drag (found), the pull key not pressed; "Alt 4 Pull out" showed on the bar only after it re-docked |
| C3 | **FAIL** "expected the focus with construct, holders ['xp']" | co-44 | yes by its words (Foot from Programs, readable at full size), but Foot sits in the XP face, not as a pane |
| T5 | pass (probe got no Super key) | co-44 | no (Probe not found on faces 1-3, a dark panel on face 4 clicked blind; co-66 family) |
| T3 | pass | co-44 | yes (Super+a, "Hello") |

### Demo watcher
| Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|
| C1 | pass | co-44 | yes (clicks on the row, "Return Step in", "Alt Esc Step back"; 6 actions) |
| C2 | pass | co-44 | **no** (Super+F1 over the face, Super+T, Escape left the sheet open, Close click closed it, Alt+Escape; never saw a pull key; look cap) |
| T4 lock | pass (stand-in swaylock x2) | co-44, I2 x1 at Lock (co-51 family) | no by its eyes (co-57 known) |

### Fix check
| Row | Result | Capture / state dump |
|---|---|---|
| co-70 (the told pull acts) | **holds by hand 5/5** (row, Return, Super+t, Alt+Escape: bar now `... Super Alt 4 Pull out`; Super+Alt+4 pulls: s1 owner construct, rect 290,150,700,500, 5/5, 0 non-I6 breaches). **Persona reach: not met**: Max found the pane by Super+c and dragged (found, not told); demo never reached the stood-back state (it left the face by Alt+Escape from the pane and saw `Alt Esc Step back` / `Super F1`, no pull key), so no persona was told or pulled | hr12-co70.log; demo state at the end: `mode use front 0, panes [foot], construct holder construct s1, xp holder none, core focus construct s1` (demo-C2-dump.txt) |
| co-71 (key sheet closes by Close, Esc, click outside; no click passes through) | **holds by hand: Close 5/5, Esc 5/5, click outside 5/5** (3 sessions at once, each fresh; closed = `open` empty, focus construct). Persona: explorer T2 closed it by Escape (pass); **demo C2: Escape did not close the sheet** when Super+T started a window while the sheet was open (Close click closed it): new, see below. "No click passes through": the click-outside left `open` empty and the focus with construct; the pane behind was not separately dumped (partly measured) | hr12-co71-{close,esc,out}.log; demo shots 007-010 |
| co-68 | holds (no pane beside and keyboard in a window: told and acts, hand runs 5/5; not exercised by a persona with a pane beside) | hr12-co70.log |
| co-64, co-65, co-67, co-61, co-62 | co-61 holds (explorer, demo C1 by hint buttons); co-65 holds (Foot from Programs, explorer, Max); co-62 holds; co-64 and co-67: **not exercised** this round (no pane-beside drag by a click hint named; Foot of Max C3 started in a face, see C3) | |

### Hand runs, 5x each (scripts persona-r12-hand/co70.sh, co71.sh; binaries 280b9dcf; fresh compositor per run, fuzzel cache cleared)
| Row | Steps | Rate |
|---|---|---|
| co-70 | row, Return, Super+t, Alt+Escape, hint, Super+Alt+4 | told 5/5, pulls 5/5 |
| co-71 close | row, Return, Super+F1, click Close | closes 5/5 |
| co-71 esc | same, Escape | closes 5/5 |
| co-71 out | same, click at (20,780) outside the sheet | closes 5/5 |

### New rows proposed (no ledger edit here)
1. **C3 red: Foot from Programs in a face with an earlier window does not give construct the focus (contract C3 FAIL, check: contract C3; MAJOR candidate, not yet hand-reproduced)**. Max C3 began in a face that already held the terminal of C2; Foot opened in the XP face (holder xp s3, construct none, cube navigate front 0). Dump: media/2026-11-24/persona-r12-ends/max-C3-dump.txt. Needs a hand run 5x before routing.
2. **Escape does not close the key sheet when a face window opened behind it (check: none, one capture read, demo shots 007-010; MINOR candidate, co-71 family)**; the hand runs without that window close 5/5.
3. **C2 for the demo watcher is still unreached (fourth round)**: the pull key is told only in the stood-back cube; a person who steps out of the face by Alt+Escape from a pane stays in the face (mode use) where no pull key is told. The family of co-68/co-70, now the demo's path. The fix of co-70 does not cover it.

### triage.py (`python3 triage.py --format feedback/2026-11-24-p2r12co-*.md`: FORMAT OK; the r12 lines of the full output)
```
  2026-11-24-p2r12co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r12co-demo.md: Lock click: I2 xp shows s3 with no box (co-51 family) (0 days)
  2026-11-24-p2r12co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r12co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-24-p2r12co-demo.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r12co-explorer.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r12co-max.md: xp face contrast rows (I6) (0 days)
  2026-11-24-p2r12co-demo.md: C1: cube, step in, step out reached (0 days)
  2026-11-24-p2r12co-explorer.md: C1: cube, step in, step out reached (0 days)
  2026-11-24-p2r12co-demo.md: C2: no pull key told on the way, never pulled (0 days)
  2026-11-24-p2r12co-max.md: C2: terminal moved by drag, the pull key never pressed (0 days)
  2026-11-24-p2r12co-max.md: C3: Foot from Programs: focus not with construct (contract red) (0 days)
  2026-11-24-p2r12co-explorer.md: T1: terminal through the Programs menu (0 days)
  2026-11-24-p2r12co-explorer.md: T2: key list found by the hint bar (0 days)
  2026-11-24-p2r12co-max.md: T3: helper chat by Super+A (0 days)
  2026-11-24-p2r12co-demo.md: T4: Lock gives the persona no lock it can read (0 days)
  2026-11-24-p2r12co-max.md: T5: the probe window cannot be found (0 days)
  2026-11-24-p2r12co-demo.md: Key sheet: Escape does not close it when a face window got the keyboard behind it (0 days)
```

### Stop rule (p2-plan.md)
Round 12 is **not met** for construct. Contracts: 9 of 10 green, **C3 red** (Max). Reached by the personas: C1 (explorer, demo), T1, T2, T3. Not reached: C2 (Max by drag only, found; demo not), T5 (probe not found, co-66 family), T4 lock by eyes (co-57 known); C3 reached by Max's words but the contract is red. Fix checks: co-70 holds by hand 5/5 (told and acts) but is not reached by a persona; co-71 holds by hand 5/5 for Close, Esc and click outside, and by one persona (explorer), with one persona miss (demo, Esc with a face window behind the sheet). New check-backed BLOCK or MAJOR: **one candidate, C3 contract red (check: contract C3), unreproduced by hand**; one check-none MINOR (Esc with a window behind the sheet); one UX gap (demo C2 path). No ledger edit here.
