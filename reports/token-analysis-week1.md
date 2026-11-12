# Token analysis, budget week 1 (11-09 to 11-12 02:24 UTC)

Read-only analysis of the PM transcript, 236 local subagent transcripts and 22 cloud worker sessions
(claude-code-remote `get_session` usage). Weighted units are the house measure of `reports/swarm-2/usage.py`
(input + 1.25 cache write + 0.1 cache read + 5 output; haiku 0.2, sonnet 0.6, opus and Fable 1.0).
Days are Bangkok days (17:00 UTC to 17:00 UTC), as in `reports/week-1.md`. M = million. "Est." marks an estimate.
Limits: cloud sessions report only cumulative usage, so cloud per-day figures are my pro-rata split over each
session's life (est.); subagents are classified by description text (a few are debatable); PM spend by purpose is
by the tool called in each call; the running tour worker spawned 6 subagents whose usage may not be in its figure.

## 1. Totals

Week to 02:24 UTC 11-12, weighted units (M):

| Where | opus | sonnet | Fable | haiku | all |
|---|---|---|---|---|---|
| PM own turns (2,178 calls) | 111.10 | 0.04 | 0 | 0 | 111.15 |
| Local subagents (236, 12,551 calls) | 126.89 | 107.43 | 8.04 | 2.82 | 245.18 |
| Local total | 238.0 | 107.5 | 8.04 | 2.82 | 356.3 |
| Cloud workers (22 sessions) | 107.06 | 29.06 | 83.20 | 0.01 | 219.3 |
| All | 345.1 | 136.5 | 91.2 | 2.83 | 575.6 |

`week-1.md` section 4 counts only local (338.4 M at 08:55 Bangkok); cloud adds 219.3 M (38 % of all). Cloud list
price is $610.19 (`cost_usd`, 21 sessions with usage); the PM file's last `cost-state` says $876.77 for the PM
and its local agents. Cloud is 41 % of the dollars.

| Day | local opus | local sonnet | local Fable | local haiku | local all | cloud (est.) | all | local /3.8 |
|---|---|---|---|---|---|---|---|---|
| Mon 11-09 | 44.4 | 27.3 | 0 | 0.9 | 72.6 | 0.0 | 72.6 | 19.1 pt |
| Tue 11-10 | 74.3 | 32.3 | 6.0 | 0 | 112.5 | 32.1 | 144.6 | 29.6 pt |
| Wed 11-11 | 88.3 | 0.2 | 2.1 | 1.2 | 91.7 | 162.9 | 254.6 | 24.1 pt |
| Thu 11-12 (to 02:24 UTC) | 31.0 | 47.8 | 0 | 0.7 | 79.5 | 24.3 | 103.8 | 20.9 pt |
| Week | 238.0 | 107.5 | 8.0 | 2.8 | 356.3 | 219.3 | 575.6 | 93.8 pt |

Cloud by model: Fable 83.2 M (6 sessions), opus 107.1 M (13), sonnet 29.1 M (3), haiku 0.01 M (1).

Share of the weekly limit. The user's last reading is 90 % at 02:17 UTC. Local units alone give 356.3 / 3.8 = 93.8
points, so the 3.8 M per point calibration (61.0 M local units from 71 % to 87 %, `week-1.md`) already contains
the cloud work running in that window. Check (est.): from 11-11 16:42 to 11-12 01:26 UTC the tour worker alone is
about 23 M units (8.7 of its 10.9 hours at 28.98 M), so the window holds about 61 + 23 = 84 M for 16 points, 5.3 M
per point. Whole week without the Fable pool (the user resets it separately): (575.6 - 91.2) / 90 = 5.4 M per point;
with Fable 6.4 M. So **5.4 M units per point for all work (local + cloud, Fable excluded)**; 3.8 is a local-only
figure that holds only while cloud runs at the same rate. By day on that basis (non-Fable all): 13.4, 20.4, 36.6,
19.2 points, sum 89.6, in line with the 90 % reading. Wed 11-11 was 37 points because five cloud lanes ran at once.

Token mix, local: cache read 263.5 M units (74 %), cache write 54.2 M (15 %), output 38.6 M (11 %), fresh input about 0.
PM: cache read 99.1 M of 111.1 M (89 %), cache write 5.8 M, output 6.2 M. The bill is cache reads of long contexts.

## 2. Spend by purpose

PM own turns, 111.1 M (31 % of local), 2,178 calls, median context 454 k (max 901 k; auto-compacted at 903 k three
times: 11-09 07:58, 11-10 09:23, 11-11 15:46 UTC; once manually at 183 k). PM share of local spend per day: 41, 27, 39, 19 %.

| PM activity (by the call's tool) | calls | M | % of PM |
|---|---|---|---|
| Turn-closing reply (text only, no tool) | 613 | 30.72 | 27.6 |
| Dispatch and steering (Agent, SendMessage, create_session) | 331 | 19.32 | 17.4 |
| Reading files, logs, git state | 396 | 18.13 | 16.3 |
| Bookkeeping: git commit in the log branch | 283 | 14.54 | 13.1 |
| Gates and cargo | 150 | 8.18 | 7.4 |
| Git merge, push, worktree ops | 145 | 7.52 | 6.8 |
| ReadNotifications | 88 | 4.46 | 4.0 |
| Git commit elsewhere (repo, worktrees) | 69 | 3.23 | 2.9 |
| Cloud session tools, Artifact, edits, other | about 100 | 4.9 | 4.4 |

Log edits by Write/Edit are only 0.16 M; log writing happens inside the commit calls (heredocs). `claude/session-log`
has 397 commits this week (82, 86, 177, 52 per day): bookkeeping is 14.5 M if each commit costs one full-context
call, plus 97 stop-hook calls (3.7 M) that ask for a commit.

PM by what woke it (calls since the latest incoming event of that kind):

| Incoming event | segments | calls | M | calls each |
|---|---|---|---|---|
| Local agent finished (task-notification), PM acts | 358 | 983 | 50.44 | 2.7 |
| User message | 148 | 493 | 24.87 | 3.3 |
| Cloud worker message (remote notification) | 79 | 280 | 14.30 | 3.5 |
| Local agent hand-back report (the "read the report" call) | 231 | 233 | 13.66 | 1.0 |
| Stop hook feedback | 51 | 97 | 3.69 | 1.9 |
| Scheduled trigger (watchdog, self-check) | 11 | 43 | 1.79 | 3.9 |
| System restarts, other | 16 | 49 | 2.40 | 3.1 |

No-action segments (no tool except ReadNotifications, under 500 output tokens): 250, 13.3 M (12 % of PM).

Local subagents (245.2 M):

| Purpose | agents | calls | M | % | models (M) |
|---|---|---|---|---|---|
| Builders (incl. fix work inside a lane) | 59 | 6,339 | 155.3 | 63.3 | opus 78.5, sonnet 74.4, Fable 2.4 |
| Code and security reviews | 84 | 1,623 | 26.5 | 10.8 | opus 15.7, sonnet 6.1, Fable 4.7 |
| Fix rounds (named fix round, review fixes, mc fixes) | 12 | 1,412 | 25.4 | 10.4 | sonnet 22.5, opus 3.0 |
| Design passes | 13 | 557 | 12.1 | 4.9 | opus 12.1 |
| UI reviews | 12 | 710 | 11.6 | 4.7 | opus 10.6, sonnet 1.0 |
| Judges and reflections | 8 | 250 | 5.3 | 2.2 | opus 4.4, Fable 0.9 |
| Swarm testers (haiku) and triage | 36 | 1,352 | 4.3 | 1.8 | haiku 2.8, sonnet 1.5 |
| Research and analysis | 5 | 153 | 2.6 | 1.0 | opus 2.0, sonnet 0.6 |
| Records, reports, hook work | 7 | 155 | 2.1 | 0.8 | sonnet 1.4, opus 0.7 |

Per look, local agents: tiling v2 17.0 M (build 5.0; fix lanes 3.96 + 2.23 + code-review fixes 1.18; UI reviews 1.05 +
2.19 + 0.48; code review 0.23); mc v2 23.0 M (build 3.9; fix lanes 6.42 + 4.69 + 1.36; UI reviews 2.83 + 1.63 + 0.54;
code review 0.37); xp v2 about 15 M so far (build 9.35, continuation 5.09); construct v2 is mostly cloud. Fix and
review rounds are 78 % of mc and 67 % of tiling (mc's first fix lane also carried HUD host work). The second tiling
pixel review (2.19 M) cost twice the first (1.05 M); the confirm-only rule came after it.

Cloud workers (219.3 M, no per-call data): builder and fix lanes 159.6 M (virtual outputs, core stability, HUD looks,
UX loop, desktop basics, voice/chat, cube, marble); showcase, demos and screencast tour 51.9 M (showcase 10.4, demos
12.5, tour 29.0); persona tester 5.3; designer review 2.3; aborted starts (setup.sh or add_repo denied) 0.3, $0.86.

Judges and swarms are not a driver: 5.3 M and 4.3 M, together 4 % of local spend.

## 3. Top 15 single agents or sessions

| # | M | Agent | Calls | Context peak | Produced |
|---|---|---|---|---|---|
| 1 | 35.73 | cloud, Fable, virtual outputs D69 (11-10 08:15 to 11-11 06:34), $118 | n/a | 618 k at end | claude/virtual-2 cc9e59df, gate green |
| 2 | 35.24 | local, opus, "Build construct HUD rework" (11-11 13:19 to 15:57) | 560 | 856 k | construct HUD rework (judge point 3) |
| 3 | 28.98 | cloud, sonnet, screencast tour (11-11 15:11, still running), $65 | n/a | 540 k | tour v2 reshoot in progress |
| 4 | 28.03 | cloud, Fable, core stability (11-10 07:43 to 11-11 06:30), $94 | n/a | 389 k | stale-frame fix ad24ee1a |
| 5 | 24.93 | cloud, opus, UX loop chat/HUD/bars (11-11 09:49 to 12:45), $56 | n/a | 690 k | claude/ux-chat ea58449 |
| 6 | 16.68 | local, opus, "K5: construct on the 3D base" (11-10 08:55 to 23:25) | 329 | 525 k | claude/k5 (reviewed at dc91cbb) |
| 7 | 16.36 | cloud, opus, desktop basics (11-11 07:11 to 09:14), $38 | n/a | 555 k | claude/fix-basics f142bbf |
| 8 | 11.12 | cloud, opus, UX loop construct/cube | n/a | 426 k | claude/ux-construct 73088de |
| 9 | 11.07 | cloud, opus, backdrop blur/marble | n/a | 519 k | claude/marble e341021 |
| 10 | 10.42 | cloud, opus, showcase captures | n/a | 423 k | clips and stills in session-log de69c66 |
| 11 | 10.20 | cloud, opus, voice and chat blockers | n/a | 438 k | claude/fix-chat e17e95dc |
| 12 | 9.35 | local, sonnet, "Build xp v2" (11-11 23:14 to 11-12 00:39) | 408 | 524 k | claude/xp-v2 build; xp gate red on paperclip size (state.md) |
| 13 | 8.72 | cloud, Fable, demo polish D2 | n/a | 504 k | demo clips; ended on an unanswered question |
| 14 | 8.39 | cloud, opus, cube pull-out fixes | n/a | 424 k | claude/fix-cube 5e8f0746 |
| 15 | 6.91 | cloud, Fable, HUD looks finish | n/a | 472 k | claude/hud-looks c09e3bc6 |

The PM itself (111.1 M) is larger than the top three together. Just behind: local opus render findings 6.61, local
sonnet "mc fixes plus HUD host lane" 6.42 (541 k), XP split step X2 6.23 (496 k), flake F9 root cause on opus 5.71.
Eight of the 15 are opus builder, fix or capture lanes that sonnet could do.

## 4. Cost drivers

1. **Cache reads of long contexts**, 74 % of local units. Agent calls by context: under 150 k 6,951 calls 83.0 M;
   150-250 k 2,809 calls 57.8 M; 250-400 k 1,838 calls 52.5 M; 400-600 k 709 calls 29.8 M; over 600 k 244 calls 22.0 M.
   Calls above 250 k are 22 % of calls and 104 M (42 % of agent spend). 24 agents peaked above 250 k and cost 139 M
   (57 %); 7 peaked above 400 k and cost 79.7 M. A 600 k call costs 60 k units in cache read, ten times a 60 k call.
2. **The PM's context.** 51 k units per call. Calls at 400-800 k context: 1,055 of 2,178 and 66 M. The PM ran up to 903 k
   until auto-compaction, three times in four days. Calls above 300 k read 41.7 M units more than at 300 k (upper bound).
3. **Event-driven wakes.** 894 incoming-event segments; each agent finish gives two (notification and hand-back):
   231 report-reading calls (13.7 M, one call each, 59 k) plus 358 acting segments (50.4 M). Cloud workers woke the PM 79
   times for 14.3 M (181 k each). 250 segments did nothing (13.3 M). The reply text is not the cost: the 613 text-only
   calls output 144 k tokens (0.7 M units); the cost is the extra full-context call that ends each turn (50 k each).
4. **Repeated "waiting" notifications.** 98 local agent ids sent 121 task notifications (23 repeats; one id 13 times);
   40 short PM replies say "wait". Their own cost is a few M; the main cost is the segment count in 3.
5. **Gate reruns.** 1,232 agent calls (10 % of calls) and 148 PM calls run a gate script: 31.5 M of agent spend (12.9 %)
   and 7.9 M of PM spend. Runs per agent: K5 90, HUD rework 81, mc build 74, flake F9 74, tiling fix round 3 68, tiling
   fix round 63, handover guard 59, tiling build 57. A foreground gate costs no tokens while it runs; the cost is the
   calls around it, at 100-500 k context.
5b. **Review rounds per look.** Each of tiling and mc had a pixel review, a second, a final narrow review and a code
   review, plus three fix lanes; fix and review rounds are 67-78 % of a look (section 2).
6. **Opus where sonnet would do.** Local opus builders 78.5 M (HUD rework 35.2, K5 16.7, render findings 6.6, flake F9
   5.7, grants step 2 4.3). Cloud opus 107.1 M, nearly all builder, fix and capture lanes. Since 11-12 builders are
   sonnet (rules.md 6): local sonnet is 47.8 of 79.5 M today.
7. **Idle wakes.** Scheduled triggers cost 1.79 M (43 calls): not a driver; the cadence was cut on 11-12 (judge R3-2).
8. **Cloud workers have no context control.** Sessions end at 360-690 k; autocompact is at 784 k; the sonnet tour
   worker has run 11 h at 540 k with 350 M cache reads (29 M units, $65).
9. **Meter blind spot.** `usage.py` sees local units only; cloud was 38 % of the week and Fable cloud lanes (83 M)
   were invisible to the PM's estimate (section 1).

## 5. Rules for week 2 and expected savings

Estimates in M units per week against week 1 (575.6 M); they overlap. At 5.4 M per point, 10 M is 1.9 points.

| # | Rule | Evidence | Saving (est.) |
|---|---|---|---|
| A | **Agent context cap 250 k**, local and cloud: hand over in a note at 250 k, after 3 build steps or one fix round; a fresh agent continues. rules.md 6 says it, but cloud workers are not guarded; put "stop at 250 k used_tokens" in the worker prompt and let lanes.py read it | Agent calls above 250 k read 30.7 M more than at 250 k (59.0 M at 150 k); restarts cost about 0.1 M each (about 5 M) | local about 25 M net; cloud about 25 M (same 12.5 % ratio) |
| B | **PM compaction at 300 k** (manual, with the state card, at a quiet point); never work above 400 k | 41.7 M gross above 300 k, 58.2 M above 200 k; 4 compactions cost 0.1-0.2 M each; risk is context leaks (friction 33) | 30 to 38 M |
| C | **Auto-commit hook for the log**: the stop hook commits whitelisted log paths itself; the PM commits by hand only at merges and rule changes; the 97 "uncommitted" stop-hook turns go | 283 log-commit calls 14.5 M; 97 stop-hook calls 3.7 M | 10 to 15 M |
| D | **Wake cadence**: cloud workers message only at done or blocked; a notification needing no action gets no turn (acknowledge in batches of 15 min); no chat reply after a no-change wake | 79 worker wakes 14.3 M; 250 no-action segments 13.3 M; 231 report calls 13.7 M | 10 to 14 M |
| E | **Partial gates inside a lane, one full gate at hand-over**; at most 3 full gates per agent | 1,380 gate calls 39.4 M; K5 90 and HUD 81 runs | 8 to 10 M |
| F | **Per look: one fix lane, rounds 2 and 3 confirm-only on sonnet, target 10 M** | mc 23.0 M and tiling 17.0 M; fix and review rounds 17.8 and 11.3 M | 8 to 12 M |
| G | **Sonnet for builders, fixes, showcase, screencasts**; opus only for design, UI and security reviews; Fable only for named items | Opus builder, fix and capture lanes: local 78.5 M, cloud about 100 M; sonnet at 0.6 weight and 25 % more tokens still saves 25 % | 25 to 40 M if the week-1 mix returned; in force since 11-12, keep |
| H | Haiku for swarm testers and triage | Swarms and triage are 4.3 M; triage on sonnet 1.5 M | about 1 M, no new rule |
| I | Confirm-only judge rounds on sonnet; one opus judge a day | judges 5.3 M | about 1 M |
| J | Archive a cloud lane at its hand-back; no 11 h workers | tour worker 11 h; aborted starts $0.86 | 3 to 5 M |

A to F and H to J sum to 120 to 145 M; after overlap (A and B both cut cache reads, C and D both cut turns) I expect
**about 100 to 125 M, 17 to 22 % of week 1, or 18 to 23 points**. Rules A and B carry about 60 % of it. G comes on top
only if the old mix would otherwise return.

### rules.md changes

- **Rule 6 (Lanes)**: any agent, cloud workers included, hands over at 250 k context or after 3 steps; the PM starts a
  fresh agent with a note. Guard: lanes.py also reads cloud `get_session` `context_usage.used_tokens` at each watchdog wake.
- **Rule 1 (Truth)**: "one log commit per wake" becomes "the stop hook commits the log; the PM commits by hand only at a
  merge or rule change". Guard: check_log.py counts commits per hour (at most 6).
- **Rule 4 (Routines)**: a wake with no state change ends with no text and no commit; workers message only at done or blocked.
- **Rule 7 (Merge)**: lanes run partial gates; the full gate only at hand-over and in the train.
- **Rule 8 (UI)**: target 10 M units per look; one fix lane per look; rounds 2 and 3 on sonnet.
- **Rule 10 (Budget)**: replace "usage.py units since, at 3.8 M units per point" with "(usage.py local units + cloud
  `usage` of `get_session`, Fable excluded) / 5.4 M per point, as a range of 5 to 6; Fable is a separate pool".
- **New rule 16 (PM context)**: the PM compacts manually at 300 k with the state card and never works above 400 k.
  Guard: lanes.py flags a PM context above 350 k. The file has 15 rules (limit 20), so none needs to go.
- `pm-card.md` still says 2.8 M per point; `week-1.md` says 3.8; both move to 5.4 with cloud. Add a `--cloud` option
  to `usage.py` or `lanes.py` that sums child-session `usage` (list_sessions mine, get_session).

## Data notes

PM events before 2026-11-08 17:00 UTC (0.19 M) are excluded. Duplicate content blocks of one assistant message count
once (message id), as `usage.py` does. Cloud `cost_usd` per M weighted unit is $0.20 to $0.45. Nothing in the data was modified.
