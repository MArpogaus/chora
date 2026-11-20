# p2-r3-mc

## r3
Lane: cloud, branch claude/p2-r3-mc at 3d23a3f0 (train 86 plus mc-fix cb2b80f1). Step 0: `git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace` green. No product or test code changed.
Binaries of the round: sha 3d23a3f0012a79cf5986f72ca52aaaf357436ed9, snapshot /tmp/persona-round/bin (round.sh snapshot once). Sessions 1280x800, PERSONA_CHECK=1, 2 slots (cook and sofa together, nurse after, on xp+mc). Seeded after each start: helper transcript (user + agent entry in voice/entries.json) and the agenda panel (3 lines). Personas: haiku Agents with act.py only, caps 45 actions and 20 looks (used 17/9, 32/17, 43/19), no coaching. fuzzel is 1.9.2 here (launcher click: known, not a finding).
Driver slip: for the nurse the `task` lines of `check.py end T6 T3 T7` were cut by a tail of mine and the session was stopped; they are not recorded. The persona says T6 (nosuchprogram, through a terminal, not the launcher) and T3 reached. T7 was not reached (persona on xp never found the way to the other look).

### cook (17 actions, 9 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 turn the page | pass | 0 | no: stuck on the Foot page (Right, Escape, Right dead); back to page 1 not attempted |
| T2 key list Super+F1 | pass | 0 | yes |
| T4 clock | pass | 0 | yes (12:17) |

### sofa (32 actions, 17 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| M1 | pass | 0 | yes |
| T1 start a program | FAIL "no new focused window of mc; new: ['s1']" (Foot started; the persona went home before the end: checker reads the end only) | 0 | yes by Super+space, Foot, Return; the click on the row did nothing (fuzzel 1.9.2) |
| T3 helper | pass | 0 | yes ("Hello", env) |
| T4 volume | pass (set-volume 5%+ in calls.log) | 0 | yes after a click on the label; the key left the label at 50% |

### nurse (43 actions, 19 looks, xp+mc, xp first)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T6 honest failure | not recorded (driver slip) | 77 lines, I3 and I5 (below) | yes by a terminal ("command not found"); the launcher was not used |
| T3 helper | not recorded | same | yes ("Hello") |
| T7 xp -> mc | not recorded | same | no |
Breaches (nurse): I5 'Volume 50%' (mc-r0/r1-right-doors-volume) over '1' (xp-pager-0), '<- 6 Overview' over 'Log Off' and 'Turn Off Computer' (xp start menu); I3 "mc and xp each say they hold the focus" for a dozen actions. Verify (hidden second look counted as shown?).

### Fix check (mc findings fixed since r1)
| row | persona reached? | holds | capture |
|---|---|---|---|
| mc-42 BLOCK (Foot page swallows keys, no way out for a remote) | yes: cook (by page turn, page 5) and sofa (by launcher) | FAIL for arrows and Escape: Right, Escape, Right on the Foot page do nothing to a page change (the shell gets them); column with doors and key list is drawn (pass); Left and Back untried by cook | cook 007, 008; sofa 008, 009 |
| mc-43 MAJOR (Super+F1 on a Foot page) | cook yes | pass: "All keys" list read on the Foot page | cook 009 |
| mc-44 MAJOR (unknown program silent) | no (nurse on xp used a terminal; sofa and cook did not try) | not reached | nurse |
| mc-46 MAJOR (remote has no key out of a window) | sofa yes | partly: XF86HomePage goes to page 1 (pass); XF86Back pressed once did nothing (mc-50: first press); Escape does nothing | sofa 009, 010 |
| mc-48 (I5 at Super+a) | sofa yes (Super+a) | pass: 0 breaches in the run | sofa breaches.jsonl (absent), 011-014 |
| mc-49 (xp+mc never quiet) | nurse yes | pass: `check.py start` on xp+mc quiet (1 of 1 here) | nurse start output |
(mc-45 guard: sofa closed the chat with Escape, T3 green, holds.)

### triage.py (lines of p2r3-cook, -sofa, -nurse)
```
== mc / cell gate-mc-paths terminal-back (2)
  p2r3-cook: The Foot page keeps Right and Escape: a remote cannot leave it
  p2r3-sofa: Escape on the Foot page does nothing, XF86Back once does nothing
== mc / contract T1 (...)
  p2r3-sofa: T1 contract red: Foot started, the persona then left it
== mc / contract T4:volume (...)
  p2r3-sofa: Volume key shows no change; the label changes after a click
== xp+mc / I3 (1)
  p2r3-nurse: I3: mc and xp each say they hold the focus
== xp+mc / I5 (1)
  p2r3-nurse: I5: mc nodes overlap the xp taskbar and start menu while xp is shown
== xp+mc / none (1)
  p2r3-nurse: No visible way to give a window to the other look
== mc / none: p2r3-cook ux (page numbers 5 vs 7), ux none other; p2r3-sofa ux Atlas "Hello" (env)
== xp / none: p2r3-nurse ux Run in the Start menu does nothing
```
(the full output of `triage.py` over all files is long; the grouping lines above are the p2r3 lines of it; the sofa volume line sits in the "mc / none" group.)

### Stop rule (p2-plan section 3)
mc does NOT meet the stop rule: new findings backed by a check or capture: (1) the Foot page keeps arrows and Escape (mc-42 not holding for a remote that arrives by page turn; cell terminal-back is green, so the cell does not cover that path), BLOCK candidate; (2) I3 and I5 on xp+mc, verify. r1, r2, r3 are the hard cap of 3 rounds: the PM decides whether a round 4 runs after (1) and (2) are routed. mc-47 stays "not a bug of mc" (fuzzel 1.9.2). mc-48 and mc-49 hold. No ledger edits.

Files: feedback/2026-11-20-p2r3-{cook,sofa,nurse}.md; captures media/2026-11-20/p2r3-{cook,sofa,nurse}/ (nurse evidence cut to 3 files of 45).
