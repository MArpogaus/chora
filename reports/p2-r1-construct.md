# p2-r1-construct (RC 2, P2 item 1)

Lane: cloud, branch claude/p2-construct at b97369d1 (no code changes). Binaries of the round: sha b97369d1cd9a3c9837ceaf756cc115c0dc5c113b, snapshot /tmp/persona-round/bin. Step 0: unshallow, cargo check and build green.

## r1
Setup: round.sh snapshot once, sessions construct 1280x800, PERSONA_CHECK=1, 2 slots (explorer and Max together, the demo watcher after). Helper transcript seeded in voice/entries.json (user + agent entry) after each start. Personas: haiku Agents with act.py only, caps 45 actions, 20 looks. Max's session also got a chora-probe window for T5.
Driver slips: (1) the T5 probe died with "Broken pipe" (wrong WAYLAND_DISPLAY: I gave the socket path), so T5 is vacuously green and not measured. (2) Persona demo called read_documentation once (outside its tool list; no effect). Persona limits otherwise held (26, 34, 35 actions; 14, 16, 20 looks).
Harness reds (not issues): C2 on Max and demo ("windows lost [], new ['s1']": the window was opened during the run, the contract compares to the start snapshot); T5 probe (above). Possible harness, to confirm before the ledger: chora-xp did not connect in any of the three sessions ("XP did not start ... within 10 s"), which empties the faces (I7 on 4 outputs after every action, 100 and 136 lines) and may explain the mouse path to the cube and Programs > Foot.

### Explorer (26 actions, 14 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 cube, step in, out | pass | I7 x100 (all four outputs, every action) | yes, after two failed tries (row, Super+C), then Alt+Escape |
| T1 start a program | FAIL: expected open [], got [construct-help] (Help left open; also the mouse path Programs > Foot gave the chat) | I7 | yes by Super+T |
| T2 key list | FAIL: same open [construct-help] | I7 | yes (Super+F1) |

### Max (34 actions, 16 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C2 pull out and back | harness red (new s1) | 0 | yes (Alt+Escape pane, Return fly in) |
| C3 window as a pane | pass | 0 | yes, but typing in a pane goes to walk keys |
| T5 no stray keys | pass, vacuous (probe died) | 0 | Super+D/Q/Tab reached the terminal as text |
| T3 helper | pass | 0 | yes ("Hello") |

### Demo watcher (35 actions, 20 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | I7 x136 | partly (faces "did not start") |
| C2 | harness red (new s1); no pull seen | I7 | no |
| T4 lock | pass (swaylock -f logged) | I7 | not verified (no look after) |

Real candidates (to verify): I7 on every output (construct-wide, maybe the xp face), the cube row/Super+C start failure of XP, Programs > Foot gives the helper chat, Esc steps out of the pane while closing the key list, pane input goes to walk keys, Super chords reach a client (needs a live probe in R2).

### triage.py (construct and format)
```
== construct / I7 (2)
  2026-11-20-p2r1-demo.md: I7 on every output through the whole run (0 days)
  2026-11-20-p2r1-explorer.md: I7 on every output through the whole run (0 days)
== construct / contract C1 (2)
  2026-11-20-p2r1-demo.md: Every XP face says "did not start" while the notice says "The other faces still work" (0 days)
  2026-11-20-p2r1-explorer.md: The cube row leads to a "Starting XP" spinner, then a failure notice; the cube only came by Alt+Escape (0 days)
== construct / contract C2 (1)
  2026-11-20-p2r1-demo.md: A window cannot be pulled out of its desktop by the gesture a demo viewer tries (0 days)
== construct / contract T1 (1)
  2026-11-20-p2r1-explorer.md: Programs > Foot opens the helper chat instead of a terminal (0 days)
== construct / contract T4:lock (1)
  2026-11-20-p2r1-demo.md: Top-bar Lock click and Super+Shift+Escape show no lock to the viewer (0 days)
== construct / none (6)
  2026-11-20-p2r1-demo.md: Esc closes the key list and steps out of the pane too (0 days)
  2026-11-20-p2r1-explorer.md: Step into a face needs a click and Return; scroll does nothing (0 days)
  2026-11-20-p2r1-explorer.md: The chat panel stays open and covers the key list (0 days)
  2026-11-20-p2r1-max.md: Typed text in a pane goes to the walk keys, not the window (0 days)
  2026-11-20-p2r1-max.md: Super+D, Super+Q and Super+Tab reach a client as text (0 days)
  2026-11-20-p2r1-max.md: The helper panel opens only from the top bar (0 days)
OVERDUE
```
Files: feedback/2026-11-20-p2r1-{explorer,max,demo}.md; captures and logs media/2026-11-20/persona-{explorer,max,demo}/. No ledger edits.
