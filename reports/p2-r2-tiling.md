# p2-r2-tiling (RC 2 persona round 2, P2 item 1)

## r2

Lane: cloud, branch claude/p2-r2-tiling at ce2c436b. Binaries of the round: sha ce2c436b6fad932fe1614af273dfdbccfafe7b81, snapshot /tmp/persona-round/bin. Step 0: unshallow, cargo check and build green. fuzzel 1.11.1 (so ti-1 is not in play). Sessions tiling 1280x800 (Kemal, beginner at once; ricer after as tiling+xp for T7), PERSONA_CHECK=1, helper transcript seeded (user + agent entry), haiku personas, caps 45 actions / 20 looks. Driver slips: one `look` of mine on Kemal's session before the brief (action 1 of its log); scripted T5 and toast steps follow Kemal's persona actions in the same log (lines 29-37); ricer restarted once as tiling+xp before its persona started. Harness limits as in r1: L1 compares with `start` (windows of the run never "gone"), L2 vacuous (empty config folder, no L2 pass is a real one), T7 compares with `begin`.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal (29 actions) | T1 Super+Space | pass | 0 | yes |
| Kemal | L1 two terminals, focus, close | harness red (gone), as r1 | 0 | yes after Super+Tab; close only by `exit` (Super+q, Super+Shift+q nothing) |
| Kemal | T5 (scripted, probe window) | pass | 0 | n/a |
| Kemal | T3 Super+a | pass | 0 | yes (answer "Hello", fixed) |
| beginner (20 actions, 10 looks) | T1 by Apps click | pass | 0 | yes (Foot opened; r1 FAIL gone, fuzzel 1.11.1) |
| beginner | T2 Keys button | pass | 0 | yes |
| beginner | T4 power cancel | pass | 0 | yes (Esc) |
| beginner | T6 unknown program | pass | 0 | yes by eye ("Nothing found for ...") |
| ricer (38 actions, 16 looks) | L2 gap and reload | vacuous pass | 0 | no (no editor or config door) |
| ricer | L3 workspace and back | pass | I3 and I5 breaches during the run | yes |
| ricer | T2 key list | pass | same | yes |
| ricer | T7 tiling -> xp | FAIL: new [s1, s2] (begin before the windows; no loss by I1) | I3 x4, I5 x2 (3 distinct) | no: window vanished from view, not found |

### Fix check
| Row | Reached | Holds | Capture |
|---|---|---|---|
| ti-1 Apps click row (fuzzel) | beginner | pass on fuzzel 1.11.1 (Foot clicked once, terminal opened) | beginner shots 002-003 |
| ti-2 Nothing found row | beginner | pass ("Nothing found for frobnicate", Screenshot not first); the typed text clears after Return (ux) | beginner shots 007-008 |
| ti-3 Keys click | beginner | pass | beginner shot 004 |
| ti-5 toast with chat open | scripted act.py step (Super+a, chora-notify started in the session bus, notify-send) | pass: the card shows at the bottom of the left tile, beside the chat (x 868+), not under it. Caveat: the card wears the xp balloon skin (chora-notify --assets of xp) | kemal shots 017, 018 |
| ti-6/19 tooltips | beginner clicked Apps, Keys, power | pass: no tooltip in shots 002-010 | beginner shots 002, 004, 006 |
| ti-7 fullscreen bar | not reached | - | - |
| ti-27 key list after power cancel | not reached (the persona closed the key list with Esc before the power menu) | - | - |
| ti-28 Super+q | Kemal | open, duplicate (known) | kemal shots 009-011 |
| ti-12 Super+Right | Kemal | open, duplicate (known) | kemal shots 006-009 |
| ti-29 launcher order, chat over tile | Kemal, beginner | fail: "Chat with a helper" still first and pre-selected; the chat panel still covers the right tile | beginner shot 002, kemal 017 |
| ti-4/22 notice | not reached | - | - |
| ricer r1 yellow square | beginner | seen again under Apps with the launcher open (hit +1, unfiled) | beginner shot 002 |

### New and repeated (see feedback/2026-11-20-p2r2-{kemal,beginner,ricer}.md; captures media/2026-11-20/persona-tiling-r2-*)
- ricer, T7 look hand-over: I3 (two looks claim the focus; the core focuses xp-icons while tiling says it holds it) and a handed-over terminal that is on no visible screen (state.get keeps it shown under the xp desktop). The only high rows of the round.
- I5: the key list overlaps the notice "Run could not start" (the notice is a wayland lock file error from the nested session after the xp Start menu Notepad click; env part of it, the overlap is real).
- ux: Foot has no word for terminal, unlabelled look numbers in "Give window to a look", Lock pre-selected in power, key list words, no config door.

### triage.py (tiling section, the three r2 files; r1 and older rows left out)
```
== tiling / I3 (1)
  2026-11-20-p2r2-ricer.md: Two looks both claim the focus after a window is given (0 days)
== tiling / I5 (1)
  2026-11-20-p2r2-ricer.md: "Give window to a look" overlaps the "Run could not start" notice (0 days)
== tiling / contract T1 (1)
== tiling / contract T4:power (1)
== tiling / contract T6 (1)
== tiling / contract T7 (1)
  2026-11-20-p2r2-ricer.md: Handed-over terminal is on no visible screen (0 days)
== tiling / none (24)
  2026-11-20-p2r2-beginner.md: ux: the program is called "Foot", no word says terminal (0 days)
  2026-11-20-p2r2-beginner.md: ux: Keys list uses words a newcomer does not know (0 days)
  2026-11-20-p2r2-beginner.md: ux: after Return on an unknown program the typed text disappears (0 days)
  2026-11-20-p2r2-beginner.md: ux: a yellow square sits under the Apps button while the launcher is open (0 days)
  2026-11-20-p2r2-beginner.md: ux: the power menu opens with Lock selected (0 days)
  2026-11-20-p2r2-beginner.md: ux: terminal text is small and a locale warning shows (0 days)
  2026-11-20-p2r2-kemal.md: Super+Right switches workspace instead of moving focus (0 days)
  2026-11-20-p2r2-kemal.md: Super+q does nothing; Super+Shift+q neither (0 days)
  2026-11-20-p2r2-kemal.md: The helper panel still covers the right-hand tile (0 days)
  2026-11-20-p2r2-kemal.md: Closing a window by keyboard has no visible way (0 days)
  2026-11-20-p2r2-ricer.md: No way to edit the config from the desktop (0 days)
  2026-11-20-p2r2-ricer.md: The Start menu of the second look shows up by itself and stays on workspace 2 (0 days)
  2026-11-20-p2r2-ricer.md: "Give window to a look" numbers are not labelled (0 days)
```

### Stop rule (p2-plan.md)
No: tiling does not meet round 2. New backed-by-check rows: I3 and T7 hand-over (high, two rows) and I5 (medium); none duplicates an open row (ti-25's "stale buffers" is not the same). Round 3 needed for tiling after a fix of the tiling+xp hand-over. Tasks of the p2-plan tiling rows pass (a)+(b)+(c) on this sha for T1, T2, T3, T4:power, T5, T6, L3; L1 and L2 are harness-limited, T7 fails. No ledger edits.
