# Foundation week: goal and done criteria

Owner 11-17 10:56 UTC: "New goal for the week: super clean well tested good designed codebase as
foundation for future feature sprints." Applies after the rc tag (today) until the week ends
(Sat 11-21 03:59 UTC). No new feature; visible behaviour stays as in tour v8 (the baseline).
Inputs: reports/priorities-sprint1.md, reports/rework-audit.md, reports/architecture-rc.md,
reports/debt-rc.md; the plan is the Fable lane's (reviewed by two opus reviewers).

Met (owner 11-17 10:59: "Be perfectionist and very critical about the met targets"): a criterion is met only with the measured
value, the command that measured it and an independent reviewer's check; "partly" counts as open; what is still not perfect goes to backlog.

| # | Criterion | Measure | State |
|---|---|---|---|
| F1 | Plan | Fable plan reviewed, sequenced in merge-sized slices, each with its gate; ranked by the work it unblocks (priorities top 20) | not started |
| F2 | Shared core | window lifecycle, system actions, key sheet, ask card, notices, launching, config in chora-kit; every look uses them; no copy left (grep check in the cut gate) | not started |
| F3 | WM core | the shared window-management layer Fable proposes (focus, stacking, desktops, cube service DB12) designed; first slice merged | not started |
| F4 | Size | no production file over 1,500 lines (xp wm.rs split, AR7); reports/debt-<date>.md counts lower than debt-rc.md | not started |
| F5 | Tests | every moved module has unit tests through kit testing (FakeCore); coverage of chora-kit at least 80 % lines; no flaky gate (F17 closed) | not started |
| F6 | Gates | each slice merges through tests/merge-gate.sh green; the dependency cut gate blocks a plugin-to-plugin dependency | partly (cut gate planned) |
| F7 | No regression | tour v9 at the end matches tour v8 per look (fresh-eyes review, rules 7); findings only for changes | not started |
| F8 | Docs | docs/decisions.md records for the new structure; dev-rules and CONTRIBUTING name where shared code lives | not started |

Order: rc tag; Fable plan (F1); slices by rank (F2, F3, F4) with haiku tests alongside (F5);
cut gate (F6); tour v9 and review (F7); docs (F8). Budget: about 40 % for the week, at most 12.5 % a UTC day.
