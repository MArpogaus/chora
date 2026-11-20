# p2-r3-co (RC 2 persona round 3, construct)

## r3
Lane: cloud, branch claude/p2-r3-co at bbaad12b (no code changes; Step 0 unshallow, cargo check and build green). Binaries of the round: sha bbaad12bbd7b8c053a08b27c8f39008db5e7f2aa, snapshot /tmp/persona-round/bin. fuzzel not used.
Setup: round.sh snapshot once, construct 1280x800, PERSONA_CHECK=1, 2 slots (explorer and Max together, the demo watcher after), helper transcript seeded (one agent entry), Max with `round.sh probe`, `check.py begin` once per persona. Haiku personas with act.py only; limits held (9, 30, 34 actions; 5, 15, 17 looks).

### Explorer (9 actions, 5 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 cube, in, out | pass | I7 x4 (first click) | yes (row goes into face 1, Alt+Escape shows the cube) |
| T1 start a program | pass | same | yes by Super+T |
| T2 key list | pass | same | yes by Super+F1 |

### Max (30 actions, 15 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C2 pull out and back | pass | I3 x23 from the first drag | no: a drag on the probe in the walk room lifted the pane cut off at the top; three put-back drags did nothing. Not a face pull (no cube open) |
| C3 window as pane | pass | same | no (scroll, orbit: no clear view) |
| T5 no stray keys | **pass**: probe report has pointer lines only, no key | same | done (Super+d, q, Tab pressed after a click on the probe) |
| T3 helper | pass | same | yes ("Hello", the stand-in's answer) |

### Demo watcher (34 actions, 17 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | I7 x4 (first click) | not seen (flat XP desktop; cube after step back never read) |
| C2 | **FAIL** "windows lost ['s2']" | I1 and I3 x48 (xp-owned s1, s2, xp-menu: checker does not read the xp child) | no (drag onto a taskbar tile, Super+Alt+1 x2: nothing pulled) |
| T4 lock | pass | same | yes by Super+Shift+Escape |

C2 counts for no persona: Max pulled no face window, demo's red is the end state s2 with the terminal in place on the last shot. I7 at the first click only. I1/I3 of demo are the known checker lead (r1b, r2). New against r2: Max's I3 x23 is a different case (construct's own pane): "the focused s1 is no shown window of construct" after a pane drag lifted the window out of the picture; a check-backed lead, not confirmed against a scene read.

### Fix check (construct findings fixed since r2)
| Row | Reached | Holds | Capture |
|---|---|---|---|
| co-25 (no stray Super chords) | Max: click on the probe (walk room), Super+d, q, Tab | **pass** (r2: fail; the same persona path, probe report with no key 125, 32, 16, 15) | media/2026-11-20/persona-r3-max/probe.report; T5 pass |
| co-33 (one-screen pull hint) | demo: one title drag (200,22 to 680,490, onto the taskbar, not the edge); Max: no title drag (pane drag) | **not reached**: act.py drags are one action, the button is up before any look, so a persona cannot see a hint that shows only while the button is down; the capture after the drag shows the taskbar tooltip "Desktop 2 of 4", no hint. No persona then asked the helper to pull (`cube.pull` not tried) | persona-r3-demo shot 008, actions.jsonl line 14; cell pull is the proof |
| co-26 (Super+F1 key list) | explorer, demo | pass (sheet "Construct · Cube") | persona-r3-explorer shot 005 |
| co-16, co-18 (a drag turns the cube) | Max (walk room, no cube) and demo (face) | not reached (no cube stood back dragged) | - |
| co-19, co-20 (F1 and Escape stood back) | demo pressed Escape on the sheet: stood the cube back and left the sheet open | not reached cleanly (new ux finding: Escape and Close on the sheet) | persona-r3-demo shots 010 to 014 |
| co-21 (seams) | not reached | - | - |
| co-27 to co-32, co-1 to co-4 | not reached (gate-only) | - | - |

### triage.py (construct section; the three files give 15 lines; mc, tiling, xp sections are other lanes)
```
== construct / I1 (3)
  2026-11-20-p2r3co-demo.md: I1 and I3 on a window of the cube's XP face (checker lead) (0 days)
== construct / I3 (2)
  2026-11-20-p2r3co-max.md: I3: the focused probe window s1 is no shown window of construct (0 days)
  2026-11-20-p2r3co-max.md: A pane window dragged in the walk room floats cut off; no way back by the mouse (0 days)
== construct / I7 (7)
  2026-11-20-p2r3co-demo.md: I7 on all four outputs at the first click (0 days)
  2026-11-20-p2r3co-explorer.md: I7 on all four outputs at the first click (0 days)
== construct / contract C1 (2)
== construct / contract C2 (5)
  2026-11-20-p2r3co-demo.md: C2 red: windows lost ['s2'] (0 days)
== construct / contract T1 (1)
== construct / contract T4:lock (2)
== construct / contract T5 (3)
  2026-11-20-p2r3co-max.md: Super+d, Super+q, Super+Tab do not reach the probe (co-25 holds) (0 days)
== construct / none (32)
  2026-11-20-p2r3co-demo.md: Pull hint not seen on a title drag (0 days)
  2026-11-20-p2r3co-demo.md: "A cube of desktops" row and Super+C show no cube; Super+C again does nothing (0 days)
  2026-11-20-p2r3co-demo.md: Escape in the key sheet does not close it; Close button does nothing; a Start menu shows (0 days)
  2026-11-20-p2r3co-demo.md: Super+Alt+1 "Give window" has no visible effect (0 days)
  2026-11-20-p2r3co-demo.md: The lock leaves a dark rectangle at the top left (0 days)
  2026-11-20-p2r3co-explorer.md: The row "A cube of desktops" steps into face 1; the cube shows only after Alt+Escape (0 days)
  2026-11-20-p2r3co-explorer.md: The top bar is gone once a window is open (0 days)
  2026-11-20-p2r3co-explorer.md: The key sheet has no close button in the desktop view (0 days)
  2026-11-20-p2r3co-max.md: A window as a pane: scroll and orbit give no clear view of it (0 days)
  2026-11-20-p2r3co-max.md: Helper answers "what time is it?" with "Hello" (0 days)
```
Files: feedback/2026-11-20-p2r3co-{explorer,max,demo}.md; captures media/2026-11-20/persona-r3-{explorer,max,demo}/. No ledger edits.

### Stop rule
Construct does not meet round 3 (p2-plan.md: no new check-backed BLOCK or MAJOR): C2 is red once more (demo, end state) and no persona saw a pull; Max's I3 on a pane dragged out of the picture is a new check-backed lead (MAJOR candidate, a lost-focus window the mouse cannot bring back). co-25 holds and C1, T1 to T5, T4:lock pass. Needed: a persona-side way to see the hint (a held drag), and the checker reading the xp child.
