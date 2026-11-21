# p2-r4-tiling (RC 2 persona round 4, tiling with the fixed harness)

## r4

Lane: cloud, branch claude/p2-r4-tiling at 1c9e64f7 (train 103). Binaries of the round: sha 1c9e64f70d716bbdddee51662f48c4fa32ec3cdf, snapshot /tmp/persona-round/bin (round.sh snapshot once). Step 0: unshallow, cargo check and build green. fuzzel 1.11.1 (ti-1 not in play). Sessions 1280x800: Kemal and beginner tiling (2 slots), ricer tiling+xp after; PERSONA_CHECK=1; helper transcript seeded (user + agent entry) before each persona. Haiku personas via Agent with act.py only, caps 45 actions / 20 looks (own actions: Kemal 25/12 looks, beginner 20/10, ricer 35/18). One `begin` before each goal (goals handed over one at a time by SendMessage to the same persona, no coaching). Persona logs hold persona actions only (persona-actions.jsonl); scripted steps are in scripted-actions.jsonl (Kemal T5: 4 actions; ricer: 6 actions, a give of its xp-owned terminal to tiling, then Super+1, Super+4). Slip: the ricer persona ran one Bash grep outside act.py on the host (goal 5), no effect on the session, and left a stray "clear" typed on the terminal line. Captures: media/2026-11-21/persona-tiling-r4-{kemal,beginner,ricer}/.
Harness note found: `check.py end` hides an invariant of a look when known-red.toml has any row for that (look, invariant) (check.py:220), so all 661 I6 rows of the ricer run and 106 of the beginner run (tiling I6 has many rows) never print; the breach files hold them.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal | T1 Super+Return | pass | 0 | yes |
| Kemal | L1 two terminals, focus, close one | pass | I2 x13: "tb-s2 and tb-s3 are 10 apart, no whole number of 4" (new vs r3) | yes (Super+Return, Super+j, exit; Super+Right = workspace, Alt+Tab nothing) |
| Kemal | T5 (scripted, probe window) | pass | I2 x4 (tb-s3/tb-s4) | n/a |
| Kemal | T3 Super+a | FAIL: helper never worked | 0 | no: panel opened, persona never typed (focus into the input unverified) |
| beginner | T1 Apps click | pass | 0 | yes |
| beginner | T2 Keys button | pass | 0 | yes (Esc closes) |
| beginner | T4 power cancel | pass | 0 | yes (Esc) |
| beginner | T6 unknown program | pass | 0 | yes ("Nothing found for frobnicate" only after Return) |
| ricer | T1 (Super+Return) | FAIL: new window is xp-owned (s1), not tiling | 0 | yes (a floating xp foot window) |
| ricer | L3 workspace and back | pass on the second go; first go FAIL (holders [xp], window xp-owned, tags do not hide it) | 0 | no, then yes (after the scripted give to tiling) |
| ricer | T7 tiling -> xp | pass, vacuous (no handover happened) | 0 | no (three tries: Tile button, Super+Shift+Space; the give key was only seen in the key list afterwards) |
| ricer | T2 key list | pass | 0 | yes |
| ricer | L2 gap and reload | "pass", vacuous (nothing changed) | 0 | no (no settings door; host grep slip) |

### Harness check (p2-harness-l)
| Check | Result | Capture |
|---|---|---|
| L1 against the right baseline | pass: `begin` before the goal, no "gone" text; Kemal closed one of two windows | kemal shots 005-008, check line "task L1 pass" |
| L2 | harness red remains, not a pass: the session has a non-empty config now, so it prints "pass", but the contract is only Expect(focus=look) (check.py:643) and the persona changed no gap. Named as not measured | ricer shots 019-021 |
| L3 | pass on the second go (own begin, terminal tiling-owned); the first go was red because the terminal was xp-owned (T1 red), a product-or-setup question, not the old holder bug | ricer shots 003-006 (red), 010-012 (pass) |
| T7 | pass but vacuous: no handover was done, so no baseline question arose; not reached by the persona | ricer shots 013-016 |
| T5 / I2 on a two-tile layout | I2 appears (new) | kemal evidence/001-key.png |

### Fix check
| Row | Reached | Holds | Capture |
|---|---|---|---|
| ti-1 Apps click (fuzzel 1.11.1) | beginner | pass (Foot clicked once, terminal opened) | beginner shots 002-003 |
| ti-2 Nothing found row, sorted, Screenshot not first | beginner | pass (list starts with Dark look; row shown after Return; xr-2 word-while-typing MINOR remains) | beginner shots 008-009 |
| ti-3 Keys click | beginner, ricer | pass | beginner shot 004, ricer 017 |
| ti-4/22 notice, ti-5 toast, ti-7 fullscreen bar, ti-27 key list after power cancel, I5 key list over a notice | not reached | - | - |
| ti-6/19/23 tooltips on Apps, Keys, power | beginner, ricer | pass for Apps, Keys, power (none in beginner shots 003-007); NEW: stale tooltip after a click on Tile | ricer shot 014 |
| ti-24 power icon, one helper word in bar | beginner, Kemal | pass (power icon, "atlas · idle") | beginner shot 005 |
| ti-29 launcher order | Kemal, beginner | pass: "Chat with a helper" no longer first (Dark look first); helper row absent from launcher | beginner shot 002 |
| ti-29 helper beside tiles | Kemal (one terminal) | pass: panel on the right, left tile shrinks, not hidden | kemal shot 012 |
| I3 one focus holder (r2) | ricer | pass: 0 breaches | ricer end L3 |
| T7 given window visible | not reached | - | - |
| ti-12 Super+Right, ti-28 Super+q | Kemal (ti-12) | open, duplicate (known); ti-28 not tried | kemal shots 005-006 |
| ti-30 key list 13 px / contrast | beginner, ricer | still red (I6, 13 px, 6.51:1), open, seen in breaches.jsonl | beginner breaches.jsonl |

### New and repeated (feedback/2026-11-21-p2r4-{kemal,beginner,ricer}.md)
- NEW issue: I2 gap 10 px between tiles (Kemal), check I2.
- NEW issue: Super+Return in tiling+xp opens an xp-owned window (T1, L3 red); verify whether setup or product.
- NEW issue: Kemal T3 red; chat input focus after Super+a unverified (one capture).
- NEW ux: stale Tile tooltip; Dark look as first launcher row. Repeats: Super+Right (ti-12), no config door, power menu on Lock, Nothing found after Return.

### triage.py (the three r4 files; older rows left out)
```
== tiling / I2 (1)
  2026-11-21-p2r4-kemal.md: Two tiles are 10 px apart, not a whole number of 4 (0 days)
== tiling / contract T1 (2)
  2026-11-21-p2r4-ricer.md: Super+Return in a tiling+xp session opens an xp-owned floating window (0 days)
== tiling / contract T3 (1)
  2026-11-21-p2r4-kemal.md: Super+a opens the helper panel but the persona could not ask (0 days)
== tiling / contract T7 (3)
  2026-11-21-p2r4-ricer.md: No way to float a window or hand it to the XP look is visible (0 days)
== tiling / none (47)
  2026-11-21-p2r4-beginner.md: The "Nothing found" word shows only after Return, then the full list returns (0 days)
  2026-11-21-p2r4-beginner.md: The power menu opens with Lock selected (0 days)
  2026-11-21-p2r4-beginner.md: Small text; "Foot", "Tile" and the tag numbers are unexplained (0 days)
  2026-11-21-p2r4-kemal.md: Super+Right switches workspace instead of moving focus; Alt+Tab does nothing (0 days)
  2026-11-21-p2r4-kemal.md: The launcher has no helper row and the first row "Dark look" is pre-selected (0 days)
  2026-11-21-p2r4-kemal.md: "Listening" chip and the top-bar text are small and unexplained (0 days)
  2026-11-21-p2r4-ricer.md: Tile click leaves the tooltip "Next layout (Super+Shift+Space)" over the window (0 days)
  2026-11-21-p2r4-ricer.md: No settings or config door for the gap (0 days)
== xp / I6 (1)
  2026-11-21-p2r4-ricer.md: xp desktop text under the guide: 11 px labels, icon labels 1.00:1, pager 3.2 to 3.7:1 (0 days)
```
(`triage.py --format`: FORMAT OK.)

### Stop rule (p2-plan.md)
No: tiling does not meet round 4. L1 and L3 pass, L2 is not measured (the harness contract is vacuous), but breaches are not 0: Kemal I2 x13 (new, backed by a check), plus red contracts T1 and L3 first go (ricer, xp-owned window) and T3 (Kemal, unverified). T7 was not reached. Harness reds named: L2 (vacuous), T7 (vacuous), the (look, invariant) known-red filter hides xp I6. No ledger edits.
