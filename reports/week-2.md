# Week 2 report (Sat 11-14 to Fri 11-20 23:59 UTC)

Written Fri 11-20 ~20:50 UTC by a sub-agent of the PM from the session log (head eeed10d1) and git. All times UTC.
A sha, file or report name follows each claim; "not verified" marks what I did not check. Still running at 20:45: train t92c (state.md; t92b went red 20:10), lanes tour-one-r2 (F7) and
guide-checks-r1 (F7), see state.md Lanes. Nothing after ~20:45 is in this report.

## 1. In short

- Code: 161 merges (first parent) and 1,361 commits on the session branch since 11-14 00:00; against 8f358fa4 (last
  first-parent commit before that, 11-12 03:08): 669 files, +84,156 and -19,021 lines (`git diff --shortstat`). Test
  attributes (`#[test]`, `#[tokio::test]`) went 1,786 to 2,723 (git grep; this counts tests written, not tests run). 52 of the
  merges are on 11-19, 36 on 11-20.
- Product: rc.4 (v0.9.0-rc.4, d060dc03, 11-17 13:41) is tagged local only, not pushed (owner 11-19 05:59 "Keep local for now").
  The week went into the foundation refactor (shared kit, wm core, size, docs), no new feature. Tour v9 is not done.
- Not reached: F5 (flake half) and F7; RC criteria 2 (persona round 4) and 11 (tour) are open; no 1.0 date (owner 11-17 12:56).
  Nothing was run on real hardware (not verified: no report says otherwise).
- Budget: owner readings 40 % (11-20 01:45), 61 % (09:13), 65 % (14:39 at 71 % time). Over pace before the 11-19 reset,
  under pace after it. About 35 % left for 13 h at 14:39.
- Process: the PM re-learned the same lessons (opus after a stop line, unmeasured budget, briefs for already-fixed items);
  trains were red on flakes six times on 11-20 (audit-f r4). Details in section 6.

## 2. Goals against results

F = design/foundation.md (owner 11-17 10:56, "no partial credit" 10:59); verdicts from reports/audit-f.md r4 (11-20
14:23, head d81c8293, opus lane that did not build the slices). State at 20:43: state.md "F1-F4, F6, F8 met; F5 open; F7 plan
accepted".

| Goal | Evidence | Verdict |
|---|---|---|
| F1 Plan | reports/arch-plan-fable.md, 26 slice rows, two reviews (audit-f r4 s4) | met |
| F2 Shared core | tools/copies.toml 42 rows all `max = 0`, `tools/cut-check` 0 copies lines (audit-f r4) | met |
| F3 WM core | kit `wm/` book, focus, give, kept, open, stack, state; WM4 desks and WM6 seams cut to next week | met (desks cut) |
| F4 Size | 0 files over 1,500 lines; `#[allow]` 360 (RC 378); `eprintln!` 131 (135); debt 25 (47); margin on eprintln is 4 (audit-f r4 s1) | met |
| F5 Tests | kit coverage 91.87 % lines met; flake half open: 14 flake rows with no merged fix, 0 greens at --jobs 2 since t86, flake-series 0 of 5 (audit-f r4 s3) | partly (counts open) |
| F6 Gates | 24 of 24 first-parent commits since 79c8d2b5 are train merges; cut-check selftest ok (audit-f r4 s2) | met |
| F7 No regression | tour v9 not built; known-red rows 0 on P. Plan accepted 11-20 (tour-v9, 9.5 %); build started; the owner said no tour work on 11-17 14:20 until the refactor was done | not met (open by design) |
| F8 Docs | D96-D118, CONTRIBUTING lists every kit module; D95 row still "proposed" in backlog (audit-f r2 rf-1) | met |

RC (design/rc.md, state.md RC table). Owner 11-17 10:47 "quick and dirty rc": rows 1, 2, 9, 11 waived for the rc only.

| # | Criterion | Verdict |
|---|---|---|
| 1 | no BLOCK or MAJOR open | waived (owner 11-17 10:47); findings go to known issues; not met |
| 2 | persona round 4 | partly: xp r4 and mc r5 meet the stop rule; co r3 (11-20 17:12) and tiling r2 (17:04) fail |
| 3 | three greens in a row | met for the rc (30b, 33-40, 9 greens); see F5 for later greens at --jobs 1 |
| 4 | CI green | met (29e2cd77, 11-20 13:13; earlier CI green on train 38/39, 11-17) |
| 5 | packaging smoke | met (arch, rpm, podman on 29e2cd77) |
| 6 | code review per area | met (11-20 08:59: no HIGH, MEDIUM fixed or known issues) |
| 7 | deny, machete | met (ci-rc) |
| 8 | docs, CHANGELOG, known issues | met by state.md; CHANGELOG `## Unreleased` still open as debt B3 (audit-f r4) |
| 9 | logging audit, flakes | partly: logging merged (train 14); flakes open (F5) |
| 11 | tour | not met: v7 MERGE-OK; the new tour is after P2 (owner 11-17) |

## 3. Burn-down

Moved this week (merge dates from `git log --first-parent`; audit-f for verdicts):
- F1: met in audit-f r1 (11-19 14:40). F3: kit wm WM1, WM2, WM5, WM7a/b merged 11-19 (trains 52-63). F6 met r1.
- F2: copies-0, kit3, focus-port; met in r2 (11-19 20:10). F8: docs-f8b-e, D107-D115; met r2.
- F4: cubesplit, debt-down, lint-down and debt-f4; not met in r2 and r3, met in r4 (11-20 14:23).
- RC: CI first ran on train 38/39 (11-17); packaging and rc.4 tag 11-17; code review per area closed 11-20 08:59.
- 11-14 to 11-16: rc phase (UX rounds, security rounds r1-r5, class reopened four times, friction 51-52).
Open at 20:45:
- F5 flakes: 14 rows (audit-f r4: flakes-l owns 7, flakes-m 3, 2 unowned: rw-3, MX-CAP); then 5 of 5 `merge-gate.sh --jobs 2`,
  needs the last trains (about Sat 00:00). Audit-f r4 outlook: "test half met, flake half open at 03:59" (likely).
- F7 tour v9: tour-one r2 (review r1 FIX: rt-1, rt-2 MAJOR) and guide-checks r1 running; v9 lanes 4-5 wait for xp-give.
- Persona round 4: co (co-pane review r2 FIX, rw-1 MAJOR: xp Super chords lost in use mode) and tiling (I3, T7; P2-R3-TILING after t92c).
- t92c running; t93 queue: xp-tidy, co-pane, tour-diff.

## 4. Budget

Owner readings (decisions-now.md, answers.md): 11-16 01:17 38 %; 11-16 12:10 45 %; 11-17 15:35 61 % ("F 10 % all 61 %");
11-17 23:59 62 %; then a full reset Thu 11-19 05:55 ("budget is reset and we shall use it up completely until Sat", 06:04);
11-19 10:39 17 %; 13:50 25 %; 11-20 01:45 40 %; 09:13 61 % ("half of which comes from other agent", session about 30 %);
14:39 65 % at 71 % time (4 % in 5.4 h; 35 % left in 13.3 h = 2.6 %/h shared).

Pace: before the reset the PM was over pace (selfcheck 11-16: 24 %/day against a 12.5 % cap; 11-18: 9 % over). After the reset the session spent less than the owner asked: critic 11-20 13:05 "HIGH under-spending";
state.md 14:39 "under-spending". Reasons: the 5-h window bound on 11-20 (limit hit 06:39-06:47, lanes idle to 07:57, about
70 min, friction 11-20 07:58); setup.sh blocked lanes on 11-19; red trains held the queue; one build at a time on 12 G disk;
the 09:13 reading read as "hold lanes", corrected 13:15. PM share by tokens.py: 11.4 % before
the last compaction, 0.9 % since (19:48). The other 30 % of the week is another agent (owner 09:13); not verified how
it splits. What the 35 % is spent on before Sat 03:59: queue 4-5 in state.md.

## 5. Decisions

PM decisions (state.md, selfchecks, procedures; owner may reverse):
- 11-14 17:17 Owner made the PM the decider ("Du bist der Projektmanager").
- 11-17 about 11:00 rc with waived rows (owner order); foundation week plan, F-rows only after independent check.
- 11-19 about 14:00 each MERGE-OK sha joins the next train; WM7b hosted cube (owner 10:52, "Hosted").
- 11-20 about 05:00 trains at --jobs 1 until flakes-l merges (procedures Train 3); about 13:15 --jobs 1 greens do not count for F5 or
  RC 3 and briefs forbid rerun-to-green (selfcheck 11-20).

Owner decisions (decisions-now.md, UTC):
- 11-14 04:51 agree with (a)-(d); 05:33 parallelize.
- 11-15 01:22 Fable used up; 03:32 end of Fable; 09:27 week resets Sat 03:59, model and budget are the PM's main job.
- 11-16 08:52 spend the whole budget; 09:16 Thu 12:00 cutoff, WIP 6; 11:17 Helper kept; 16:56 one Fable lane, 1.5 h.
- 11-17 10:47 "quick and dirty rc now"; 10:56 new goal: foundation week; 10:59 be critical about "met"; 12:56 no 1.0 date,
  unstable until daily driving; 14:24 refactor, then simulated user test, then new tour; 15:35 decisions agreed, P1 next week.
- 11-19 05:59 rc.4 tag stays local, D95 amends D77, delete release-rc4 (owner, GitHub 403); 06:04 use the week to Sat 03:59;
  06:12 seeded reviews only; 06:56 opus planning, sonnet routine, haiku tests; 07:01 batched trains; 10:52 hosted cube.
- 11-20 01:45 reading 40 %; 08:30 update status page; 09:13 61 %; 14:39 65 %.

## 6. Missed and weaknesses

- Red trains and flakes: t84 (ink@2), t85 (merge fault), t85b (MX-CAP 320 s), t87 to t87d, t88: six flaky reds at --jobs 2 on 11-20;
  the only greens since t86 are at --jobs 1, which hides load races (audit-f r4). t92b went red 20:10 on a build error
  (mc-minor against handover-ti `Focus::holder`), PM fix dcb77733; the merge of two lanes was not built together before the train.
- Earlier reds: t64 (MERGE-OK sha whose last gate ran before its head merge, friction 11-19 14:18), t64b, t64c (matrix cap), t82 (four
  flakes). The launcher-click cell is red 10 of 10 in lane containers (fuzzel 1.9.2 against 1.11.1; audit-f r4).
- FIX verdicts: 41 of the 100 `reports/review-*.md` files contain "FIX" (grep; includes older weeks, not split by week). This week
  co-pane r2 FIX (rw-1 MAJOR), tour-one r1 FIX (rt-1, rt-2 MAJOR), mc-fix r4 reviewed again. mc-42 was "fixed" on a too-narrow cell and reopened by persona r3 (selfcheck 11-20).
- The state.md 11-18 note "gate PASS" was corrected (selfcheck 11-18); audit-f r1 found integ red on a unit test, not the lane gate.
- Wasted lane work: kit-minors brief with five items already closed (~0.5 %), co-majors-r4 for fixes already in (~0.3 %), lint-down
  brief naming an unpushed sha. Rules were added to procedures.md, but they are by hand (guard to build in start_lane.sh).
- Process breaks: five opus reviews after the "no new opus" line (11-15); budget unmeasured 3.5 h (11-15); a 5-h limit missed for
  70 min (11-20); lanes asked for re-authored commits twice (friction 11-19 20:40); 4 merge commits authored "t <t@t>" on the log branch
  (00:20 run, cannot be rewritten without force push; identity guard H-ID is only queued).
- Still not done: rc.4 not pushed or published; release-rc4 branch not deleted (owner); F7 unbuilt; persona co and tiling fail.
  No real-hardware run (not verified).

## 7. Architecture records (docs/decisions.md on the session branch, added since 11-14; the owner may veto)

41 records, D78-D118 (D117 and D118 on 11-20). By date of the first commit (git log -S):
- 11-17 (3cc1450f, 7df5d8a5): D78 injected flag on input; D79 no API keys in the voice agent env; D80 notification image limits;
  D81 log files; D82 watchdog and run-queue wait; D83 root order (retired by D89); D84-D88 cube panes, xp editor, tiling wobbly default,
  launch tracking (D104 revises); D89 z-order of roots; D90-D91 setup and test-script rules; D92-D94 word tables, config parity, render budget;
  D95 packaging and repo cut (accepted 11-19 05:59, amends D77).
- 11-19 03:42 (ba9dd020): D96 FakeCore; D97 acts; D98 actions; D99 window book; D100 ask card; D101 paths; D102 language;
  D103 size ratchet and cut gate; D104 launch pidfd; D105 D92 kept; D106 hosted cube (owner 10:52).
- 11-19 17:19 (65ef41a5): D107-D115 kit focus, open, stack, notice, helper, boxes, cube, keysheet, module map. D116 construct keys for
  a new window (18:24; status still "proposed", audit-f r2 rc-4).
- 11-20: D117 core event `input.ungrabbed`; D118 swallow of unbound Super chords (d3dbaa22).
Whether the owner has read them: not verified.

## 8. Next week (state.md Queue, audit-f r4 s5)

1. F5: flakes-l and flakes-m rows, FLAKE-SERIES 5 of 5 at --jobs 2 on the final head, no rerun-to-green; GATE-SILENT merged (11-20 16:25).
2. F7: tour v9 (tour-one r2, guide-checks, then the build and the fresh-eyes review against v8), by the owner's order of 11-17 14:24.
3. Persona round 4 for co and tiling (P2-R3-TILING, P2-R4-CO), co-pane fix (rw-1), then the simulated-user test.
4. rc.4 decision: push the tag or cut a new one; delete release-rc4 (owner); CHANGELOG `## Unreleased`.
5. WM4 desks and WM6 seams; D116 and D95 status lines; identity guard H-ID; start_lane.sh checks.
6. Budget: the week resets Sat 03:59; slower pace after 11-30 (owner 11-17 12:56).
