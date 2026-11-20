# p2-r3-tiling (RC 2 persona round 3, tiling after handover-ti)

## r3

Lane: cloud, branch claude/p2-r3-tiling at dcb77733. Binaries of the round: sha dcb777334f96e57ffc31ae1d5985b46dd61b1cf7, snapshot /tmp/persona-round/bin (round.sh snapshot). Step 0: unshallow, cargo check and build green. fuzzel 1.11.1 (ti-1 not in play). Sessions: Kemal and beginner tiling 1280x800 (2 slots), ricer tiling+xp after the beginner slot freed; PERSONA_CHECK=1; helper transcript seeded (user + agent entry) before each persona; haiku personas, caps 45 actions / 20 looks. Persona logs hold persona actions only (Kemal 35, beginner 18, ricer 36); the scripted T5 steps are in media/.../persona-tiling-r3-kemal/scripted-actions.jsonl. Slips: ricer persona ran one Bash command outside act.py (`which nvim vim nano micro`, in its own shell, no effect on the session). Harness limits as r2: L1 compares with `start`, L2 vacuous (empty config), T7 and L3 compare with a `begin` taken before the run (one `end` for several goals). A repeat `end` compares with the end state of the previous `end`, so a second pass of T7 means nothing.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal (35 actions) | T1 Super+Space | pass | 0 | yes |
| Kemal | L1 two terminals, focus, close | harness red (gone), as r1/r2 | 0 | yes by Super+Tab; close only by `exit` (Super+q, Super+Shift+q nothing) |
| Kemal | T5 (scripted, probe window) | pass | 0 | n/a |
| Kemal | T3 Super+a | pass | 0 | no by the persona: "Hello" twice (fake agent's fixed answer, env) |
| beginner (18 actions, 9 looks) | T1 Apps click | pass | 0 | yes |
| beginner | T2 Keys button | pass | 0 | yes |
| beginner | T4 power cancel | pass | 0 | yes (Esc) |
| beginner | T6 unknown program | pass | 0 | yes ("Nothing found for frobnicate" after Return) |
| ricer (36 actions, 16 looks) | L2 gap and reload | harness red (empty config; the focus text) | 0 | no (no config door, no editor door) |
| ricer | L3 workspace and back | harness red (holders [xp] at the end of the run, after the give goal) | 0 | yes (terminal visible again, shot 013) |
| ricer | T2 key list | pass | 0 | yes |
| ricer | T7 tiling -> xp | red by harness begin: windows lost [], new [s1, s2] | 0 | no by the persona (it took the covered window for a lost one) |

### Fix check
| Row | Reached | Holds | Capture |
|---|---|---|---|
| I3 one focus holder (r2 ricer) | ricer | pass: 0 breaches; state after the run: xp holder s1, tiling holder null (one holder). Click-back on tiling's window (rw-1) not tried by the persona | ricer state read, shots 015-016 |
| T7 given window visible (r2 high) | ricer | pass for visibility: s2 given to tiling, shown true, visible true, rect 14,54,1252,702 (r2: on no screen). The window is drawn under the xp window s1 (root order, known MINOR); the persona read the dark area as "vanished". Contract T7 red only by the begin | ricer shots 014-016 |
| I5 key list over a notice | not reached (no notice; the key list was opened and closed at action 21-23) | - | - |
| ti-1 Apps click (fuzzel) | beginner | pass (Foot clicked once, terminal opened) | beginner shots 002-003 |
| ti-2 Nothing found row | beginner | pass (row shown, Screenshot not first); the word comes only after Return (xr-2) | beginner shots 008-009 |
| ti-3 Keys click | beginner | pass | beginner shot 004 |
| ti-4/22 notice | not reached | - | - |
| ti-5 toast with chat open | not reached (no scripted toast step) | - | - |
| ti-6/19 tooltips | beginner clicked Apps, Keys, power | pass: no tooltip in shots 002-006 | beginner shots 002, 004, 006 |
| ti-7 fullscreen bar | not reached | - | - |
| ti-27 key list after power cancel | not reached (the key list was closed with Esc before the power menu) | - | - |
| ti-12 Super+Right | Kemal | open, duplicate (known) | kemal shots 006-007 |
| ti-28 Super+q | Kemal | open, duplicate (known) | kemal shots 011-013 |
| ti-29 launcher order | Kemal, beginner | fail: "Chat with a helper" first and pre-selected | beginner shot 002 |
| ti-29 chat over a tile | not reached (one terminal left at the time) | - | kemal shot 014 |
| r1/r2 "yellow square" | beginner, ricer | explained: the pointer marker of the capture at the last click (shots 004, 014, 016); harness, not a finding | ricer shots 014, 016 |

### New and repeated (feedback/2026-11-20-p2r3-{kemal,beginner,ricer}.md; captures media/2026-11-20/persona-tiling-r3-*)
- ricer T7: the handed-over terminal is on the screen but under the xp window of the same session (stacking by root order, the known MINOR of handover-ti r2). The r2 high rows (I3, window on no screen) did not repeat.
- ricer: the xp Start menu is open after Super+Return with no click (seen in r2 too, check none, no contract for it, so `verify`, not backed). Closes with the give.
- ux repeats: launcher order (ti-29), power menu opens on Lock, Foot has no word terminal, no config door (L2), Listening chip unexplained, helper answers "Hello" (env).

### triage.py (tiling section, the three r3 files; older rows left out)
```
== tiling / contract L1 (1)
  2026-11-20-p2r3-kemal.md: Contract L1 red: expected 1 windows gone, got [] (0 days)
== tiling / contract L3 (1)
  2026-11-20-p2r3-ricer.md: Contracts L2 and L3 red: "expected the focus with tiling, holders [xp]" (0 days)
== tiling / contract T7 (1 of 2)
  2026-11-20-p2r3-ricer.md: Handed-over terminal is covered by the xp window, not lost (0 days)
== tiling / none (15 of 39)
  2026-11-20-p2r3-beginner.md: The "Nothing found" row shows only after Return (0 days)
  2026-11-20-p2r3-beginner.md: The power menu opens with Lock selected (0 days)
  2026-11-20-p2r3-beginner.md: The program is called "Foot"; the Keys list uses words a newcomer does not know (0 days)
  2026-11-20-p2r3-beginner.md: Apps launcher has no visible search hint; "Chat with a helper" first (0 days)
  2026-11-20-p2r3-beginner.md: Text is small; Tile and the tag numbers are unexplained (0 days)
  2026-11-20-p2r3-kemal.md: Super+Right switches workspace instead of moving focus (0 days)
  2026-11-20-p2r3-kemal.md: Super+q does nothing; Super+Shift+q neither (0 days)
  2026-11-20-p2r3-kemal.md: The launcher lists "Chat with a helper" first and pre-selected (0 days)
  2026-11-20-p2r3-kemal.md: Focus moves only with Super+Tab; the focused border is faint (0 days)
  2026-11-20-p2r3-kemal.md: Helper answers "Hello" to every question, status idle (0 days)
  2026-11-20-p2r3-kemal.md: "Listening" chip and the top bar text are small and unexplained (0 days)
  2026-11-20-p2r3-ricer.md: Start menu of xp opens by itself and stays open (0 days)
  2026-11-20-p2r3-ricer.md: Super+Alt+1 turns the whole screen dark; which look got the window is not shown (0 days)
  2026-11-20-p2r3-ricer.md: No way to edit the config from the desktop (0 days)
  2026-11-20-p2r3-ricer.md: xp window frame on a tiling window; two desktop styles at once (0 days)
```
(`triage.py --format`: FORMAT OK. The I3, I5 and T1/T4/T6 groups hold only r2 rows; no r3 row falls in them.)

### Stop rule (p2-plan.md)
Yes, with caveats: tiling meets round 3. No new BLOCK or MAJOR backed by a check: breaches 0 in all three runs; the red contracts are L1, L2, L3 (harness limits of the begin and the empty config) and T7 (begin; no window lost, the window is shown, covered by an xp window: the known MINOR). The Start-menu-by-itself row has `check: none`, so it enters as `verify`. Tasks T1, T2, T4:power, T5, T6 pass (a)+(b)+(c); T3 passes (a)+(b), (c) is no by the fake agent's fixed answer; L1, L2, L3 are harness-limited; T7 (a) is red by the harness begin, (b) and the visibility hold. A cell that takes `begin` after the windows of T7 and L3 would settle both. No ledger edits.
