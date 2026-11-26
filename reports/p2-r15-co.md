# p2-r15-co: RC 2 persona round 15 on construct

## r15
Lane: cloud, branch claude/p2-r15-co at 57ce1480 (session head 7c6955c8 + claude/co-reach-r6 bcecb671, an ancestor of HEAD; holds co-reach r6 and co-drop r1-r4). Run only, no code change. Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` ok; smoke `begin smoke` + `check.py end smoke` ("no contract for smoke"). **Binaries of the round: sha 57ce14805c74d738db8b23b0a89d6060e00eb803** (/tmp/persona-round/bin, one snapshot).

Setup as r14: construct 1280x800, PERSONA_CHECK=1, helper transcript seeded (one agent entry), 2 slots, one `begin` per goal, `check.py end` after it (output in media/2026-11-26/persona-r15/ends/), one haiku Agent per goal (act.py only), probe started once per Max run before T5. **The task words are kept this time** ($P/hand/goals.txt); r14's were not stored, so these are fresh, same in both runs; the Max C2 and demo C2 words say to start a window first when none is open (r14's demo goal named none). Caps written before the run (media/2026-11-26/persona-r15/hand/caps.txt). Used (actions/looks): explorer 17/10; Max 1 36/20; Max 2 37/19; demo 1 21/13; demo 2 31/15. **No cap breached**, no retries past 3, no coaching. Every task of every persona ran; all 17 contract lines read: **16 pass, 1 FAIL (Max run 2 T3)**. Captures: media/2026-11-26/persona-r15/captures/ (breach evidence pngs dropped; breaches.jsonl kept); hand runs media/2026-11-26/persona-r15/hand/ (hand.sh with the new co79d, logs, key sheet captures).
Non-I6 breaches: co-44 I2 only (every persona). No I3, no I5, no other invariant.

### Per task
| Persona / run | Task | (a) contract | (b) breaches (not I6) | (c) reached |
|---|---|---|---|---|
| Explorer | C1 | pass | co-44 | yes (7 actions, all by mouse; the row click opened the cube) |
| Explorer | T1 | pass | co-44 | yes (Programs, Foot; 5) |
| Explorer | T2 | pass | co-44 | yes (hint bar All keys, Close; 5) |
| Max 1 | C2 | pass | co-44 | **no** (drag from the Terminal face put a foot window on the front; no put-back, look cap) |
| Max 1 | C3 | pass | co-44 | **no** (Foot from Programs fills the view, co-65 by design) |
| Max 1 | T5 | pass (probe got no Super key) | co-44 | **no** (title unreadable; never pressed Super+t) |
| Max 1 | T3 | pass | co-44 | yes (top bar Helper, "Hello"; 12) |
| Max 2 | C2 | pass | co-44 | **no** (drag jumped the view, window gone from sight) |
| Max 2 | C3 | pass | co-44 | **no** (Super+Return nothing, start button Terminal fills the view) |
| Max 2 | T5 | pass | co-44 | **no** (Super+t pressed, probe not identified) |
| Max 2 | T3 | **FAIL** "the helper never worked" | co-44 | **no** (typed into the probe window left from T5; persona slip, see feedback) |
| Demo 1 | C1 | pass | co-44 | yes (7; cube row, Return, Alt+Escape) |
| Demo 1 | C2 | pass | co-44 | **no** (XTerm from Programs left no window, look cap) |
| Demo 1 | T4 lock | pass (swaylock -f) | co-44 | yes (top bar Lock, "Screen locked" face; 3) |
| Demo 2 | C1 | pass | co-44 | yes (8) |
| Demo 2 | C2 | pass | co-44 | **no** (Foot, Alt+Escape, click the pane "Fly in"; no pull key told; look cap) |
| Demo 2 | T4 lock | pass (swaylock -f at Super+Shift+Escape) | co-44 | **no** by eyes (the key lock worked but the key list overlay hid the lock screen; Super+L nothing) |

### Noise check (Max and the demo watcher twice)
Max: C2 not reached both (drag moves the window out but the persona does not get it back; contracts pass); C3 not reached both (Foot fills the view, co-65); T5 not reached both (title unreadable, probe outside the cube ring; r14: reached once); T3 reached then not (run 2 typed into the probe left from T5, contract red = persona slip, run 1 by the top bar Helper). Demo: C1 reached both; C2 not reached both (a window is started now, but the pane state tells no pull key); T4 reached then not (run 2 pressed the told key Super+Shift+Escape, the lock worked per calls.log, the overlay hid it). Contracts identical across runs except T3.

### Fix check
| Row | Result | Capture / state dump |
|---|---|---|
| co-75 (Super+x in a face closes the focused face window, never a parked pane) | **holds by hand 5/5** (surfaces 2 -> 1, pane foot stays, panes 1). Persona: not reached (no persona pressed Super+x) | media/2026-11-26/persona-r15/hand/hand-co75.log; dump after: "mode use front 0, panes ['foot'], core focus ['xp-icons']" |
| co-76 (in a pane flown into, after the hints something leads to the key list or out) | **holds by hand 5/5** (step-back and keys hints present after 9 s). Personas: explorer T2 and Max 1 T3 left the pane by the hint bar All keys / Alt Esc | hand-co76.log, captures/explorer/shots/009.png |
| co-79 (stepped out of a pane by Alt+Escape, a pull key is told; C2 reached) | **pane path, direct pull key: FAIL 0/5 (still)**. Terminal row, Alt+Escape: mode use, front 0, panes [foot], core focus ['construct-look', 's1'], bar "Super C Cube, Return Fly in, F1 All keys, Super T Terminal, W A S D Walk" (r14: no Super C Cube), no "Pull out", Super+Alt+4 does nothing. **New in r6: the bar now tells Super C Cube; the told way 5/5 (co79d): Alt+Escape, Super+c (bar "Click Fly in ... Super Alt 2 Into face"), Return into the face (bar "Alt Esc Step back", no window in it), Super+t (bar "Super Alt 4 Pull out" told 5/5), Super+Alt+4 pulls: panes [foot, foot-2], core focus s2, 5/5.** Face with a window, keyboard in it (co79c): told 5/5 and pulls 5/5. Personas: **C2 not reached by any of 4 persona runs**; none was told by a pull key. Max found the pull by dragging (window to the front, no put-back); demo 2 stood in the pane state | hand-co79.log, hand-co79c.log, hand-co79d.log, hand/co79-pane-bar.png, captures/demo2/shots/011.png |
| co-drop (a drop or put stood back sends no key to a hidden window) | not exercised by a persona; gate case `turned`/`side` stays the check | |
| co-drop key sheet at 1024x768 shows every row text whole | **holds 5/5 by hand** in walk (101 texts, none with "…", sheet x 72 y 35 w 880 h 698) and 5/5 in cube mode (113 texts, none cut, x 32 y 32 w 960 h 722, ends 754 of 768, rcdr4-1 known) | hand/keysheet-walk-1024x768.png, keysheet-cube-1024x768.png, hand-keys.log, hand-keyscube.log |
| co-70 (the told pull acts) | holds by hand (co79c, co79d 5/5); no persona pulled by the told key | |
| co-71 (key sheet closes) | holds: explorer T2 closed it by Close | captures/explorer/shots/010.png |
| co-64, co-67 | not exercised | |
| co-65 | holds: Max 1 and 2 C3, Foot from Programs took the whole view, contract pass | max1 shots/009.png |

### Hand runs, 5x each (hand.sh; binaries 57ce1480; fresh compositor per run, fuzzel cache cleared)
| Row | Steps | Rate |
|---|---|---|
| co-75 | Terminal row, Super+c, Return, Super+t, Super+x | face window closed, pane stays 5/5 |
| co-76 | Terminal row, wait 9 s | step-back and keys hints present 5/5 |
| co-79 (Alt+Escape from a pane, pull key told) | Terminal row, Alt+Escape | pull key told **0/5**; Super C Cube told 5/5 |
| co-79 (pane, then the told way through the cube) | Terminal row, Alt+Escape, Super+c, Return, Super+t, Super+Alt+4 | told after Super+t 5/5, pulls 5/5 |
| co-79 (face, keyboard in a window) | row, Return, Super+t, bar, Super+Alt+4 | told 5/5, pulls 5/5 |
| key sheet 1024x768 walk | Super+F1 | no cut text 5/5 |
| key sheet 1024x768 cube | Super+c, Super+F1 | no cut text 5/5, sheet inside the output 5/5 |

### New rows proposed (no ledger edit here)
1. co-79 pane path: r6 tells Super C Cube from the pane (5/5) and the chain to a told pull key works (5/5 by hand), but no persona found it: C2 stays 0 of 4 by persona. By the strict wording (a pull key told right after Alt+Escape from a pane) it is still 0/5. Open, MAJOR (reach) unless the PM rules the told chain enough; no gate cell for the chain.
2. T4 by the key list: Super+Shift+Escape locks (swaylock -f logged) but the key list overlay stays on top, so the person sees no lock; UX, verify.
3. Max T3 after T5: the probe window stays focused in the cube and takes the typing; harness/ordering, not a product red.
4. act.py rejects "Esc" (harness, known).

### triage.py (`python3 triage.py --format feedback/2026-11-26-p2r15co-*.md`: FORMAT OK; the r15 lines of the full output)
```
  2026-11-26-p2r15co-demo.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-26-p2r15co-explorer.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-26-p2r15co-max.md: Cube stood back: I2 gaps rows (co-44, known) (0 days)
  2026-11-26-p2r15co-demo.md: xp face contrast rows (I6, known) (0 days)
  2026-11-26-p2r15co-explorer.md: xp face contrast rows (I6, known) (0 days)
  2026-11-26-p2r15co-max.md: xp face contrast rows (I6, known) (0 days)
  2026-11-26-p2r15co-max.md: T3 run 2: contract red, the persona never reached the helper (0 days)
  2026-11-26-p2r15co-demo.md: C2 both runs: a persona told "make sure a window is open" starts one, then does not find the way to pull it (0 days)
  2026-11-26-p2r15co-demo.md: T4 run 2: Super+L does nothing, the key list names Super+Shift+Esc, act.py cannot press it as "Esc", and Super+Shift+Escape did not lock (0 days)
  2026-11-26-p2r15co-demo.md: C1 both runs: cube row, Return, Alt+Escape reached; act.py rejects "Esc" (0 days)
  2026-11-26-p2r15co-explorer.md: C1: cube row, Step in by mouse, Step back by clicking the hint: reached in 7 actions (0 days)
  2026-11-26-p2r15co-explorer.md: T1: terminal is called Foot in Programs, no plain Terminal entry (0 days)
  2026-11-26-p2r15co-explorer.md: T2: key list found by the hint bar All keys and closed by Close (0 days)
  2026-11-26-p2r15co-max.md: C2 run 1: a drag from the Terminal face to the front moves the window out, but Alt+Escape then leaves it floating on the front and the persona cannot put it back (0 days)
  2026-11-26-p2r15co-max.md: C2 run 2: the drag from the Terminal face jumps the view to the front, the terminal is gone from sight (0 days)
  2026-11-26-p2r15co-max.md: C3 both runs: the terminal fills the view, the persona never sees a small pane next to the cube (0 days)
  2026-11-26-p2r15co-max.md: T5: the Probe window has no title bar and sits outside the cube ring (co-66 family) (0 days)
  2026-11-26-p2r15co-max.md: T3 run 1: Helper reached by the top bar, answer Hello (0 days)
  2026-11-26-p2r15co-max.md: act.py rejects "Esc" while hints print "Esc" (harness) (0 days)
```

### Stop rule (p2-plan.md)
Round 15 is **not met** for construct. Contracts: 16 of 17 green; the one FAIL is Max 2 T3 (persona typed into the probe window, no product red). Reached by the personas: C1, T1, T2 yes; T3 yes in one of two runs; T4 yes in one of two (the other pressed the told key and the lock worked, the overlay hid it); T5 and C3 not reached by Max in either run (contracts pass); **C2 not reached in any of four persona runs, so design/rc.md row 2 is still not met for construct**. **No new check-backed BLOCK or MAJOR.** Fix checks: co-75, co-76, co-drop key sheet hold 5/5; co-79 pane path: direct pull key 0/5, told chain via Super C Cube 5/5, face 5/5. No ledger edit here.
