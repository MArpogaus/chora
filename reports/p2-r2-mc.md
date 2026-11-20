# p2-r2-mc (RC 2, P2 item 1, persona round 4)

Lane: cloud, branch claude/p2-r2-mc at dac835dd (mc-remote, contains mc-p2-fix 17d9a798). No code changes. Step 0: unshallow, `cargo check --workspace` and `cargo build --workspace` green.

## r2
Binaries of the round: sha dac835ddd00ea88f9e431b8fddafa15ff5fa5b04, snapshot /tmp/persona-round/bin (round.sh snapshot once). Sessions mc 1280x800, PERSONA_CHECK=1, 2 slots (cook and sofa together, nurse after). Seeded after each start: helper transcript (user + agent entry in voice/entries.json) and one agenda panel (`panel.set agenda today`, 3 lines). Personas: haiku Agents with act.py only, caps 45 actions and 20 looks (used 18/9, 32/16, 22/11), no coaching. `check.py end` run at the end of each persona (not between tasks). Sofa used XF86Back and XF86HomePage.
Deviation: nurse T7 (xp -> mc) not run. `round.sh start nurse xp+mc` came up, `check.py start` failed again with NotQuiet; written up as an issue (feedback p2r2-nurse). The nurse ran on plain mc. Traceback:
```
harness.NotQuiet: still busy after 10 s: mc:write:mc-s-agenda-panels-today
  (check.py main -> run.snapshot -> harness.wait_quiet, harness.py:236)
```
Write path: node `mc-s-agenda-panels-today`, written by mc itself while xp is the second look (agenda panels of scene agenda); it is the same node as in r1 (5 of 5), also with the panel seeded at r1 or not (here the seed came after the failed start; the failure is before the seed).

### cook (18 actions, 9 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 turn the page | FAIL "nothing changed" (checker: start equals end, the persona turned to page 3 and back) | 0 | yes |
| M2 agenda item and back | pass, but vacuous (open [] and focus mc) | 0 | no (Return, Down, Return did nothing) |
| T2 key list Super+F1 | pass | 0 | yes ("ALL KEYS" read) |
| T4 clock | pass | 0 | yes (05:31, small and grey) |

### sofa (32 actions, 16 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 | FAIL (same checker gap) | 17 x I5 at one action (Super+a, action 18) | yes |
| T1 start a program | FAIL (open [mc-keys] left by the persona's own Super+F1, no new focused window); Foot window started | see M1 | yes by keyboard; click on the launcher row Foot did nothing |
| T3 helper | pass | 0 | yes ("Hello") |
| T4 volume | FAIL only on open [mc-keys]; calls.log has `wpctl set-volume ... 5%+` twice | 0 | no (label stays 50%: the stand-in always answers 0.50) |

### nurse (22 actions, 11 looks, plain mc)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M2 | pass, vacuous | 0 | no (click, Return, double-click did nothing) |
| T6 honest failure | FAIL (checker knows only a notice or No helper) | 0 | yes: launcher said `Nothing found for 'nosuchpro...'` (shot 012) |
| T3 helper | pass (focus back with mc) | 0 | yes |
| T7 xp -> mc | not run (NotQuiet, see above) | | |

### Fix check (mc findings fixed since r1)
| row | act line | persona reached? | holds | capture |
|---|---|---|---|---|
| mc-42 BLOCK (terminal page swallows keys, no column) | gate-mc-paths terminal-back | sofa yes (Foot page); cook no (stayed on pages 1-3) | pass: the Foot page has its column, doors and KEYS list; Left and Home leave it | sofa 006 (column beside Foot), 009, 015 |
| mc-43 MAJOR (Super+F1 draws nothing on a Foot page) | gate-doors mc keys-key on a window page | not reached: Super+F1 pressed only on page 1 (cook 016, sofa 028) | not reached (cell is the proof); the key list on pages draws | cook 016, sofa 028 |
| mc-44 MAJOR (unknown program silent) | gate-mc-sofa launcher-none | nurse yes | pass: `Nothing found for 'nosuchpro...'` | nurse 012 |
| mc-45 (focus after chat Escape) | gate-mc-paths chat-focus | nurse and sofa closed the chat with Escape | pass: T3 green on both, focus back with mc | nurse 022, sofa 024 |
| mc-46 MAJOR (no key out of a window for a remote) | gate-mc-sofa remote-back | sofa yes | pass, with a nit: XF86Back on the Foot page needs two presses, the first shows no change (gives the keyboard to the page); XF86HomePage goes to page 1 | sofa 013, 015, 017 |

### New or open from this round (details in the feedback files)
- Click on a launcher row (Foot) does nothing on mc (sofa, shots 005-007; r1 it opened the chat). Typing the name and Return works. Candidate MAJOR/BLOCK for T1 by mouse; the contract was red for another reason, so it needs a cell (a click on a row of the kit list) before the ledger.
- I5 duplicate nodes `mc-r0-*` and `mc-r1-*` at the same place when the chat opens with Super+a (17 lines; the capture shows one clean screen): breach outside known-red; replay to tell a stale set from a double draw. Verify.
- Agenda item: nobody (cook, nurse; r1 all three) could open one; the M2 contract is vacuous (check.py: `Expect(open=[], focus="mc")`). A ruling is needed whether an agenda row opens; if not, M2 leaves the plan.
- T7 xp -> mc is still untestable: the NotQuiet at `mc:write:mc-s-agenda-panels-today`.
- Checker gaps (not mc defects): M1 compares start with end; T6 does not accept `Nothing found`; T1 and T4:volume read `open` after a persona leaves the key list open; the wpctl stand-in never changes its 0.50.

### triage.py over the three (lines of p2r2-cook, -sofa, -nurse)
```
== mc / I5 (1)
  2026-11-20-p2r2-sofa.md: I5: duplicate region nodes overlap when the chat opens with Super+a (0 days)
== mc / contract M1 (2)
  2026-11-20-p2r2-cook.md: M1 contract red although the pages turned (0 days)
== mc / contract T1 (3)
  2026-11-20-p2r2-sofa.md: Click on a launcher row (Foot) does nothing (0 days)
  2026-11-20-p2r2-sofa.md: T1 contract red: mc-keys open and no new focused window of mc (0 days)
== mc / contract T4:volume (2)
  2026-11-20-p2r2-sofa.md: T4 volume: the key reaches wpctl, the display does not change (0 days)
== mc / contract T6 (2)
  2026-11-20-p2r2-nurse.md: T6 contract red, but the screen said "Nothing found" (0 days)
== mc / contract T7 (1)
  2026-11-20-p2r2-nurse.md: T7: check.py start on xp+mc is not quiet (mc:write:mc-s-agenda-panels-today) (0 days)
== mc / none (27)
  2026-11-20-p2r2-cook.md: Agenda item does not open on Return or Down, Return; no sign of selection (0 days)
  2026-11-20-p2r2-cook.md: ux: a yellow square appears after the first Return and stays (0 days)
  2026-11-20-p2r2-cook.md: ux: the clock is small and grey in the header (0 days)
  2026-11-20-p2r2-nurse.md: Agenda item: click, Return and double-click do nothing (0 days)
  2026-11-20-p2r2-nurse.md: ux: a yellow square and an old chat exchange (0 days)
  2026-11-20-p2r2-sofa.md: ux: the first XF86Back press shows nothing (0 days)
  2026-11-20-p2r2-sofa.md: ux: the volume is a text label with no hint, and the pointer marker looks like a stray block (0 days)
```
(`tools/triage.py` is `triage.py` at the root of this branch; it prints all files, the lines above are the three of this lane; the xp lines of the other lane are left out.)

Stop rule (p2-plan section 3): mc does NOT meet round 4. This triage has new findings backed by a check or capture that are BLOCK or MAJOR candidates: the launcher row click (T1 by mouse, capture) and I5 (invariant, verify). No BLOCK or MAJOR of r1 came back (mc-42, 44, 45, 46 hold; mc-43 not reached); the open P2-met items for mc are M2 (vacuous, no open item), T7 (untestable) and the two candidates above. Round 3 for mc is needed after the two are routed (hard cap 3 rounds).

Files: feedback/2026-11-20-p2r2-{cook,sofa,nurse}.md; captures media/2026-11-20/p2r2-{cook,sofa,nurse}/ (shots, evidence, actions.jsonl, calls.log, breaches.jsonl). No ledger edits.
