# p2-r14-co: RC 2 persona round 14 on construct

## r14
Lane: cloud, branch claude/p2-r14-co at 4d8bbe36 (t153+154 merge; co-reach r5, co-drop r1-r4). Run only, no code change. Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` ok; smoke `begin smoke` + `check.py end smoke` ("no contract for smoke"). **Binaries of the round: sha 4d8bbe36891b4304709a929a72b7a0992e0806b8** (/tmp/persona-round/bin, one snapshot).

Setup as r13: construct 1280x800, PERSONA_CHECK=1, helper transcript seeded (one agent entry), 2 slots, one `begin` per goal, `check.py end` after it (output in media/2026-11-26/persona-r14/ends/), one haiku Agent per goal (act.py only), probe started once per Max run before T5. Task words fresh in a user's words (r13's were not kept verbatim; the same in both runs; see feedback files). Caps written before the run (media/2026-11-26/persona-r14/hand/caps.txt) as r13. Used: explorer 19 actions, 11 looks; Max 1 32/18; Max 2 32/18; demo 1 29/15; demo 2 27/15. **No cap breached**, no retries past 3, no coaching. Every task of every persona ran; all 17 contract lines read: **17 pass, 0 FAIL** (Max run 2 C3 passes this time). Captures: media/2026-11-26/persona-r14/captures/; hand runs media/2026-11-26/persona-r14/hand/ (hand.sh, logs, two key sheet captures).
Non-I6 breaches: co-44 I2 (every persona), one I2 "xp shows s1 on VIRTUAL-1, no box" at the Lock click (demo 1, co-51 family). No I3, no I5.

### Per task
| Persona / run | Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|---|
| Explorer | C1 | pass | co-44 | yes (9 actions; row click only highlighted, Super+c opened) |
| Explorer | T1 | pass | co-44 | yes (Programs, Foot; 5) |
| Explorer | T2 | pass | co-44 | yes (Super+F1 from the bar, Close; the view then showed the cube) |
| Max 1 | C2 | pass | co-44 | **no** (drag from the side face; look cap) |
| Max 1 | C3 | pass | co-44 | **no** (xp face, Super+Return and Super+F1 nothing, start button no menu) |
| Max 1 | T5 | pass (probe got no Super key) | co-44 | yes (taskbar chora-probe, Super+t) |
| Max 1 | T3 | pass | co-44 | yes (pill, "Hello") |
| Max 2 | C2 | pass | co-44 | **no** (drag panned the view) |
| Max 2 | C3 | pass | co-44 | **no** (Foot from Programs filled the view, no pane look) |
| Max 2 | T5 | pass | co-44 | **no** (probe not found in 4 looks) |
| Max 2 | T3 | pass | co-44 | yes (Helper in the bar, "Hello") |
| Demo 1 | C1 | pass | co-44 | yes (8; Alt+Esc rejected by act.py, Alt+Escape) |
| Demo 1 | C2 | pass | co-44 | **no** (nothing open after C1, no window on any face; cap) |
| Demo 1 | T4 lock | pass | co-44, I2 at Lock (co-51 family) | yes by eyes (Screen locked in the taskbar, Start then Lock) |
| Demo 2 | C1 | pass | co-44 | yes (7) |
| Demo 2 | C2 | pass | co-44 | **no** (same: no window to pull; cap) |
| Demo 2 | T4 lock | pass | co-44 | yes by eyes |

### Noise check (Max and the demo watcher twice)
Max: C2 not reached both (a drag on the cube does not pull; r13 run 2 reached by drag); C3 not reached both, contracts pass both (run 1 never got a terminal started by the persona, the contract passes on the terminal of C2); T5 reached then not (co-66 family, the probe is found by the taskbar entry or not at all); T3 reached both. Demo: C1 reached both; C2 not reached both (the persona has no window because the goal text names none and C1 left the cube empty); T4 reached both by eyes. Contracts identical across runs; the verdicts differ only on T5.

### Fix check
| Row | Result | Capture / state dump |
|---|---|---|
| co-75 (Super+x in a face closes the focused face window, never a parked pane) | **holds by hand 5/5** (surfaces 2 -> 1, pane stays, panes 1). Persona: not reached (no persona pressed Super+x) | media/2026-11-26/persona-r14/hand/hand-co75.log |
| co-76 (in a pane flown into, after the hints something leads to the key list or out) | **holds by hand 5/5** (step-back and keys hints present after 9 s). Persona: explorer T2 used the Super F1 hint bar; Max 1 and 2 saw the bar | hand-co76.log, captures/explorer/shots/009.png |
| co-79 (stepped out of a face by Alt+Escape from a pane, a pull key is told; C2 reached) | **pane path FAIL 0/5** (still): Terminal row, Alt+Escape: mode use, front 0, panes [foot], core focus construct-look and s1, bar now "Return Fly in, F1 All keys, Super T Terminal, W A S D Walk" (r13 it was empty), no pull key, Super+Alt+4 does nothing (bar empty after). Face with a window, keyboard in it (hand co79c) **told 5/5 and pulls 5/5**. Personas: demo 1 and 2 and Max 1 and 2 did not reach C2 (told: no window existed; Max by drag only) | hand-co79.log, hand-co79c.log |
| co-drop (a drop or put stood back sends no key to a hidden window) | not exercised by a persona (no drop stood back); gate case `turned`/`side` stays the check (co-drop r3) | |
| co-drop key sheet at 1024x768 shows every row text whole | **holds 5/5 by hand** in walk and 5/5 in cube mode: no text with "…" among 113 texts; cube mode sheet x 32, y 32, w 960, h 722 (ends 754 of 768, rcdr4-1 known), Esc Close in the foot row; all rows whole incl. wrapped ones; the LISTENING pill still covers the scope label "CONSTRUCT · CUBE" (rcdr4-3 known) | hand/keysheet-cube-1024x768.png, keysheet-walk-1024x768.png, hand-keys.log, hand-keyscube.log |
| co-70 (the told pull acts) | holds by hand (co79c 5/5); no persona pulled | |
| co-71 (key sheet closes) | holds: explorer T2 closed it by Close; the view then showed the cube, not the terminal (ux finding) | explorer shots/011.png |
| co-64, co-67 | not exercised | |
| co-65 | Max 2 C3: Foot from Programs took the whole view, cube gone, contract pass | max2 shots/010.png |

### Hand runs, 5x each (hand.sh; binaries 4d8bbe36; fresh compositor per run, fuzzel cache cleared)
| Row | Steps | Rate |
|---|---|---|
| co-75 | Terminal row, Super+c, Return, Super+t, Super+x | face window closed, pane stays 5/5 |
| co-76 | Terminal row, wait 9 s | step-back and keys hints present 5/5 |
| co-79 (Alt+Escape from a pane) | Terminal row, Alt+Escape | pull key told 0/5 |
| co-79 (face, keyboard in a window) | row, Return, Super+t, bar, Super+Alt+4 | told 5/5, pulls 5/5 |
| key sheet 1024x768 walk | Super+F1 | no cut text 5/5 |
| key sheet 1024x768 cube | Super+c, Super+F1 | no cut text 5/5, sheet inside the output 5/5 |

### New rows proposed (no ledger edit here)
1. co-79 pane path unchanged: the person who flies into a pane and leaves it by Alt+Escape is told no pull key (0/5). Open, MAJOR (reach) as in r13; no gate cell.
2. C2 by a persona without a window: goal "pull a window out of a side" with an empty cube leaves four runs with no window and no hint to start one; nothing in the cube bar names a first step. UX, not a check red.
3. Closing the key list by its Close button from a terminal view lands on the cube overview (explorer T2). UX, verify.
4. act.py rejects "Esc" (harness).

### triage.py (`python3 triage.py --format feedback/2026-11-26-p2r14co-*.md`: FORMAT OK; the r14 lines of the full output)
```
  2026-11-26-p2r14co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-26-p2r14co-demo.md: Lock click: I2 xp shows s1 on VIRTUAL-1 with no box (co-51 family), run 1 (0 days)
  2026-11-26-p2r14co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-26-p2r14co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-26-p2r14co-demo.md: xp face contrast rows (I6, known) (0 days)
  2026-11-26-p2r14co-explorer.md: xp face contrast rows (I6, known) (0 days)
  2026-11-26-p2r14co-max.md: xp face contrast rows (I6, known) (0 days)
  2026-11-26-p2r14co-max.md: C3 run 2: Foot from Programs fills the view, the cube is gone, no small pane (0 days)
  2026-11-26-p2r14co-demo.md: C2 both runs: with C1 done nothing is open, so there is no window to pull out and nothing says to start one first (0 days)
  2026-11-26-p2r14co-demo.md: C1: cube row then Return then Alt+Escape works, act.py rejects the key name Esc (0 days)
  2026-11-26-p2r14co-demo.md: T4: Super+L does nothing, Start then Lock works; a dark rectangle remains at the top left after Lock (0 days)
  2026-11-26-p2r14co-explorer.md: C1: the first click on the cube row only highlights; Super+C opens it (0 days)
  2026-11-26-p2r14co-explorer.md: C1: cube, step in, step out reached (0 days)
  2026-11-26-p2r14co-explorer.md: T1: terminal is called Foot in Programs, no plain Terminal entry (0 days)
  2026-11-26-p2r14co-explorer.md: T2: closing the key list by its Close button left the view on the cube, not the terminal (0 days)
  2026-11-26-p2r14co-max.md: C2 run 1: a drag from a side face turns the cube, the window does not leave (0 days)
  2026-11-26-p2r14co-max.md: C2 run 2: Terminal pane sits left of the faces, a drag pans the view, Ctrl+Alt+Left never brings it to the front (0 days)
  2026-11-26-p2r14co-max.md: C3 run 1: on an xp face nothing starts a pane: Super+Return and Super+F1 do nothing, the start button opens no menu (0 days)
  2026-11-26-p2r14co-max.md: T5: probe window found by the taskbar entry in run 1, not in run 2 (co-66 family) (0 days)
  2026-11-26-p2r14co-max.md: T3: Helper reached by the pill (run 1) and the top bar (run 2), answer Hello (0 days)
```

### Stop rule (p2-plan.md)
Round 14 is **not met** for construct. Contracts: 17 of 17 green. Reached by the personas: C1, T1, T2, T3, T4 yes; T5 yes in one of two runs (co-66); C2 not reached in any of four persona runs (the demo goal had no window; Max drags do not pull) and C3 not reached by Max in either run (contracts pass). **No new check-backed BLOCK or MAJOR.** Fix checks: co-75 and co-76 hold 5/5; the co-drop key sheet at 1024x768 holds 5/5; co-79 holds in a face with a window and still fails for the step out of a pane (0/5). No ledger edit here.
