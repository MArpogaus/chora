# p2-r4-mc

## r4
Lane: cloud, branch claude/p2-r4-mc at 9dd23de6 (train 88b plus mc-fix bb7cbeb8). Step 0: `git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace` green. No product or test code changed.
Binaries of the round: sha 9dd23de6dab2f12f98f349f175d4a74b9fb98cc5, snapshot /tmp/persona-round/bin (round.sh snapshot once). Sessions 1280x800, PERSONA_CHECK=1. Cook and sofa ran together on mc, nurse after on xp+mc (xp first). Seeded after each start: helper transcript (user + agent entry) and the agenda panel (3 lines). Personas: haiku Agents with act.py only, caps 45 actions and 20 looks (used 12/7, 39/19, 42/19; one driver `look` at the start of cook and sofa is in their logs), no coaching. fuzzel is 1.9.2 here (the launcher click worked for the sofa).
Harness note: the cook's Super+Right was refused by the agent's permission classifier, so it never ran; the cook stopped after 2 failed attempts. Not worked around, not a product fact.

### cook (12 actions, 7 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 turn the page | pass | 0 | no: on the Foot page "7 FOOT" (arrived by the Right from page 4) Right is dead; Super+Right refused by the harness |
| T2 key list Super+F1 | FAIL "the key list mc-keys was never open" (never pressed) | 0 | partly: read the KEYS column |
| T4 clock | pass | 0 | yes (15:05) |

### sofa (39 actions, 19 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 | pass | I3 x8 (see T3) | yes, by Super+Right on the Foot page |
| T1 start a program | pass | | yes: Super+Space, click on the Foot row (page 8 Foot-S2) |
| T3 helper | pass | 8 lines I3 "the core focuses s2 of mc, no look says it holds the focus", Super+a to Super+F1 | yes ("Hello", env) |
| T4 volume | pass (wpctl set-volume in calls.log; 50% -> 55%) | | yes by XF86AudioRaiseVolume |

### nurse (42 actions, 19 looks, xp+mc, xp first)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T6 honest failure | pass | 1 (I3 below) | yes, in xp Run: "Nothing found for nosuchprogram" (xp, not mc) |
| T3 helper | pass | | no: 3 questions, 3 "Hello" (env); the real answer is the seeded one above |
| T7 xp -> mc | pass (vacuous: no handover, set unchanged) | | no: no way found in Control Panel, Display, Effects |
Breach: I3 "mc and xp each say they hold the focus" once, at a click (evidence/001-click.png); no I5 (mc-51 holds).

### Fix check (mc findings fixed since r1)
| row | persona reached? | holds | capture |
|---|---|---|---|
| mc-42 BLOCK (Foot page swallows keys, remote cannot leave) | yes: cook and sofa, both by the page turn from page 4 | FAIL on first arrival: Right from page 4 starts Foot and lands on "7 FOOT" with the keyboard in the window (filled cursor, keys "Super+<- ->"), Right does nothing (cook). Holds on the later arrival: sofa Right from page 6 to 7 shows the plain form, hollow cursor, so rv-1 works for a window that is already there. Left and Escape untried. Column with doors and key list: drawn (pass) | cook 006-008; sofa 006 (first), 010-011 (second) |
| mc-43 Super+F1 on a Foot page | sofa (page 8 Foot-S2) | pass: the All keys list draws in the column beside the window (after the chat was closed; with the chat open it does not show: ux row) | sofa 018, 019 |
| mc-44 unknown program silent in mc | no (nurse used xp Run; launcher in mc not tried with a bad name) | not reached | |
| mc-46 remote has no key out of a window | no (no XF86Back or XF86HomePage pressed; sofa used Super+Right) | not reached | |
| mc-48 I5 at Super+a | sofa yes | pass: no I5 (only I3 above) | sofa breaches.jsonl |
| mc-49 xp+mc never quiet | nurse yes | pass: `check.py start` quiet on xp+mc | nurse start |
| mc-51 I5 on xp+mc | nurse yes | pass: 0 I5 | nurse breaches.jsonl |
| mc-52 I3 state lie, xp+mc | nurse yes | partly: 1 line "mc and xp each say they hold the focus" at a click, none after | nurse evidence/001-click.png |
(mc-45 guard: sofa closed the chat with a click on Close, not Escape; untested. mc-50 yellow square: not seen.)

### triage.py (lines of p2r4-cook, -sofa, -nurse; `triage.py --format`: FORMAT OK)
```
== mc / I3 (1)
  p2r4-sofa: I3 while the chat is open over a launcher-started window page: core focuses s2 of mc, no look holds the focus
== mc / cell gate-mc-paths terminal-turn (2)
  p2r4-cook: The Foot page that a Right from page 4 starts keeps Right: mc-42 not holding on first arrival
  p2r4-sofa: ux: the first Right from page 4 lands on "7 FOOT" and Right is dead there
== mc / none (36)
  p2r4-cook: ux: footer names page 5 Terminal, the page that arrives is "7 FOOT"
  p2r4-cook: ux: none other seen
  p2r4-sofa: Super+F1 with the helper chat open shows nothing until the chat is closed
  p2r4-sofa: ux: Listening pill overlaps the date with the chat open on a window page
  p2r4-sofa: ux: Atlas answers "Hello" to what is on today (env)
== xp / none (26)
  p2r4-nurse: ux: Display Properties Theme and Appearance dropdowns look greyed and do not open
== xp+mc / I3 (2)
  p2r4-nurse: I3 once on a click: mc and xp each say they hold the focus
== xp+mc / none (3)
  p2r4-nurse: No visible way to give a window or the screen to the other look
  p2r4-nurse: ux: Atlas answers "Hello" three times (env)
```

### Stop rule (p2-plan section 3)
mc does NOT meet the stop rule at round 4: new findings backed by a check or capture: (1) mc-42 still red for the first Right that starts the terminal (cell terminal-turn green, does not cover a fresh session), BLOCK candidate, the same row as r3, third hit; (2) I3 on mc with a launcher window and the chat open, verify; (3) I3 once on xp+mc, verify. Round 4 is beyond the hard cap of 3 rounds (the PM asked for it); the PM decides on a round 5. mc-44 and mc-46 were not reached and stay unproved by a persona. No ledger edits.

Files: feedback/2026-11-20-p2r4-{cook,sofa,nurse}.md; captures media/2026-11-20/p2r4-{cook,sofa,nurse}/.
