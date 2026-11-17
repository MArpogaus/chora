# Plan after the foundation week (2026-11-17 12:5x, PM, sharpened with the opus critic)

Owner 11-17 11:54: "Sharpen this plan further in reflection round. Have unnecessary detailed and
expensive desktop effects land before 1.0 so we can have som cachy sisuals to get attention."
Earlier the same day: foundation first, then UX/UI polish towards perfection, then the collected
features (no new demo sprints); the demo is a real-world workflow in one take (design/demo-playbook.md).

## Facts that bound the plan
- Budget: 58 % used at Tue 12:09; even pace would be 52 %. 42 % left to Sat 03:59 = 11.5 % a day, no reserve.
- Owner 11-17 12:56: no 1.0 target; chora stays unstable until daily driving proves stability. Land as much as possible by 11-30; after that the session goes on at a slower pace.
- The full collected list (N1, N2, DB12, X10-X12, A14, voice, mail) costs about 60-70 % before S3 and the demo.
The plan below fits 11-30 at full pace; the rest follows at the slower pace.

## Phases to 11-30 (each ends at a merge boundary; met = measured, command, independent check)
| Phase | When (cost) | Done when |
|---|---|---|
| P0 rc tag | 11-17 (~2 %) | rc.md with the 10:47 waiver; CI deny green; packaging smoke re-run on the tagged commit |
| P1 foundation | to 11-21 03:59 (~38 %) | F1-F8; F2 limited to the subset the Fable plan marks for this week (rest to backlog); F3 includes the scene effect hook S3 needs |
| P2 slim polish | 11-21 to 11-23 (~25 %) | no BLOCK or MAJOR on any path the playbook uses; MINOR count halved per look; fresh-eyes tour finds no new MAJOR. xp taskbar, tray and Start menu excluded (X10-X12 replace them) |
| P3 core features | 11-23 to 11-27 (~35 %) | N2 settings core, DB12 one cube, X10 XP tray (absorbs the xp tray MINOR rows); each with gate cell and playbook beat |
| P4 effects | 11-27 to 11-28 (~8 %) | S3: one effect (burn) as a plugin on the F3 hook; inside its frame budget (gate); off by default outside the demo |
| P5 one-take demo | 11-28 to 11-30 (~10 %) | playbook beats 2-6 and 9 green 3 times in a row as one e2e run; recorded; reviewed |

## Practices
- The playbook is written in P2 as pending gate cells (demo/playbook.yaml, ~2 %); each phase turns beats green. It replaces another copied tour script (rework audit C10).
- Tour v8 is the F7 baseline; it gets an independent fresh-eyes review before the first refactor merge.
- One phase at a time; a phase opens only when the previous one is met (owner 11-17 10:59, no partial credit).
- Every slice: red test first, merge gate, independent review; Queue lines carry their % cost (routines.md budget ledger).

## After 11-30 (slower pace, same order)
Beats 1, 7, 8 (mail and calendar sources, A14 cylinder and slides), X11, X12, N1 widget host, N2 skins for the other looks, voice tools, more S3 effects. Daily driving by the owner feeds findings; 1.0 only when daily driving proves stability (rc.md in full, persona round 4).

## Critic notes kept (reports: opus critic 11-17 12:5x)
- Waste today: polish and tour rounds on code that is now waived and will be refactored; three briefs written and cancelled within 8 minutes; about 25 one-line log commits for owner input; the quick rc not tagged two hours after the order.
- Risks: deny fix growing into the jiff migration (kept as an ignore, DB13); unreviewed v8 baseline; no budget slack in the foundation week.
