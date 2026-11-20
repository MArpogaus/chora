# p2-r5-mc

## r5
Lane: cloud, branch claude/p2-r5-mc at e7f2410a (session fee9c695 plus mc-fix 929de144). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green. No product or test code changed.
Binaries of the round: sha e7f2410a6ad5a4785577da34d8b0253e93bf5646, snapshot /tmp/persona-round/bin (round.sh snapshot once). Sessions 1280x800, PERSONA_CHECK=1, cook and sofa together on mc, nurse after on xp+mc. Seeded after each start: helper transcript (user + agent entry) and agenda panel (3 lines). Personas: haiku Agents with act.py only, caps 45 actions / 20 looks (used 25/13, 38/20, 27/13), no coaching. fuzzel 1.9.2 here (the launcher click worked for the sofa).

### cook (25 actions, 13 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 turn the page | pass | 0 | yes: Right 1 to 6, Left back to 1 (page 5 is "5 TERMINAL", plain arrows, hollow cursor, no page 7) |
| T2 key list Super+F1 | pass | 0 | yes (list read, Escape closed it) |
| T4 clock | pass | 0 | yes (16:28, 16:29) |

### sofa (38 actions, 20 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 | pass | 0 | yes (Right/Left through all pages) |
| T1 start a program | pass | 0 | yes: Super+Space, click on the Foot row (page 7 FOOT) |
| T3 helper | pass | 0 (r4: 8 x I3) | no by the persona: two questions, two "Hello" (env); chat opened over the Foot page without a breach |
| T4 volume | pass (wpctl set-volume 5%+ in calls.log) | 0 | not verified: the Volume readout is hidden while the chat is open (ux) |

### nurse (27 actions, 13 looks, xp+mc, xp first)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T6 honest failure | pass | 0 | yes, in xp Run: "Nothing found for nosuchprogram" (xp, not mc) |
| T3 helper | pass | 0 | persona saw "Hello" (env) |
| T7 xp -> mc | pass (vacuous: no handover, set unchanged) | 0 | no: Control Panel, Desktop, Display all grey, 3 attempts |

### Fix check (mc findings fixed since r1)
| row | persona reached? | holds | capture |
|---|---|---|---|
| mc-42 BLOCK first arrival | yes: cook and sofa, the first Right from page 4 | PASS: cook lands on "5 TERMINAL" (plain form, hollow cursor, Right goes on to 6 Overview, Left back); no page "7 FOOT" appears from the page turn. The sofa's launcher start gives page 7 FOOT as it should | cook 005-006; sofa 013 |
| I3 chat over a window page (mc-53) | sofa yes (Foot page 7, then Super+a, two questions) | PASS: 0 breach lines (r4: 8) | sofa 014, breaches.jsonl empty |
| mc-52 I3 xp+mc | nurse yes | PASS: 0 I3 (r4: 1) | nurse breaches none |
| mc-43 Super+F1 on a Foot page | no (cook used Super+F1 on page 1 only; sofa did not press it) | not reached | |
| mc-44 unknown program in mc | no (nurse used xp Run) | not reached | |
| mc-46 remote key out of a window | no (no XF86Back or XF86HomePage) | not reached | |
| mc-48 I5 at Super+a | sofa yes | PASS: 0 I5 | sofa breaches |
| mc-49 xp+mc quiet | nurse yes | PASS: check.py start quiet | nurse start |
| mc-51 I5 xp+mc | nurse yes | PASS: 0 | nurse breaches |
(mc-45 guard, mc-50 yellow square: not seen. mc-47 launcher click: worked, fuzzel 1.9.2.)

### triage.py (r5 files; `--format`: FORMAT OK)
```
== construct / I1 (2)
== construct / I7 (5)
== construct / contract C1 (2)
== construct / contract C2 (4)
== construct / contract T1 (1)
== construct / contract T4:lock (2)
== construct / contract T5 (2)
== construct / none (22)
== mc / I3 (1)
== mc / I5 (1)
== mc / cell gate-mc-paths terminal-back (2)
== mc / cell gate-mc-paths terminal-turn (2)
== mc / contract M1 (2)
== mc / contract T1 (4)
== mc / contract T2 (2)
== mc / contract T3 (1)
== mc / contract T4:volume (2)
== mc / contract T6 (2)
== mc / contract T7 (1)
== mc / none (41)
  2026-11-20-p2r5-cook.md: ux: Mission Control shows "No data yet" and an empty red "Journal, errors today" heading (0 days)
  2026-11-20-p2r5-cook.md: ux: page 5 Terminal has no footer naming its neighbours (0 days)
  2026-11-20-p2r5-cook.md: ux: the Morning page says no briefing yet, the collector has not run (0 days)
  2026-11-20-p2r5-sofa.md: ux: the Volume readout is hidden while the chat panel is open (0 days)
  2026-11-20-p2r5-sofa.md: ux: Atlas answers "Hello" to a repeated question (env) (0 days)
== tiling / contract T1 (1)
== tiling / contract T4:power (1)
== tiling / contract T6 (1)
== tiling / none (11)
== xp / I5 (1)
== xp / contract T1 (3)
== xp / contract T2 (1)
== xp / contract T3 (1)
== xp / contract T4:clock (4)
== xp / contract T4:lock (2)
== xp / contract T4:volume (2)
== xp / contract T6 (3)
== xp / contract X2 (2)
== xp / none (33)
  2026-11-20-p2r5-nurse.md: ux: Display Properties dropdowns look greyed and do not open (0 days)
== xp+mc / I3 (2)
== xp+mc / I5 (1)
== xp+mc / none (5)
  2026-11-20-p2r5-nurse.md: ux: no visible way in xp to give the screen to the other look (0 days)
  2026-11-20-p2r5-nurse.md: ux: Atlas answers "Hello" over the seeded answer (env) (0 days)
waiting 175, overdue 27, missing ux 0
FORMAT OK
```

### Stop rule (p2-plan section 3)
mc MEETS round 5: no new BLOCK or MAJOR backed by a check or capture. 0 breaches in all three runs; all 8 contracts green; mc-42 (first arrival), mc-53 and mc-52 hold on a persona. Open: ux rows only (Volume readout hidden under the chat, no visible way to hand the screen to another look in xp (T7 vacuous, third round), Mission Control empty states). Not reached by a persona and so unproved: mc-43 (Super+F1 on a Foot page), mc-44, mc-46. No ledger edits.

Files: feedback/2026-11-20-p2r5-{cook,sofa,nurse}.md; captures media/2026-11-20/p2r5-{cook,sofa,nurse}/.
