# Week 3 plan: 11-21 04:00 to 11-28 03:59 UTC

Written 11-21 13:1x UTC by the week-plan r1 lane (opus high, brief briefs/week-plan-r1.md). It reads code
on the session head 1c599c74 (`cargo check --workspace` green) and the session-log files at ac434b9c.
This lane changed no code. "Open" means no passing evidence on 1c599c74 or later. Evidence from before
a rule change counts as open (for example, greens at --jobs 1: the rule of 11-20 13:15 says they do not count).

Inputs: decisions-now.md, rules.md, vision-card.md, design/rc.md, design/foundation.md, state.md,
backlog.md, reports/week-1.md, week-2.md, reflections/selfcheck-2026-11-21.md, reports/debt-2026-11-20.md,
reports/flake-series.md, design/tour-v9.md, findings/*.md, GitHub Actions runs and open PRs (read-only).

## 0. Measured here (11-21 13:00 to 13:10, on 1c599c74)
- `cargo deny check`: advisories, bans, licenses and sources ok (exit 0). `cargo machete`: no unused dependencies.
- `tools/size-ratchet`: 0 files over 1,500 production lines. 31 files over 1,000. Top: mc main.rs 1,430,
  assistant chat.rs 1,419, voice agents.rs 1,410, voice core.rs 1,401, core ipc.rs 1,401, core mesh.rs 1,390.
  The ratchet counts `xp/src/wm/tests_moved.rs` (1,176) and `construct/src/cube/tests_construct.rs` (1,029)
  as production code, because its test-name pattern is `test_.*`, not `tests_.*`. Fixed in WM-TESTS.
- `tools/lint-ratchet`: allow 360, no-reason 0, eprintln 131 (list: 360, 0, 131).
- GitHub pre-commit workflow: **red on 1c599c74** (run 44886235380, 11-21 08:45). shellcheck flags SC2128 and
  SC2178 at tests/gate-injected-presses.sh:61-65. The same run is red on every lane push since then (12:54).
  Locally, `pre-commit run shellcheck --all-files` reports only this file. The merge gate checks only the
  files a change touches, so it let this through (earliest gate: CI-HEAD). ci.yaml last ran 11-17 (rc.4, d060dc03).
- Open Dependabot PRs: 9 (7 minor: #108, #107, #94, #88, #86, #2, #1; 2 major: #96 zbus 6, #87 serde-saphyr 2).
- tests/acts/known-red.toml: 0 rows on 1c599c74. When guide-checks merges, it brings about 260 guide rows
  (reports/guide-checks.md r1). Their count after that merge is the baseline, and it may only fall.

## 1. RC table (design/rc.md) on 1c599c74

The rc.4 waiver (owner 11-17 10:47, "For this rc only") does not carry over. Every row must hold for the next candidate.

| # | Criterion | State | Evidence, date | Closed by (slices, section 2) |
|---|---|---|---|---|
| 1 | UX: no BLOCK or MAJOR open, independent pass | open | findings ledgers 11-21 13:00: 3 MAJOR verify lines open (co-36 construct, ti-20 tiling, mc-47 mc); about 260 guide rows wait in guide-checks (MERGE-OK, unmerged); no independent pass on this head | VERIFY-MAJ r2, REVIEW-V9, LOOKS-1, LOOKS-2, P5-FIX, V9-OWNER |
| 1b | Owner sees each look in the final tour | open | no v9 recording exists; tour-one, tour-diff, chapters-a, chapters-b merged 11-20 22:01 to 11-21 07:01 | RECORD-V9, V9-OWNER |
| 2 | Persona round 4: every basic task by mouse, keys, voice | open | xp r4 and mc r5 meet (p2-r4-xp, p2-r5-mc, 11-19/20). tiling r3 (11-20 22:03) has 0 breaches, but L1-L3 are red as harness limits. co r4 (11-21 03:32) fails T2 (co-41): co-keys r1 closed it, then co-keys r3 went red in t99 | CO-KEYS r3, P2-R5-CO, P2-HARNESS-L, P2-R4-TILING |
| 3 | 3 greens in a row, no flake; opt-in gates green once | open | t98 green 08:46 and the greens before it ran at --jobs 1 (do not count, rule 11-20 13:15). flake-series r3 on dcb77733 (11-20 21:33-23:52): 2 of 5 at --jobs 2. r4 runs on 1c599c74. Opt-in gates (battle, watchdog-starved, 0.9-shader-hang, mc-scale, demo): no green found in this week's reports | FLAKE-SERIES r4, FLAKES-S, GATE-WAITS, GATE-DRIFT, FLAKE-SERIES r5, r6, OPT-IN |
| 4 | GitHub CI green on a PR | open | pre-commit red on 1c599c74 (section 0). The green of 11-20 13:13 was on 29e2cd77, a commit of 11-17 11:55 that is 124 first-parent commits behind this head | CI-HEAD, CI-PR |
| 5 | Packaging smoke | open | last green 29e2cd77 (11-20); not run on this head | CI-PR |
| 6 | Code review per area | open (delta) | the 11-20 08:59 review covers code up to about adf71956 (08:38). 34 first-parent merges since then (xp-fields, chapters, handover-ti, mc-minor, lint-down, gate-path and others) are unreviewed per area | REVIEW-DELTA, ASK-ONE (security MINORs DB7) |
| 7 | deny, machete clean; updates applied | open (updates) | deny and machete clean on 1c599c74 (this lane, 11-21 13:0x); 7 minor Dependabot bumps not applied | DEPS |
| 8 | Docs, CHANGELOG, honest known issues | open | CHANGELOG.md:1 is `## Unreleased` with no rc entry. docs/known-issues.md last changed 11-18 (edb94c25) and has 7 "verify" lines (debt H5). R4 mc keys are not in the notes | DOCS-RC |
| 9a | Logging audit, chora-report | met | review-logging-r2 (11-15), train 14; no rule change since. Recheck in REVIEW-DELTA | - |
| 9 | Flakes closed or explained; no known window loss | open | open flake rows: FLK-XP-FACES, FLK-XP-HW, FLK-HUD1, FLK-DOORS-TILING-CHAT. Unreproduced and not closed: FLK-HUD2, FLK-LAY1, FLK-K1, RW-3, F7, F16, F18, A5. A13 (mc drag onto construct loses panes) is not verified | FLAKES-S, VERIFY-MAJ r2, SIM-LONG, FLAKE-SERIES r5, r6 |
| 10 | Real-hardware smoke | waived | the owner runs it before 1.0 | - |
| 11 | Tour with checks, visual acceptance | open | = F7: v9 not recorded or reviewed | RECORD-V9, REVIEW-V9, V9-FIX, V9-OWNER |

Foundation goals carried from week 2 (owner 11-17 15:35):

| F | State | Evidence | Closed by |
|---|---|---|---|
| F5 Tests | open | kit coverage 91.76 % lines (flake-series r3, 11-20 23:52): met. Moved-module tests: met (audit-f r4, 11-20 14:23). No flaky gate: open (2 of 5 at --jobs 2) | FLAKE-SERIES r4, FLAKES-S, GATE-WAITS, FLAKE-SERIES r5 |
| F7 No regression | open | tour v9 not recorded; RECORD and REVIEW not started | RECORD-V9, REVIEW-V9, V9-FIX |

Open RC criteria: 1, 1b, 2, 3, 4, 5, 6, 7 (updates only), 8, 9, 11. Met: 9a. Waived: 10.

## 2. Ranked slices

Slice % = build + seeded review (sonnet 0.3, opus 0.5) + an r2 pool of 30 % of the build cost, for product
slices with a review. Test-only series, reviews and persona rounds cost their build % only. Models follow
the owner's order of 11-19 06:56: haiku for tests, sonnet for routine, opus for complex work and planning.
Fable is not used. Each slice merges only through tests/merge-gate.sh (rules 8). Its gate cell is the
proof, and the lane's full gate on its merge with the session head is needed for MERGE-OK (selfcheck 11-21, item 6).

### A. Carried goals F5, F7 (13.7 %)

| # | Slice | Criterion | Area | Model, effort | % | Done when | Gate cell | Day |
|---|---|---|---|---|---|---|---|---|
| 1 | FLK-DOORS-CHAT r1 (running) | F5, rc 2-3 | tests/doors, tiling launcher | sonnet medium | 0.80 | doors cell 5 of 5 at --jobs 2 on its fix; joins t100 | doors | D1 |
| 2 | CO-KEYS r3 (running) | rc 2, F7 | construct hints | sonnet medium | 0.60 | 3 construct act rows 5 of 5; co-39 closed | acts, construct-hud | D1 |
| 3 | FLAKE-SERIES r4 (running) | F5, rc 3 | test-only series | haiku high | 1.00 | 5 full gates --jobs 2 on 1c599c74, every red named; kit coverage figure | merge-gate x5 | D1 |
| 4 | WEEK-PLAN r1 (this) | plan | planning files | opus high | 2.00 | plans/week-3.md | - | D1 |
| 5 | FLAKES-S r1 | F5, rc 3, 9 | the cells red in r4 + FLK-XP-FACES, FLK-XP-HW, FLK-HUD1/2, FLK-LAY1, FLK-K1, RW-3, F16, F18, A5, F7 | sonnet high | 2.25 | each row: fix sha with red test first, or closed as unreproduced (0 of 50 loaded, 0 of 5 full, diagnostic logged) | the named cells 10 of 10 loaded; full gate --jobs 2 | D2 |
| 6 | GATE-WAITS | F5, rc 3 (W9) | gate scripts that flaked this week: doors, xp-faces, construct-hud, layout, pull, tiling-look | sonnet medium | 1.60 | fixed sleeps in those scripts become until_ok waits on a state; `grep -c sleep tests/*.sh` falls below 673 | those cells 10 of 10 at --jobs 2 loaded | D2 |
| 7 | RECORD-V9 | F7, rc 11 | demo/tour | sonnet medium | 1.20 | 733967aa replay and v9 take, tour-diff, video check, timeline and stills committed | run.py --check; tour-diff.py | D2 |
| 8 | REVIEW-V9 | F7, rc 1, 11 | fresh eyes (not the builder) | opus high | 1.00 | verdict per look against v8; findings in reports/journey-N.md only for changes | review report, rules 7 | D3 |
| 9 | V9-FIX | F7 | the looks REVIEW-V9 names | sonnet high | 2.25 | REVIEW-V9 r2 MERGE-OK; touched chapters re-taken | acts, run.py --check | D3 |
| 10 | FLAKE-SERIES r5 | F5 verdict | test-only series | haiku high | 1.00 | 5 of 5 full gates --jobs 2 on the Tue head, no rerun-to-green: F5 met or the next FLAKES round | merge-gate x5 | D4 |

### B. RC open criteria (with the simulated user test and the looks rounds, which serve rc 1 and 2) (32.8 %)

| # | Slice | Criterion | Area | Model, effort | % | Done when | Gate cell | Day |
|---|---|---|---|---|---|---|---|---|
| 11 | CI-HEAD | rc 4, 5 | tests/gate-injected-presses.sh, tests/merge-gate.sh | sonnet low | 0.69 | SC2128/SC2178 at :61-65 fixed; the gate's pre-commit step runs the CI hook set on all files (selftest) | pre-commit --all-files; GitHub pre-commit green on push | D1 |
| 12 | VERIFY-MAJ r2 | rc 1, 9 | co-36, ti-20, mc-47, A13 (window loss) | sonnet medium | 0.95 | each verify line closed or confirmed with a cell | named cells 5 of 5 | D1 |
| 13 | P2-HARNESS-L | rc 2 | tests/persona | sonnet medium | 0.95 | tiling L1-L3 harness limits, P2-CHECK gaps, ph-1, ph-2, RPM-1 fixed with selftests | persona selftest | D2 |
| 14 | P2-R5-CO | rc 2 | construct persona round | sonnet medium | 0.80 | T1-T7, C1-C3 reached, 0 breaches (after co-keys merges) | tests/persona check.py | D2 |
| 15 | P2-R4-TILING | rc 2 | tiling persona round | sonnet medium | 0.60 | L1-L3 pass after P2-HARNESS-L, 0 breaches | tests/persona check.py | D3 |
| 16 | CI-PR | rc 4, 5 | PM opens the CI PR; fix pool for red jobs | sonnet medium | 0.80 | check, msrv, e2e, coverage, docs, pre-commit, packages green on the PR head | GitHub CI | D3, D7 |
| 17 | DEPS | rc 7 | Cargo.lock, .pre-commit-config.yaml, workflows | sonnet low | 0.82 | the 7 minor Dependabot bumps (#108, #107, #94, #88, #86, #2, #1) applied on the session branch; zbus 6 (#96), serde-saphyr 2 (#87) named in known issues | ci-check; deny; build; test | D2 |
| 18 | DOCS-RC | rc 8 | CHANGELOG, docs/known-issues.md, decisions | sonnet low | 0.82 | Unreleased ready for cz; R4 mc keys in notes; 7 verify lines resolved; known issues match the ledgers; D116, D95 status | docs cell; vale, lychee | D6 |
| 19 | REVIEW-DELTA | rc 6 | seeded per-area review of adf71956..Thu head | opus high | 1.50 | every HIGH and security MEDIUM fixed or a lane named; other MEDIUM to known issues | review report per area | D6 |
| 20 | OPT-IN | rc 3 | battle, watchdog-starved, 0.9-shader-hang, mc-scale, construct-hud, construct-keys, demo | haiku high | 0.80 | each green once on the candidate, logs kept (no evidence found this week) | each script exit 0 | D7 |
| 21 | FLAKE-SERIES r6 | rc 3, F5 | test-only series on the candidate | haiku high | 1.00 | 5 of 5 full gates --jobs 2 on the candidate (covers 3 in a row) | merge-gate x5 | D7 |
| 22 | V9-OWNER | rc 1b, 11 | tour re-take after the looks rounds; fresh-eyes review | sonnet medium + opus high | 1.80 | tour sent to the owner; review leaves no BLOCK or MAJOR | run.py --check; review report | D7 |
| 23 | SIM-LONG | rc 9 (leaks, stalls) | battle script, several-hour session | haiku high | 1.00 | RSS and fd growth per hour in RESULTS.md; no stall over the watchdog bound | tests/battle | D4 |
| 24 | P5-XP | rc 2 (round 5) | xp personas full breadth (two screens, voice EN, DE) | sonnet medium | 0.80 | report with contracts and breaches; findings in the ledger | check.py | D5 |
| 25 | P5-TILING | rc 2 (round 5) | tiling: ricer, beginner, voice | sonnet medium | 0.80 | same | check.py | D5 |
| 26 | P5-MC | rc 2 (round 5) | mc: voice-only EN and DE, remote | sonnet medium | 0.80 | same | check.py | D5 |
| 27 | P5-CO | rc 2 (round 5) | construct: explorer, two screens | sonnet medium | 0.80 | same | check.py | D5 |
| 28 | P5-FIX x4 | rc 1, 2 | one fix lane per look, BLOCK then MAJOR from round 5 | sonnet high | 5.50 | BLOCK and MAJOR of round 5 closed with cells | acts; look cells | D6 |
| 29 | LOOKS-1 x4 | rc 1, vision (looks) | guide known-red rows per look (text size, contrast; xp Tahoma exempt) | sonnet medium | 5.50 | known-red rows per look fall, none added; tour-diff intended rows named | acts I6; known-red count | D4 |
| 30 | LOOKS-2 x4 | rc 1, vision (looks) | second and last pixel round per look (rules 6) | sonnet medium | 4.46 | same | acts I6; known-red count | D6 |
| 31 | LOOKS-REVIEW x2 | rc 1 | seeded fresh-eyes per round | opus high | 1.60 | verdict per look | review report | D4, D6 |

### C. Refactor, ranked by risk removed per % (debt counts only fall) (36.2 %)

| # | Slice | Criterion | Area | Model, effort | % | Done when | Gate cell | Day |
|---|---|---|---|---|---|---|---|---|
| 32 | HARNESS-CELLS | F6 (red trains) | tools/cells-for, start_lane.sh | sonnet medium | 1.08 | a path-to-cell map fills each brief's --only list (HARNESS t93); selftest | gate-lib selftest | D1 |
| 33 | CORE-RV | rc 9 (core safety) | crates/chora safety.rs, logs.rs, ipc.rs | sonnet high | 1.34 | CORE-L-RV rv-1 (stale suspect disconnects), rv-2, rv-5, CM-R1 with red tests first | test; watchdog-starved | D1 |
| 34 | KEYS-ROUTE | RC bar: no stray keys | xp keys.rs:331, construct choose_manager_of, camera.rs:476 | sonnet high | 1.60 | HT-rx-1, construct rx-1, CO-P2-FIX r2 rw-2, xp Start on a swallowed Super chord; red tests first | injected-presses, handover-shared, acts | D2 |
| 35 | LOOP-SLEEPS | rc 9 (stalls), W7, W8 | xp wm.rs:106 ESCAPE_WAIT, construct cube.rs:749, :1440, :1447 | sonnet high | 1.34 | no blocking sleep on a plugin event loop; workarounds count falls by 2 | test; xp, construct cells | D2 |
| 36 | GATE-DRIFT | F6, rc 3 (H11, D90) | tests/merge-gate.sh | sonnet medium | 0.95 | disk and tool-version check before the steps; an old tool gives SKIP, never red; selftest | merge-gate selftest | D2 |
| 37 | ASK-ONE | rc 6 security (A19, RF3, DB7) | chora-kit ask, assistant voice.rs, card.rs, voice client.rs | opus high | 2.45 | one confirm wire type and answer call in kit; Unshowable guard kept for a replaced question (DB7 a); fits() counts lines (b); client test asserts REJECTED (c) | test; secfix cells; e2e agent.sh | D3 |
| 38 | GATE-ALIVE | F6 (D91, B4, H3, H4) | 6 unreferenced construct gates, voice-acp.sh, voice-events.sh | sonnet medium | 1.34 | each script in merge-gate (step or opt-in) or deleted; T2, T3 closed | merge-gate --list; cut-check | D3 |
| 39 | LAUNCH-TIMER | workaround B2, DB11 F-2 | chora-kit launch.rs | sonnet high | 1.60 | 30 s GIVE_UP and StatusFile gone now pidfd owns the child; windowless editor not counted as served | kit launch tests; xp-notepad-fallback; launcher cells | D3 |
| 40 | WM6 | F3 rest, AR14, RF7 construct | construct pane flags to enums with a transition table; co-21 seam | opus high | 3.75 | table test; I13 green without a known-red row | acts I13; construct-extra | D4 |
| 41 | WM-TESTS | F5 (N2) | FakeCore wm_with_window helper; xp wm tests | sonnet medium | 1.34 | xp wm tests use the helper; tests_moved.rs and tests_construct.rs named *_tests.rs so the ratchet counts them as tests | test -p chora-xp; size-ratchet | D4 |
| 42 | KIT-COV | F5 | chora-kit runtime.rs (72.9 %), icons.rs, launcher.rs | haiku high | 0.60 | each file at least 85 % lines | cargo llvm-cov -p chora-kit | D3 |
| 43 | TI-XDG | rc 6 (KIT-SEC MINOR-1, H-mkdir) | flavors/*.sh | sonnet low | 0.82 | one rule for scripts without XDG_RUNTIME_DIR; pre-commit check for mkdir -p without -m 700 | pre-commit; tiling-empty | D2 |
| 44 | H-GUARDS | rules 9-10 guards | pre-commit, lanes.py, lint-ratchet | sonnet low | 0.82 | H-ID identity check on session-log; Queue % vs left - 5 %; a raised ratchet needs ratchet-raise: | selftests | D2 |
| 45 | LOG-DOWN | rc 9a, debt | construct (42), mc (32) eprintln | sonnet medium | 1.60 | eprintln 131 -> 57 or lower through tracing; ratchet list lowered | lint-ratchet; test | D3 |
| 46 | RF7-XP | debt RF7 | xp window.rs:105 held, startmenu.rs:147 | sonnet medium | 1.08 | bool beside a Mode becomes a Mode variant; refactors count falls | test -p chora-xp; xp acts | D5 |
| 47 | ON-PATH | debt RF2 | kit desktop_entry.rs, launcher.rs, xp session.rs, voice config.rs | sonnet low | 0.69 | one on_path in kit; copies row max 0 | cut-check; test | D5 |
| 48 | GATE-LIB | DB8-DB10 (no-bus fault) | shared notify.sh, gate helpers into gate-lib.sh | sonnet medium | 1.34 | power.sh no-bus fault fixed; helpers shared; sleep count falls | gate-lib selftest; tiling-power | D3 |
| 49 | TEST-GAPS | F5 (H13) | grab_action, reload wiring, WD2, WD3, SA9, SA10 | haiku high | 0.60 | a unit test per named gap | test | D4 |
| 50 | SPLIT-A | F4 headroom | mc main.rs 1,430, assistant chat.rs 1,419 | sonnet medium | 1.60 | each at most 1,200 production lines by pure moves; list lowered | size-ratchet --base; test | D5 |
| 51 | SPLIT-B | F4 headroom | voice agents.rs 1,410, voice core.rs 1,401 | sonnet medium | 1.60 | same | size-ratchet --base; test | D7 |
| 52 | SPLIT-C | F4 headroom | core ipc.rs 1,401, mesh.rs 1,390 | sonnet high | 1.60 | same | size-ratchet --base; test; e2e | D7 |
| 53 | ALLOW-DOWN | debt (lint) | #[allow] 360 | sonnet medium | 1.34 | allow 360 -> 300 or lower, no new allow; list lowered | lint-ratchet; clippy | D7 |
| 54 | WM4 | F3 rest (desks) | kit::wm::desks; tiling tags, then xp desktops | opus high | 5.70 | arch-plan WM4 cells green (view tag, move to tag, urgent, xp desktop switch on screen and faces); only if F5 met Tue, else to reserve | acts; 0.9-tiling-tags; xp-faces | D5 |

Total of the slices: **82.7 %** of the week. Reserve: 5 %, held for Fri (fix rounds after FLAKE-SERIES r6 and
CI-PR). The PM loop is about 1 % a day, and it includes the 5.2 % spent by 12:54 on 11-21 (tokens.py).
Together that is about 97 % of the week, so it spends the reset budget (owner 11-19 06:04) with a 3 % margin.

Ranking rule for C: risk removed per %. Safety and key-routing faults on the RC bar come first, because each
one can lose input or a client: CORE-RV, KEYS-ROUTE, LOOP-SLEEPS. Next come harness holes that cost red
trains: HARNESS-CELLS, GATE-DRIFT, GATE-ALIVE. Then the security path (ASK-ONE), workarounds (LAUNCH-TIMER),
state machines (WM6, RF7-XP), and counts (LOG-DOWN, ALLOW-DOWN, SPLIT-*). WM4 is last: it is the largest
slice and removes the least risk per %. It starts only if FLAKE-SERIES r5 meets F5 on Tue; otherwise its
5.7 % goes to the reserve and to FLAKES rounds.

## 3. Day plan (6 cloud lanes, about 13 % a day; one train at a time on the PM machine)

Day = 04:00 to 03:59 UTC. The % per day comes from section 2 (a slice spread over two days counts half on each).

| Day | % | Lanes start (model) | Trains and gates | F and RC moved |
|---|---|---|---|---|
| D1 Sat 11-21 | 8.5 (+5.2 spent by 12:54 = 13.7) | running: FLK-DOORS-CHAT, CO-KEYS r3, FLAKE-SERIES r4, WEEK-PLAN. Next as lanes free: CI-HEAD (sonnet low), CORE-RV (sonnet high), VERIFY-MAJ r2 (sonnet medium), HARNESS-CELLS (sonnet medium) | t100: guide-checks, ti-29, mc-minor, handover-mo, co-keys r3, flk-doors-chat fix. Set the known-red baseline at the guide-checks merge | rc 4 pre-commit green on push; F5 series result |
| D2 Sun 11-22 | 13.2 | RECORD-V9 (sonnet medium, on the t100 head), FLAKES-S r1 (sonnet high, the reds of r4), GATE-WAITS, KEYS-ROUTE, LOOP-SLEEPS (sonnet high), P2-HARNESS-L, P2-R5-CO (after co-keys merges), DEPS, GATE-DRIFT, TI-XDG, H-GUARDS (sonnet low or medium) | t101 (harness first: HARNESS-CELLS, GATE-DRIFT, CI-HEAD), t102 (product) | rc 2 co; rc 7 updates; F7 recorded |
| D3 Mon 11-23 | 13.2 | REVIEW-V9 (opus high), V9-FIX (sonnet high), ASK-ONE (opus high), P2-R4-TILING, GATE-ALIVE, LAUNCH-TIMER, LOG-DOWN, GATE-LIB (sonnet), KIT-COV (haiku). PM opens the CI PR on the t102 head (CI-PR) | t103, t104 | F7 met by Tue 03:59 (REVIEW-V9 r2 MERGE-OK); rc 2 tiling; rc 4, 5 on the PR |
| D4 Tue 11-24 | 14.0 | FLAKE-SERIES r5 (haiku, on the Tue head), WM6 (opus high), WM-TESTS (sonnet), TEST-GAPS, SIM-LONG (haiku), LOOKS-1 xp, tiling, mc, construct (sonnet medium, after v9 is recorded so v8 against v9 holds), LOOKS-REVIEW r1 (opus) | t105, t106 | F5 verdict by Wed 03:59; rc 9 leaks |
| D5 Wed 11-25 | 12.3 | WM4 (opus high, only if F5 met), P5-XP, P5-TILING, P5-MC, P5-CO (sonnet medium; simulated user test, owner 11-17 14:24), SPLIT-A, RF7-XP, ON-PATH (sonnet) | t107, t108 | rc 2 round 5 findings in the ledgers |
| D6 Thu 11-26 | 13.1 | P5-FIX x4 (sonnet high, BLOCK then MAJOR per look), LOOKS-2 x4 (last pixel round, rules 6), LOOKS-REVIEW r2 (opus), REVIEW-DELTA (opus high, adf71956..Thu head), DOCS-RC (sonnet low) | t109, t110; candidate freeze Fri 06:00 | rc 1, 6, 8 |
| D7 Fri 11-27 | 8.5 + 5 reserve | On the candidate: FLAKE-SERIES r6 (haiku), OPT-IN (haiku), CI-PR on the candidate, V9-OWNER (re-take and opus review, sent to the owner). Merging after the candidate verdict: SPLIT-B, SPLIT-C, ALLOW-DOWN (sonnet). Week report (Friday routine) | series and CI on the candidate; no product merge between 06:00 and the verdict | rc 1b, 3, 4, 5, 11; candidate verdict; Needs owner: push the tag or keep it local (default: local, as with rc.4) |

Rules that shape the plan: one area per lane and one lane per look (rules 5), so LOOKS-1 and P5-FIX run as
four look lanes and never next to another lane on the same look. No MINOR work on a look with an open MAJOR
(rules 6): LOOKS rounds on a look start after its VERIFY-MAJ line and P5 BLOCK/MAJOR rows are closed.
Each handback re-ranks Queue 4-5 (routines.md By event). A slice that slips moves to the next day; nothing
new is added unless it closes a named criterion.

## 4. Check against vision-card.md

Stability and looks come before anything new. Every slice above is stability (flakes, gates, core safety,
key routing, loop stalls, leaks), looks (guide rows, pixel rounds, the tour), RC evidence (CI, review, docs,
deps), or refactors that keep behaviour as it is (splits, counts, kit moves, enums). No slice adds a user-visible feature.
- WM4 (kit desks) was checked: it moves tiling tags and xp desktops into kit, and behaviour stays as in
  tour v9. It is kept, last in rank and only behind a met F5.
- Dropped as new features (they stay in backlog.md): N2 settings service and control panel, X8 Run
  dialog, X10 tray popups, X11 Quick Launch and taskbar grips, X12 Start menu pinned and recent, S3 effects pack,
  K4 and K12 cube stand-back and pull-out, AGENDA-OPEN, C0 voice tool registry, B2 and B3 chora-system,
  D0-D3 demo media, xp volume slider drag (xp-minor pr1-2), A15 temporary grants.

## 5. Rule 1 draft for the new week (228 bytes; today's rule 1 is 232, so rules.md stays at 3,000 or below)

```
1. Goal. F7 by Tue 03:59, F5 by Wed 03:59; rc.md, no waiver, on one candidate
(freeze Fri 06:00); then plans/week-3.md slices to Sat 03:59; no new feature.
Queue lines name criterion and %. Burn-down: state.md. Guard: lanes.py.
```

It changes at the next merge boundary (rules 10), not in the turn it is read.

## 6. Notes for the PM (found while planning; not slices of their own)
- The pre-commit workflow has been red on the session head since 08:45 and on every lane push since. CI-HEAD
  is in the D1 queue, and its gate change closes the gap that let this through.
- state.md RC rows 3, 4 and 5 say met. On this head they are open (section 1). This table replaces them.
- Size ratchet headroom: five files sit within 100 lines of 1,500. Any fix there breaches the ratchet
  unless SPLIT-A, B or C lands first. The splits are pure moves, so they can merge after the freeze.
