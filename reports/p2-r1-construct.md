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

## r1b
Lane: cloud, branch claude/p2-harness (b97369d1 + 6 commits, tests only; the product binaries are those of b97369d1). Binaries of the round: sha 141dfd97, snapshot /tmp/persona-round/bin. Setup as r1 (2 slots, explorer and Max together, then the demo watcher; helper transcript seeded; haiku personas with act.py only; caps 45 actions, 20 looks; Max with `round.sh probe max`, `check.py begin` before the tasks). Persona limits held (15, 41, 30 actions; 8, 19, 16 looks; no other tool called).

### Harness items (red commit, then fix; all pushed on claude/p2-harness)
1. "XP did not start within 10 s": **harness**. The cube's xp is started by the construct as `chora-xp --output N` from the PATH; chora-xp looks for its theme in `CHORA_XP_THEME`, else `/usr/share/chora/flavors/xp/theme`, else the source tree of its executable (`target/<profile>/`). The round snapshot (`/tmp/persona-round/bin`) has no source tree, so the theme load failed (`chora-xp: theme: /usr/share/chora/flavors/xp/theme/expose/decoration.svg: No such file ... I come back`, repeated in the cube log in the runtime dir) and xp never connected. The gates run from target/debug and find the tree. Fix: session.sh exports `CHORA_XP_THEME=$repo/flavors/xp/theme`. Check: tests/persona/xp-child.sh (red cb6e3e7a, fix e330fba1). Repro of the old state: `CHORA_BIN=<copy of target/debug/chora*> tests/persona/session.sh construct`, then `choractl call construct cube.open '{}'`; the cube log says the line above. No product change; one note for the ledger: a chora-xp that starts from a copy of the binaries without a theme argument says only "I come back" and loops, and the construct reports "did not connect within 10 s" without the reason (the reason is in the log file only).
2. C2: `check.py begin` takes the snapshot a task is held against (`end` reads the newest begin, else `start`). C2 now fails only for a window that was open at the begin, or was seen while the task ran, and is gone at the end; a window opened during the task is no breach. Tests in tests/test_acts.py (red f9a89088, fix f92342e5).
3. T5: `round.sh probe NAME` starts `chora-probe window --report` with `WAYLAND_DISPLAY=<socket name>` and `XDG_RUNTIME_DIR`, fails when it does not stay up and prints `PROBE_REPORT=<file>`. `check.py end --probe` is red when the report is missing or no `chora-probe` window is in the scene (no vacuous pass). tests/persona/probe-check.sh and tests/test_acts.py (red def798de, fix 141dfd97). Found while running: `CODES` had no evdev code for d, q and Tab, so Super+d, Super+q and Super+Tab were never read (T5 vacuous again); fixed (red 3d1f4663, fix 5dbd0ccf). `round.sh probe` also printed an mv error; fixed in the same commit.
Gate: merge of origin/claude/autonomous-dev-session-3w-i33pg7 was already up to date; `merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,cube,pull,out`: all 8 cells PASS (run on 141dfd97; a second run of pre-commit, ci-check, build, test, clippy on 5dbd0ccf: all PASS).

### Did I7 on every output go away with xp running?
Yes. r1: 100 and 136 lines (every action). r1b: explorer 4 lines (one per output, at the first click only), Max 0, demo 4 (first click only). Probable cause of the 4: faces not yet drawn at the first quiet snapshot (to verify before the ledger). The ledger rows from p2r1 for I7 and "Every XP face says did not start" were caused by the harness and are not product issues.

### Explorer (15 actions, 8 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 cube, step in, out | pass | I7 x4 (first click) | yes: the row goes straight into face 1; the cube only after Alt+Escape |
| T1 start a program | FAIL "expected open [], got [construct-help]": the key list of goal 3 was still open when the three tasks were checked at once (harness: begin per task would clear it) | I7 | yes by Super+T |
| T2 key list | same | I7 | yes, only from the Keys item of the top bar after stepping back (Super+F1 in a face showed nothing; the bar is hidden in a face) |

### Max (41 actions, 19 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C2 pull out and back | pass, but nothing was pulled: the contract reads no loss, not a pull | 0 | no (three drags of the face did nothing) |
| C3 window as a pane | pass | 0 | no (Return, Tab, right-click showed nothing) |
| T5 no stray keys | pass as run; **FAIL with the corrected codes**: the probe report holds key 125 and then 32, 16, 15 | 0 | Super+D/Q/Tab reached the probe |
| T3 helper | pass | 0 | yes ("Hello", working then idle) |
The route to a terminal by Programs > Foot gave the helper chat again (typing Foot in the filter and Return worked).

### Demo watcher (30 actions, 16 looks, 1 failed action: key name Esc)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| C1 | pass | I7 x4 (first click) | no (the row gave a plain XP desktop; Super+C did nothing in it) |
| C2 | pass, nothing pulled | I1 x21, I3 x21 ("s1 is owned by xp, which is no look") | no |
| T4 lock | pass (`swaylock -f` x2 in calls.log) | same | not seen (the stand-in draws nothing) |
I1 and I3: probably the checker, which reads only the state of the looks of the session (construct); a window of the cube's xp child has an owner that is no look there. Not confirmed; no ledger row until the checker reads the xp child.

### For the PM (not fixed here)
1. T5 on construct: is Super+D, Super+Q, Super+Tab to the focused client a defect or by design (the construct binds Super+T, C, Space, F1)? The contract names no chords the construct owns; with `--test-injected-press-is-real` set the injected keys count as real. Decide before the ledger.
2. C2 (and T7 on construct) pass without a pull: add a seen-pull to the contract (a pane that was a face window) before R2.
3. `check.py begin` per task is now available; a driver that checks tasks one at a time (begin, task, end) avoids the T1/T2 "key list left open" reds.
4. I1/I3 on a window owned by the cube's xp: make the checker read the xp child's state (Session looks) before the next round.
5. Persona findings of r1b: feedback/2026-11-20-p2r1b-{explorer,max,demo}.md; captures and logs media/2026-11-20/persona-r1b-{explorer,max,demo}/. No ledger edits.

## Review r1b (PM, 2026-11-20 02:55 UTC; PM not the builder; harness diff of claude/p2-harness 5dbd0ccf)
Base 79c8d2b5...5dbd0ccf, 7 files under tests/ (+275 -15); 4 red commits each before its fix. check.py begin per task, C2 = no window lost (new windows fine), T5 probe via round.sh with a dead-probe red, letter and Tab codes; session.sh sets CHORA_XP_THEME for snapshot binaries.
Findings: ph-1 NIT round.sh probe waits a fixed 3 s (wait for the probe window in the scene). ph-2 NIT the I1/I3 x21 on the cube's xp window (checker reads only the construct state) stays a harness lead for the next round.
Verdict (computed): no BLOCK or MAJOR: MERGE-OK; train 83 (harness first, after train 82).
Decision (lane question 1): Super+D/Q/Tab into a client in construct is a defect (co-25 MAJOR; precedent docs/configuration.md:324-330). Super+F1 in a face: co-26 MAJOR (verify). Construct gets the co-p2-fix lane; construct stop rule reopened.
