
## r4
Lane: cloud, branch claude/p2-r4-co at a5b77488, run only (no code changes). Step 0: unshallow, `cargo check --workspace` and `cargo build --workspace` green; `round.sh --help` ok; smoke `round.sh begin smoke` on construct took its begin (r2 works; `check.py end smoke` says "no contract for smoke", as it has none). Binaries of the round: sha a5b77488306a95dee0f830b5f52953cd324aeb00, snapshot /tmp/persona-round/bin.
Setup: round.sh snapshot once, construct 1280x800, 2 slots (explorer and Max together, demo after), PERSONA_CHECK=1, helper transcript seeded (one agent entry), `round.sh probe max`, one `begin` per goal and `check.py end` after it (one haiku Agent per goal, same session). Limits: actions 17, 41, 24 (cap 45); looks 10, **23** (cap 20: breached by the driver, the per-goal caps were summed, not the looks), 13. No retries past 3.

### Explorer (17 actions, 10 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | I7 x4 (first click, known) | yes (row, Alt+Escape) |
| T1 | pass | 0 | yes (Programs, Foot) |
| T2 | **FAIL** key list never open | 0 | no (click, F1, ?: nothing; Super+F1 not tried) |

### Max (41 actions, 23 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C2 drag the probe | pass | 0 (r3: I3 x23) | no (drag looks, probe leaves the picture) |
| C2 pull | pass (no pull) | I7 x4 (first key, known) | no (16 actions to find the cube) |
| C3 two terminals, fly, Tab | pass | 0 | no (Foot fills the view, no second window) |
| T5 | pass, vacuous | 0 | no (probe not found, no key pressed) |
| T3 | pass | 0 | yes ("Hello", stand-in) |

### Demo watcher (24 actions, 13 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | I7 x4 (first click, known) | yes |
| C2 | pass (no pull) | 0 (r3: I1/I3 x48) | no (no window on any face) |
| T4 lock | pass (calls.log: swaylock -f) | 0 | no by its eyes (click highlighted only) |

### Fix check against r3
| Row | Reached | Holds | Capture |
|---|---|---|---|
| C2 window in the xp face after a pull (co-pane r1 item 3) | no pull by any persona; demo spent 24 actions in an xp face | **partial**: 0 I1 and 0 I3 on the xp child in 24 actions (r3: x48), and C2 gave no "windows lost"; the pull itself not reached | persona-r4-demo/breaches.jsonl |
| Max I3, pane dragged out of the picture | yes (3 drags on the probe, the scene panned, the pane left the picture) | **pass**: 0 breaches (r3 I3 x23); ux: the persona did not get the pane back (reverse drag restores the previous view, not the first) | persona-r4-max shots 001-004 |
| co-33 pull hint on a held drag | no: act.py drags are one action; no title drag | not reached | - |
| co-35 Tab list after the flight | no: Foot filled the view, no second window, no flight out | not reached | persona-r4-max shots 013-017 |

### triage.py (the three files, run in the log root: `python3 triage.py feedback/2026-11-21-p2r4co-*.md`; FORMAT OK; construct lines of r4)
```
  2026-11-21-p2r4co-max.md: Probe dragged in the walk room: I3 holds, the window is still not back in the picture (0 days)
  2026-11-21-p2r4co-demo.md: I7 at the first click (0 days)
  2026-11-21-p2r4co-explorer.md: I7 at the first click on all four outputs (0 days)
  2026-11-21-p2r4co-max.md: I7 at the first key (0 days)
  2026-11-21-p2r4co-demo.md: No window on any face of the cube: C2 cannot start (0 days)
== construct / contract C3 (1)
  2026-11-21-p2r4co-max.md: Foot fills the view; no way out or to a second window; Super+Return and Escape do nothing (0 days)
== construct / contract T2 (2)
  2026-11-21-p2r4co-explorer.md: T2 red: the key list construct-help was never open (0 days)
  2026-11-21-p2r4co-explorer.md: No visible door to the key list in a face or stood back (0 days)
  2026-11-21-p2r4co-demo.md: Lock: click only highlights, Return steps in, Super+L nothing visible (0 days)
  2026-11-21-p2r4co-max.md: T5 not measured: the probe is not found (0 days)
  2026-11-21-p2r4co-demo.md: Hint says "Alt Esc"; the key tool wants "Escape" (0 days)
  2026-11-21-p2r4co-explorer.md: ux: Foot text is very small at the capture scale (0 days)
  2026-11-21-p2r4co-max.md: Super+C does nothing while the key sheet is open, then goes into the XP pane (0 days)
  2026-11-21-p2r4co-max.md: Helper answers "what time is it?" with "Hello" (0 days)
```
Files: feedback/2026-11-21-p2r4co-{explorer,max,demo}.md; captures media/2026-11-21/persona-r4-{explorer,max,demo}/. No ledger edits.

### Stop rule
Construct does not meet round 4 (p2-plan.md: no new check-backed BLOCK or MAJOR), on the strict reading: T2 is red for the explorer (check-backed, shared-bar task; r3 passed it by Super+F1, so the lead is a door that a mouse user does not find, a MAJOR candidate to verify). Better than r3: Max's I3 and the demo's I1/I3 (checker xp child) are gone, no C2 "windows lost", T1, T3, T4:lock, C1, C3 pass. Not reached for lack of a path a persona finds: the pull (C2, co-33 hint), co-35 (Tab after the flight), T5 on a visible probe (the probe was behind the full-screen Foot).
