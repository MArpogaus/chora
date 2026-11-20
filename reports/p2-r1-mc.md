# p2-r1-mc (RC 2, P2 item 1)

Lane: cloud, branch claude/p2-mc at b97369d1 (no code changes). Binaries of the round: sha b97369d1cd9a3c9837ceaf756cc115c0dc5c113b, snapshot /tmp/persona-round/bin. Step 0: unshallow, cargo check and build green.

## r1
Setup: round.sh snapshot once, sessions mc 1280x800, PERSONA_CHECK=1, 2 slots (cook and sofa together, nurse after). Helper transcript seeded in voice/entries.json (user + agent entry) after each start. Personas: haiku Agents with act.py only, caps 45 actions, 20 looks. Breaches (breaches.jsonl): 0 in all three runs. `check.py end` was run once per persona over its tasks at the end, not between tasks.
Deviation: nurse's T7 (xp -> mc) not run. A session xp+mc started, but `check.py start` failed 5 times with NotQuiet "mc:write:mc-s-agenda-panels-today" (the mc look never quiets beside xp: harness or a real mc write loop, to read). The nurse ran on a plain mc session instead (M2, T6, T3).
Contract caveat: "expected open [], got [mc-keys]" on cook M2/T2/T4:clock and sofa T1/T4:volume comes from a Super+F1 press that opened a sheet the persona never saw (the sheet is not drawn over the terminal page); it is a finding in itself (cook, sofa T2), not a defect of each task.
Harness (not issues): no agenda item seeded, so M2 is untestable (all three); the helper answers only "Hello" (fake voice); the seeded transcript shows in the chat; clock 02:01 is VM time.

### cook (21 actions, 11 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 turn the page | pass | 0 | yes |
| M2 agenda item and back | FAIL (open mc-keys); harness: no agenda item | 0 | no |
| T2 key list Super+F1 | FAIL (open mc-keys; words for lock, power, screenshot, volume missing) | 0 | no (nothing drawn) |
| T4 clock | FAIL (open mc-keys only; clock read by eye) | 0 | yes |

### sofa (35 actions, 17 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 | pass | 0 | yes |
| T1 start a program | FAIL (open mc-keys; new window ok); first click on Foot opened the chat | 0 | yes (2nd try, keyboard) |
| T3 helper | pass | 0 | yes |
| T4 volume | FAIL (no wpctl set call, open mc-keys) | 0 | no |

### nurse (31 actions, 13 looks, plain mc)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M2 | FAIL (focus holders []); harness: no agenda item | 0 | no |
| T6 honest failure | FAIL (no notice, no No helper words) | 0 | no (silent) |
| T3 helper | FAIL (focus with None, not back with mc) | 0 | helper answered "Hello"; persona: no |
| T7 xp -> mc | not run (see Deviation) | | |

Real candidates for the PM: arrows and Escape dead on a terminal page (cook, sofa; no way back by remote); Super+F1 draws nothing over a terminal page yet `open` holds mc-keys; click on launcher row Foot opens the chat; no volume path found (T4:volume); nonexistent program gives no notice (T6); focus None after the chat closes.

### triage.py over the three
```
== mc / contract M1 (1)
  2026-11-20-p2r1-cook.md: Arrow keys and Escape do nothing on the terminal page: no way back by remote (0 days)
== mc / contract T1 (1)
  2026-11-20-p2r1-sofa.md: A click on the launcher row Foot opens the helper chat (0 days)
== mc / contract T2 (2)
  2026-11-20-p2r1-cook.md: Super+F1 shows no key list, yet the sheet counts as open (0 days)
  2026-11-20-p2r1-sofa.md: Super+F1 shows nothing; mc-keys left open (0 days)
== mc / contract T3 (1)
  2026-11-20-p2r1-nurse.md: Focus is with nobody after the helper chat closes (0 days)
== mc / contract T4:volume (1)
  2026-11-20-p2r1-sofa.md: No way to change the volume found: no entry in the launcher, no key, no door (0 days)
== mc / contract T6 (1)
  2026-11-20-p2r1-nurse.md: A program that does not exist starts silently: no notice (0 days)
== mc / none (20)
  2026-11-20-p2r1-cook.md: Agenda is empty, nothing to open (0 days)
  2026-11-20-p2r1-cook.md: ux: page 5 is a root shell and its footer promises a way back (0 days)
  2026-11-20-p2r1-nurse.md: Agenda is empty, nothing to open (0 days)
  2026-11-20-p2r1-nurse.md: ux: helper answers only "Hello" to any request, no confirmation (0 days)
  2026-11-20-p2r1-sofa.md: ux: Escape closes the chat before it turns a page; chat opens with an old exchange (0 days)
```

Files: feedback/2026-11-20-p2r1-{cook,sofa,nurse}.md; captures media/2026-11-20/persona-{cook,sofa,nurse}/. No ledger edits.
