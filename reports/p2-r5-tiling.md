# p2-r5-tiling (RC 2 persona round 5, tiling)

## r5

Lane: cloud, branch claude/p2-r5-tiling at 230ea0e5. Binaries of the round: sha 230ea0e506a58eecee95074932a55d1481c9d991 (round.sh snapshot, /tmp/persona-round/bin). Step 0: unshallow, cargo check and build green; smoke (`round.sh begin smoke`, `check.py end smoke`) ran ("no contract for smoke", expected). fuzzel 1.9.2 (below 1.11, ti-1 in play). Sessions 1280x800, 2 slots (Kemal and beginner tiling, ricer tiling+xp), PERSONA_CHECK=1, helper transcript seeded (user "Hi" + agent "Hello") before each persona, haiku personas via Agent, act.py only, one `begin` before each goal, goals one at a time by SendMessage, no coaching. Caps (sum <= 45 each): Kemal T1 6, L1 10, T3 9 (25); beginner T1 5, T2 4, T4 5, T6 5 (19 used of 20); ricer T1 6, L3 10, T7 10, T2 5, L2 4 (35). Used: Kemal 19 own + 5 scripted (T5), beginner 19, ricer 14.
Captures: media/2026-11-23/persona-tiling-r5-{kemal,beginner}; ricer: the session's capture folder was NOT written (PERSONA_KEEP of the tiling+xp session missing after `round.sh stop`), only the check outputs survive (media/2026-11-23/persona-tiling-r5-ricer/, persona-tiling-r5-checks/). Not retaken (budget). The beginner T2 contract line was lost to my own output filter (44 I6 rows printed, no non-I6).
Slip: on L2 the ricer tried to type a grep of ~/.config into the terminal; the sandbox classifier blocked it before it ran (not in the action log, no effect). Other personas: act.py only.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal | T1 Super+Return | pass | 0 | yes |
| Kemal | L1 two terminals, focus, close one | FAIL (expected 1 window closed) | 0 | no: no focus key found (Super+Right = workspace, Super+l resizes, Alt+Tab nothing; Super+j not tried) |
| Kemal | T5 (scripted, probe, Super+x/q/w) | pass | 0 | n/a |
| Kemal | T3 Super+a, ask | pass | 0 | yes (typed straight into the input, answer Hello = fake agent) |
| beginner | T1 Apps click, Foot | FAIL (no new window) | 0 | no (fuzzel 1.9.2: click does nothing, ti-1 known) |
| beginner | T2 Keys button | not kept (see above) | I6 only | yes (list opened; Escape closed the Apps menu first, list stayed) |
| beginner | T4 power cancel | pass | 0 | yes (cancel not confirmed by a look) |
| beginner | T6 frobnicate | pass | 0 | no: launcher shows "Nothing found" only after Return |
| ricer | T1 Super+Return | pass | I6 xp only | yes |
| ricer | L3 workspace and back | pass | I6 xp only | yes |
| ricer | T7 tiling -> xp | pass, vacuous (persona pressed Super+Shift+2, a tag move; no handover) | I6 only | no (read Super+Alt+1-6 "Give window to a look" in the list afterwards, did not use it) |
| ricer | T2 key list | pass | I6 only | yes |
| ricer | L2 gap and reload | FAIL (value unchanged) | I6 only | no (no settings door; the typed grep was blocked) |

Breaches outside known-red: Kemal 0 (no breaches file), beginner 256 I6 (key list 13 px / 6.50:1 / 1.00:1, ti-30 known), ricer I6 only (xp pager 2.57:1, 3.77:1, known xp I6). No I2.

### Fix check against r4
| Row | Result | Capture |
|---|---|---|
| Kemal I2 (gap 12) | pass: 0 breaches in the whole run, two terminals and three windows (T5) | kemal run, no breaches.jsonl |
| Kemal T3 (types after Super+a) | pass: cursor in the input, "what time is it" sent, helper working -> idle; contract pass | kemal shots 009-010, checks/kemal-T3.txt |
| ricer T1 first go | pass: new window tiling-owned (pointer on tiling's claim by default; D55); T1b not tried | checks/ricer-T1.txt |
| ricer L3 first go | pass | checks/ricer-L3.txt |
| T7 reached | NO: persona made a tag move, not a handover | checks/ricer-T7.txt |
| L2 harness (was vacuous) | no longer vacuous: red when nothing changed ("config value did not change"); persona never changed a gap, so not a pass | checks/ricer-L2.txt |
| T7 harness | still vacuous: "pass" for a tag move inside tiling | checks/ricer-T7.txt |

### triage.py (the three r5 files; older rows left out; FORMAT OK)
```
== tiling / I6: p2r5ti-beginner Key list I6 rows (13 px, 1.00:1); p2r5ti-ricer xp pager contrast
== tiling / contract L1: p2r5ti-kemal No key moves focus between two tiles
== tiling / contract L2: p2r5ti-ricer No settings door for the gap; L2 not reached
== tiling / contract T1: p2r5ti-beginner Click on Foot does not start the program (fuzzel 1.9.2)
== tiling / contract T7: p2r5ti-ricer Give window to a look not used; T7 vacuous
== tiling / none: beginner x3 (Escape one layer, Super+R launcher, power menu Lock selected), kemal x3, ricer x2 (T1 note, capture folder not kept)
```

### Stop rule (p2-plan.md)
No: tiling does not meet round 5. Not all tasks reached: Kemal L1 (no focus key found), beginner T1 (fuzzel 1.9.2 environment, ti-1) and T6, ricer T7 and L2. No new check-backed BLOCK or MAJOR: Kemal I2 and T3 and ricer T1 and L3 now hold; the reds are L1 (persona did not find Super+j), L2 (no config door, known), T1 (ti-1 on fuzzel 1.9.2) and I6 (ti-30). Open for the PM: ricer capture folder missing after `round.sh stop` of a tiling+xp session; T7 harness vacuous. No ledger edits.
