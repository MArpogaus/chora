# p2-r1-xp (RC 2, P2 items 0b and 1)

Lane: cloud, branch claude/pw at 5755b578 (no code changes). Binaries of the round: sha 5755b578887736cbb782e3e35051b39eaa6f3514, snapshot /tmp/persona-round/bin (chora, choractl, chora-xp, chora-assistant, chora-probe and the rest of target/debug). Step 0: unshallow, cargo check and build green.

## Tooling (0b)
TRI was already on the branch (feef974e): feedback/README.md has the `task:` and `check:` lines, triage.py (at the root, not tools/) groups by (flavor, check), flags HIGH, NO CHECK, has `--format`; tests_triage.py: 4 tests OK, `triage.py --format` OK on the three new files. Nothing to add.

## r1
Setup: round.sh snapshot once, sessions xp 1280x800, PERSONA_CHECK=1, 2 slots (Erika and Murat together, the pixel-watcher after). Helper transcript seeded in voice/entries.json (a user and an agent entry) after each start. For T5 a chora-probe window (40,120,200 --report) was started with choractl run, with the WAYLAND_DISPLAY of the headless output. Personas: haiku Agents with act.py only; caps 45 actions, 20 looks. Breaches (breaches.jsonl): 0 in all three runs. One driver slip: a test look on Erika's session before her run; its shot and log line were removed before she started.
Contract caveat: "expected open [], got [xp-dlg-help]" on T2/T4/X2/T1 comes from the persona leaving the Help window open, not a defect.

### Erika (31 actions, 17 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 Notepad from Start | FAIL: no new focused window | 0 | no (balloon, then "No window opened in 30 seconds") |
| X1 find window after minimize | pass (vacuous: no window) | 0 | no (no window to minimize) |
| T4 clock | FAIL: no clock node in the scene | 0 | yes (read 11:52 by eye) |
| T4 power cancel | pass | 0 | yes |
| T3 helper | FAIL: helper never worked (no_helper in all samples), no answer | 0 | no |

### Murat (36 actions, 19 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 | FAIL (no window; open left by Help) | 0 | no |
| T2 Help and Support | FAIL: key-list words missing, open left | 0 | yes |
| T4 volume | FAIL: no volume control by click; no set call | 0 | no |
| T4 screenshot | FAIL: no grim call | 0 | not tried |
| T6 honest failure | pass | 0 | yes ("Nothing found") |

### pixel-watcher (42 actions, 17 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| X2 Start menu, drag window | FAIL: rect unchanged (probe window, no title bar: harness?) | 0 | no |
| T2 key list | FAIL (open left by Help) | 0 | yes via Help; Super+F1 showed nothing |
| T5 no stray keys | pass (probe saw no key; Super+D hid the probe window) | 0 | partial: 8 chords tried |
| T4 lock | FAIL: no loginctl call; "shared script not installed" | 0 | no |

Likely env or harness, to confirm before the ledger: Notepad (no GUI editor installed here; nano only), "shared scripts not installed" (lock, screenshot), probe window not draggable, T3 helper state (fake voice agent). Real candidates: speaker icon opens nothing (T4 volume), Super+D with no way back, key-list words check on xp.

### triage.py over the three files
```
== xp / contract T1 (2)
  2026-11-20-p2r1-erika.md: Notepad from Start never opens a window (0 days)
  2026-11-20-p2r1-murat.md: Notepad never opens (second hit) (0 days)
== xp / contract T2 (1)
  2026-11-20-p2r1-murat.md: Help key list fails the key-sheet words check (0 days)
== xp / contract T3 (1)
  2026-11-20-p2r1-erika.md: Helper chat opens but "hello" gets no answer; state stays no_helper (0 days)
== xp / contract T4:clock (1)
  2026-11-20-p2r1-erika.md: Clock on the taskbar is tiny, and the checker finds no clock node (0 days)
== xp / contract T4:lock (1)
  2026-11-20-p2r1-pixel.md: Lock does nothing: Start > Lock and Super+Shift+Escape (0 days)
== xp / contract T4:volume (1)
  2026-11-20-p2r1-murat.md: Clicking the speaker icon shows no volume control (0 days)
== xp / contract T6 (1)
  2026-11-20-p2r1-murat.md: Run box says Nothing found for an unknown program (0 days)
== xp / contract X2 (1)
  2026-11-20-p2r1-pixel.md: The small window does not follow a drag (0 days)
== xp / none (6)
  2026-11-20-p2r1-erika.md: Power dialog Cancel is small and low in the dialog (0 days)
  2026-11-20-p2r1-erika.md: Starting Notepad balloon comes back after the power dialog is cancelled (0 days)
  2026-11-20-p2r1-murat.md: Help lists key "Vol-" that cannot be pressed, and screenshot not tried (0 days)
  2026-11-20-p2r1-pixel.md: Super+D hides the window and it does not come back (0 days)
  2026-11-20-p2r1-pixel.md: Super+F1 and Super+Alt+1 show nothing (0 days)
  2026-11-20-p2r1-pixel.md: Help lists shortcuts that need scripts or programs that are not there (0 days)
waiting 15, overdue 0, missing ux 0
OK
```

Files: feedback/2026-11-20-p2r1-{erika,murat,pixel}.md; captures and logs media/2026-11-20/persona-{erika,murat,pixel}/. No ledger edits.
