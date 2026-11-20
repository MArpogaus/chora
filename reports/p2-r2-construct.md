
## r2
Lane: cloud, branch claude/p2-r2-co at 59efbc17 (no code changes; Step 0 unshallow, cargo check and build green). Binaries of the round: sha 59efbc1740062a4f1213a882d8cabc32601b53e4, snapshot /tmp/persona-round/bin. fuzzel not used (no launcher row read).
Setup: round.sh snapshot once, construct 1280x800, PERSONA_CHECK=1, 2 slots (explorer and Max together, the demo watcher after), helper transcript seeded, Max with `round.sh probe`. Haiku personas with act.py only; limits held (13, 35, 31 actions; 7, 18, 16 looks; no other tool). Driver slip: `check.py begin` once per persona before its goals, not per task (one persona run covers its goals in sequence), so a task left open by an earlier goal could still show in a later one; none did.

### Explorer (13 actions, 7 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 cube, in, out | pass | I7 x4 (first click) | yes, row then Alt+Escape; the row goes straight into face 1 |
| T1 start a program | pass | same | yes by Super+T |
| T2 key list | pass | same | yes by Super+F1 ("All keys") |

### Max (35 actions, 18 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C2 pull out and back | pass, nothing pulled | 0 | no (three drags did nothing) |
| C3 window as pane | pass | 0 | no (Tab, click, Return) |
| T5 no stray keys | **FAIL**: probe got keys 32, 16, 15 after Super+d, q, Tab | 0 | persona pressed them |
| T3 helper | pass | 0 | yes (answer is the seeded "Hello") |

### Demo watcher (31 actions, 16 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | I7 x4 (first click) | no (row and Super+C give a flat XP desktop) |
| C2 | **FAIL** "windows lost ['s2']" | I1 x23, I3 x21 (xp-owned window; checker lead of r1b) | no pull seen: **not reached** |
| T4 lock | pass | same | yes by Super+Shift+Escape (the Lock item is hidden in a face) |

C2 counts for no persona this round: Max pulled nothing; demo's red is the end-state "s2" with no pull seen (not a pass, not a reached). I7 x4 at the first click only in all runs: probably the faces not drawn at the first quiet snapshot, so checker noise, not an animating-look breach (no cube turn was read). I3/I13 reads during a cube turn: none seen. No launcher-only breach.

### Fix check (construct findings fixed since r1)
| Row | Reached | Holds | Capture |
|---|---|---|---|
| co-25 (no stray Super chords) | Max, probe, Super+d/q/Tab | **fail** with the cube open (probe on a face; the pane case is not reached) | media/2026-11-20/persona-r2-max/probe.report: key 125, 32, 16, 15. Known from co-p2-fix r2: the face case is `known: face manager swallow (xp lane)`; the persona's cube-open state is inferred from its steps, not read from the scene |
| co-26 (Super+F1 shows key list) | explorer, demo | pass (on a face and stood back: "All keys" / "CONSTRUCT - CUBE" list) | persona-r2-explorer shot 007; persona-r2-demo shot 012-014 |
| co-19 (F1 stood back) | explorer | pass (list opened with the cube stood back, names "stood back" lines) | explorer shot 007 |
| co-20 (Escape closes the cube) | not reached (the key is only listed) | - | explorer shot 007 |
| co-21 (seams closed) | not reached (no capture read) | - | - |
| co-16, co-18 (a drag turns the cube) | Max, three drags | not seen: nothing moved on the persona's pictures; unconfirmed whether the drags landed on a face | actions.jsonl of persona-r2-max |
| co-27 to co-32 | not reached (gate-only: overlay, 3d, hud, windows, flakes) | - | - |
| co-1 to co-4 | not reached (panes after an mc drag, cube xp crash, doors strip) | - | - |

### triage.py (construct section of `python3 triage.py`; the three files give 14 lines)
```
== construct / I1 (2)
  2026-11-20-p2r1b-demo.md: I1 and I3 on a window of the XP face (harness, to confirm) (0 days)
  2026-11-20-p2r2co-demo.md: I1 and I3 on a window of the cube's XP face (checker lead, harness to confirm) (0 days)
== construct / I7 (5)
  2026-11-20-p2r1-demo.md: I7 on every output through the whole run (0 days)
  2026-11-20-p2r1-explorer.md: I7 on every output through the whole run (0 days)
  2026-11-20-p2r1b-explorer.md: I7 once on all four outputs at the first click (0 days)
  2026-11-20-p2r2co-demo.md: I7 on all four outputs at the first click (0 days)
  2026-11-20-p2r2co-explorer.md: I7 on all four outputs at the first click (0 days)
== construct / contract C1 (2)
  2026-11-20-p2r1-demo.md: Every XP face says "did not start" while the notice says "The other faces still work" (0 days)
  2026-11-20-p2r1-explorer.md: The cube row leads to a "Starting XP" spinner, then a failure notice; the cube only came by Alt+Escape (0 days)
== construct / contract C2 (4)
  2026-11-20-p2r1-demo.md: A window cannot be pulled out of its desktop by the gesture a demo viewer tries (0 days)
  2026-11-20-p2r1b-max.md: Dragging a face of the cube does not pull its window out; C2 passes without a pull (0 days)
  2026-11-20-p2r2co-demo.md: Window cannot be pulled out by a drag on its title bar (0 days)
  2026-11-20-p2r2co-max.md: Dragging a face of the cube does not pull its window out; C2 passes without a pull (0 days)
== construct / contract T1 (1)
  2026-11-20-p2r1-explorer.md: Programs > Foot opens the helper chat instead of a terminal (0 days)
== construct / contract T4:lock (2)
  2026-11-20-p2r1-demo.md: Top-bar Lock click and Super+Shift+Escape show no lock to the viewer (0 days)
  2026-11-20-p2r1b-demo.md: Super+Shift+Escape shows no lock to the viewer (0 days)
== construct / contract T5 (2)
  2026-11-20-p2r1b-max.md: Super+D, Super+Q and Super+Tab reach the focused client (0 days)
  2026-11-20-p2r2co-max.md: Super+D, Super+Q, Super+Tab still reach the probe window (co-25 fix does not hold here) (0 days)
== construct / none (22)
  2026-11-20-p2r1-demo.md: Esc closes the key list and steps out of the pane too (0 days)
  2026-11-20-p2r1-explorer.md: Step into a face needs a click and Return; scroll does nothing (0 days)
  2026-11-20-p2r1-explorer.md: The chat panel stays open and covers the key list (0 days)
  2026-11-20-p2r1-max.md: Typed text in a pane goes to the walk keys, not the window (0 days)
  2026-11-20-p2r1-max.md: Super+D, Super+Q and Super+Tab reach a client as text (0 days)
  2026-11-20-p2r1-max.md: The helper panel opens only from the top bar (0 days)
  2026-11-20-p2r1b-demo.md: The cube row gives a plain XP desktop, and Super+C does nothing after it (0 days)
  2026-11-20-p2r1b-demo.md: A window cannot be taken out of its desktop by a drag onto a taskbar tile (0 days)
  2026-11-20-p2r1b-demo.md: Super+F1 in a face opens XP's Help and Support, not the construct key list (0 days)
  2026-11-20-p2r1b-explorer.md: The cube row steps into the XP face; the cube shows only after Alt+Escape (0 days)
  2026-11-20-p2r1b-explorer.md: Super+F1 does nothing visible with a terminal focused in a face (0 days)
  2026-11-20-p2r1b-explorer.md: The top bar is hidden inside a face, so Keys and Programs cannot be clicked (0 days)
  2026-11-20-p2r1b-max.md: Programs > Foot opens the helper chat instead of a terminal (again) (0 days)
  2026-11-20-p2r1b-max.md: A window in a pane: Return, Tab and right-click do nothing visible (0 days)
  2026-11-20-p2r2co-demo.md: "A cube of desktops" row and Super+C give a flat XP desktop, never the cube (0 days)
  2026-11-20-p2r2co-demo.md: The top bar, with Lock, is hidden while a window is open (0 days)
  2026-11-20-p2r2co-demo.md: The key list stays drawn over the locked screen (0 days)
  2026-11-20-p2r2co-explorer.md: The row "A cube of desktops" steps into face 1; the cube shows only after Alt+Escape (0 days)
  2026-11-20-p2r2co-explorer.md: Key list wording "stood back" and Esc against Alt+Esc (0 days)
  2026-11-20-p2r2co-max.md: A window as a pane: Tab, click, Return show nothing (0 days)
  2026-11-20-p2r2co-max.md: Programs > Foot opens the terminal full-screen; "Foot" row in Programs now gives a terminal (0 days)
  2026-11-20-p2r2co-max.md: Helper answers every question with "Hello" (0 days)
```
Files: feedback/2026-11-20-p2r2co-{explorer,max,demo}.md; captures media/2026-11-20/persona-r2-{explorer,max,demo}/. No ledger edits.

### Stop rule
Construct does not meet round 4: a new check-backed red remains, T5 (co-25 face case, probe keys 32, 16, 15; a known xp-lane item, so a duplicate of co-25 rather than a new row), and C2/C3 are not reached by any persona (no seen pull). C1 and T1 to T3 pass; the C2 contract still needs a seen-pull and the checker the xp child (I1/I3).
