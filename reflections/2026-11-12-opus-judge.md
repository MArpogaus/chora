# Judge of 11-12 (opus, read-only, no PM context)

Brief: the user's words of 11-12 01:41 UTC: "Have a intensive refection loop and fix our
latest problems and issues. The pm shall own the project remain always aware and use tokens
and time efficently. Never loose context ore focus. Titen rules and overall strategy. I shalle
note ever need to steers anymore."

Sources: charter.md (1,047 lines), decisions-now.md, pm-card.md, state.md, friction 26-38,
answers.md, reflections of 11-11 (both), design/product-briefs.md, design/decisions-taste.md,
design/rc.md, reports/review-*-v2*.md, reports/pm-hook.md, tools/pm_context.py, backlog.md,
questions.md, status/index.html, git log of both branches (last 24 h), the scheduled tasks
(list_triggers), get_session of the PM and of the tour worker, and the token usage in the
session's transcripts (usage.py weights: input + 1.25 cache write + 0.1 cache read + 5 output,
haiku 0.2, sonnet 0.6, opus and Fable 1.0). Real clock at writing: 11-12 01:45 UTC (08:45
Bangkok). All times below are UTC unless marked Bkk.

## Data (11-11 01:43 to 11-12 01:43 UTC)

| Measure | Value |
|---|---|
| User messages in the PM chat | 59; about 30 are steering (table in point 1), 4 are usage readings |
| User messages after "No mor intervention from my side" (17:50) | 9 (01:26 to 01:43), 6 of them steering |
| Session-log commits / of them `state` + `decisions` | 201 / 47 |
| Code commits on the session branch / merges | 332 / 29 first-parent |
| Local spend (weighted units) | PM 42.1 M (828 calls); subagents 106.6 M (opus 62.3, sonnet 41.5, haiku 1.9, Fable 0.9) |
| Biggest single agent | construct HUD developer on **opus**: 35.2 M units, 560 calls, context up to 856 k (13:19-15:57) |
| PM context | 359 k to 900 k (03:00-15:59), compaction, 172 k to 449 k now |
| Cost per polished look | tiling about 16 M, mc about 21 M (mc build 3.9 M, its fix rounds 11.1 M) |
| Red commits in the two v2 merges (fe3483f, 59df16e) | 0 of 30 `fix` and 19 `feat` commits |
| Tour worker (now building construct-v2) | context 497 k, 321 M cache read, $61; runs `merge-gate.sh` twice in a loop |
| Weekly use | 71 % at 16:42, 87 % at 01:26: about 1.8 points per hour; 13 points left until Sat 04:00 UTC |

At 1.8 points per hour the weekly limit stops everything about 11-12 08:00-09:00 UTC (15:00-16:00
Bkk), about 43 h before the reset. That fact decides the strategy (point 12).

## Ranked points

Each point: finding, evidence, change, guard. Severity: CRITICAL (the user steers or
work stops), HIGH (wastes a day's budget or ships a visible flaw), MEDIUM, LOW.

### 1. CRITICAL (steering). The user steered about 30 times in 24 h; most causes repeat

Every row is a user message (real time from the transcript), its root cause in the PM's
process, and the guard that would have made it unnecessary.

| Time | User | Root cause | Guard |
|---|---|---|---|
| 11-11 04:43 and 11-12 01:29 | "Check you sub session. All running? Close stale ones"; "Is construct worker still run ng? Remove stale sub sessions" | No tool lists the lanes; pm-card step 2 is prose. Repeat. | `tools/lanes.py` (point 7) run by every watchdog wake; finished sessions archived by it |
| 05:21 | "Whats reqiiuring my Konfirmation? Any trouble with floating?" | The status page has no live "needs you" and no risks | Status header and "Needs you" generated from state.md and questions.md (point 9) |
| 05:22 | "Decide such things on your own from now" | PM asked about a reversible matter (demos) | Rule Q (point 13): a chat question must name the clause that requires the user |
| 06:59, 07:01 | "Pulling from cube not possible? ... collision handling?"; "Not pols, would expect it to reach out into space" | Edge states missing from the design note; the user found a POLS gap | Design note template: every state incl. edges, checked by the UI reviewer |
| 07:04, 07:06, 10:30 | "have a ux/ui loop over the current featured set"; "After ui ux clean. Finalize foundational work and review code until ready for rc" | The user wrote the strategy; the PM had no visible plan beyond the day | A 3-day Queue in state.md and on the status page, refreshed at each merge train |
| 11:05 | "Wait? Rc not landing today?" | rc date and criteria not visible | rc table with dates on the status page (exists since; keep it current, point 9) |
| 10:37, 12:39, 13:19, 11:21, 01:26 (and 11-10 15:20) | "Post screenshots when the UX fixes land" ... "Update status page with screenshots and screen cast when new looks are ready" | Six times. The rule exists (charter 11b, decisions-now) but is a manual step after the merge | A merge of a UI lane is not done until `status/` has its captures: check in the train script (point 9) |
| 11:21, 11:26, 12:39, 17:48 | "Ui reviews shall hunt for perfectionism"; the stretched paperclip; "Why do we have the buttons?"; "Tiling is still pretty ugley" | Reviews checked conformance to a note; builders' captures went out unreviewed (friction 28) | Mechanical layout audit for all looks before review 1 (point 5); unreviewed media labelled |
| 11:30 | "we need different guidelines ... can not generalize the HUD" | A cross-look design made without the looks' people (friction 29) | Every brief links its product-brief section (now possible: design/product-briefs.md) |
| 11:34 | "Make sure you note these frictions and problems" | Friction logged late | `prompt` hook line also says: a user correction is a friction entry this turn |
| 15:08, 15:11, 16:42 | "Haiku test swarm for user tests? Record extensive screen cast"; "suggest to use remaining budget to massifly test with haiku" | The user planned how to spend the week's rest; state.md had no budget plan | Budget line in state.md "Now": reading, rate, hours left, planned use (point 11) |
| 16:47 | "Open task planed for the week still realistic?" | Plan not visible or not believed | as 07:04 |
| 16:53 | "Haiku should use considerable less tokens" | Swarm 2 ignored section 8 (800 px, 90 steps) | Launcher enforces 640 px and 30 steps (8438084): done |
| 16:56 | "You developed tooling to estimate model usage. Not working anymore?" | The cross-check failed silently | Budget line shows the time of its reading; older than 12 h prints "stale" |
| 16:59, 17:30, 17:32 | "Reread the whole charter after compactation ... Context management seems to leaks"; "This must not happen again"; "Have a sub screen the transcripts ... adjust the pm hook" | Compaction dropped rules and a decision (friction 33, 36) | Hooks (done); still fragile: point 2 |
| 17:01 | "Can't do thus here. Add to Laval Claus cobfig" | PM asked the user to edit settings | none needed |
| 17:05, 17:06 | "Pm card hook worked?"; "still aligned with the plan? What where pure latest decitiobs" | No visible proof of the hook's health | Hook output starts with one health line (state Now date, decisions count, unrecorded count) |
| 17:25, 17:27 | "Why not running any more testers?"; "you did not pick up work after compactation" | No resume after compaction (friction 34) | Watchdog (01:30, done) plus `lanes.py` idle check |
| 17:29 | "No, its not this weeks cap" | Paraphrase in the summary (friction 36) | decisions-now.md verbatim (done) |
| 17:48 | "Mc shall rather include in the right margin ... instead of stealing half the screan" | Design flaw seen by the user first | as 11:21 |
| 17:50 | "run xp and construct design passes in parallel too. Not deciding anything now." | PM serialized passes and queued taste questions for the user, following reflection 2 rule 4, which contradicted "Decide such things on your own" (05:22) | One rules file with one precedence order (point 13) |
| 17:53 | "Define clear requirements USPS and target groups for each. Be a wise leader" | Workers were briefed by symptom, no product contract | product-briefs.md (done); template field "brief section" |
| 17:54 | "Stay focused on our main targets" | Scope drift (status rework, L1 paste upload, voice research) | rc scope list (point 12) |
| 01:29 | "Arm a watchdog timer to Lee you going. What ere your next steps? What can be started right away in parallel?" | 00:39-01:30 no local lane ran; the PM made 6 calls in hour 00; plan said "Light work only" | Watchdog (done); rule L: two or more lanes while the week has budget |
| 01:35 | "Wake up more often? ... What about cloud workers? No visual polishing and test swarms planned anymore?" | Plan of 11-12 showed no polish lanes; cloud workers idle since friction 37 | Queue shows every lane; cloud env test (point 12) |
| 01:42 | "Still at 87. Don't be to paranoid about token limit" | usage.py said 93-95 %; the PM throttled; the user said the same on 11-10 15:20 | Budget from the user's reading plus measured rate; no "light work" mode (point 11) |

Change: treat each repeated steering class as a broken rule and give it the guard in the
right column now; the three classes that recur most are media on the status page (6),
lane inventory (2) and budget throttling (2).
Guard: `tools/check_log.py` (point 2) and `tools/lanes.py` (point 7); the next judge counts
steering messages per day from the transcript (the method above) as the headline metric.

### 2. CRITICAL (context). The hooks inject a stale state and a blind decision screen

Finding: the context hooks of friction 33 and 36 work, but what they inject is wrong in
three ways.
- `pm_context.py card` prints the section `## Now` of state.md. That section is dated
  "11-11 17:40 UTC" and says mc-v2 "one fix round running", "xp-v2 waits for disk", tiling
  "Merge train 1 of 3". All three are false: mc merged as 59df16e at 22:57, xp-v2 runs. The
  real plan sits in `## Plan of 11-12`, which the hook never prints. After the next
  compaction the PM resumes from a state 8 h old. The watchdog prompt ("If nothing runs and
  state.md has an open item ... start it") reads the same stale file.
- `screened-through: 2026-11-12T02:56:00Z` is in the future (real time 01:45). The screen
  lists only messages newer than that line, so any user message until 02:56 is hidden from
  the "record now" list. Cause: the PM writes estimated times, not clock times (point 6).
- decisions-now.md is 9,378 characters against its own cap of 9,000. `capped()` cuts at
  9,500 from the end, and the unrecorded user messages are appended last, so the most
  important part is the first to be cut.
- answers.md says "Newest first" but the PM appends at the bottom, and 11-11 entries sit
  under "## 2026-11-10". The hook's "newest 12 lines" is right only by accident.
Evidence: run `echo '{}' | python3 tools/pm_context.py card` (output quoted above);
decisions-now.md line 7; `wc -c decisions-now.md` = 9,378; answers.md lines 3, 12, 255.
Change: state.md becomes three sections only, rewritten (never appended) at each event:
`## Now` (header with `date -u` time, ≤ 30 lines: lanes with ids, last merge, budget line),
`## Queue` (the next 3 days, ordered), `## Blocked`. Everything else (foundation table of
11-09, "Plan of the day" of 11-10, "Next step", "Done") is deleted; git keeps it. The card
mode prints Now and Queue. Unrecorded messages print first, before the file. answers.md
gets one order (append at the bottom; fix the header).
Guard: `tools/check_log.py` as a session-log pre-commit hook: fails when state.md has a
section other than Now/Queue/Blocked or more than 80 lines; when the Now time is older than
the newest merge on the session branch; when `screened-through` is later than the commit
time or earlier than the newest human message in the transcript; when decisions-now.md is
over 9,000 characters. The card mode prints "STATE STALE" when the Now time is older than
3 h while lanes run.

### 3. HIGH (efficiency). One opus developer cost a third of all subagent spend

Finding: the construct HUD developer ran on opus for 2.6 h with a context up to 856 k:
35.2 M units, 33 % of all subagent spend of the day, more than the whole tiling or mc look.
Charter 5 says developers are sonnet; no record says why. Sonnet builders of the v2 looks
also ran to 391-541 k contexts (tiling 5.0 M, mc 3.9 + 6.4 + 4.7 M, xp 9.4 M for 8 of 15 steps).
Cache reads grow with context times calls, so a long-lived builder pays for its own history.
Evidence: subagent transcripts (table above); charter section 5.
Change: builders are sonnet; opus only for design passes, UI reviews and security views;
Fable only for the items decisions-now names after the reset. A builder works on at most
3 build steps or one fix round, then ends with a handover note (branch, done, next,
gotchas) and a fresh agent continues. Fix rounds always start fresh with the findings list.
Guard: `tools/lanes.py` reads `subagents/*.jsonl` and fails (watchdog reports it) for an
agent above 250 k context or an opus/Fable builder not listed in state.md Now with a reason.

### 4. HIGH (honesty, quality). Visible main-path flaws were merged under a "MERGE" verdict

Finding: tiling v2 merged with the white "Chat ..." tab showing behind Keys and the power
icon on the bar ("Not rated (other lane)", review-tiling-v2-r3.md). mc v2 merged with scene 2
dropping the Reading panel at 1280 wide (rated MINOR, review-mc-v2-r3.md N1), on "the first
page a user turns to". The perfection rule of 11-11 19:20 says such flaws block a release
on a main path. The chat-tab lane that state.md calls "needed next" is not in the Plan of
11-12. The status page still shows `img/hud-xp.png` (committed 11-11 11:23), the capture with
the stretched paperclip and the second "Chat ..." balloon that the user rejected at 11:26,
captioned "Paperclip, balloon and chat beside it".
Evidence: the two review files; state.md "Now" Running; status/index.html line 165.
Change: a UI review rates every visible flaw on a main path of the look under review,
whichever lane caused it; the merge waits or the flaw gets a lane that starts the same
day. Replace or remove hud-xp.png on the status page now.
Guard: the review template has a required table "main-path flaws, any lane"; merge-gate
`--push` refuses when the newest `reports/review-<lane>-*.md` of the lane has an open row in
that table (a grep in the train script).

### 5. HIGH (rules, efficiency). Five adopted rules were never built

Finding: answers.md records both 11-11 reflections as "all five top changes adopted".
Built: merge lock (e64a6b5), launcher limits, product briefs, per-look documents. Not built:
- `merge-gate.sh --fast` (reflection 1 rule 2): the script has no such flag (friction 32 and
  33 disagree on which script lacks it). Workers run the full gate; the tour worker runs it
  twice (`for i in 1 2; do tests/merge-gate.sh`), then the PM runs it again in the train:
  three full gates per lane.
- red first checked at merge (rule 3): 0 red commits in 49 fix/feat commits of the two v2
  merges; the gate checks nothing.
- window audit gate cell (rule 4): no `windows.audit` exists.
- `tests/showcase.sh` (reflection 2 rule 1): does not exist; the status page says the showcase
  "is being recorded now".
- layout audit for the helper surfaces of all looks, red first on the paperclip (rule 2):
  `tests/hud-audit.py` exists but runs only in `gate-construct-layout.sh`. Tiling review 1
  then found 13 MAJOR and mc review 2 found six texts stacked on each other, both of the
  kind the audit measures.
- pre-commit over the merged range in merge-gate (rule 5): absent.
- charter ≤ 700 lines (reflection 2): 1,047.
Evidence: `git show origin/...:tests/merge-gate.sh` (options `--only`, `--push`), `git ls-tree`,
the commit counts above, answers.md lines 19, 24.
Change: "adopted" means built and guarded, or it is "accepted, not built" with a date.
Build now, in one sonnet tooling lane: hud-audit in every look's gate (tiling, mc, xp), the
red-first check and the pre-commit range in merge-gate.sh, and a `--worker` mode (build,
test, clippy, the area's gates) for lanes. Drop the window audit or schedule it after the rc.
Guard: `check_log.py` fails a reflection whose numbered points lack an "Answer:" line with
one of built (commit), scheduled (date), rejected (reason) within 24 h.

### 6. HIGH (honesty, context). The PM writes estimated times, about 20 to 70 minutes off

Finding: user messages and events carry invented times. decisions-now: "11-12 ~02:45 UTC",
"~02:50", "~02:55" for messages at 01:41, 01:42, 01:43; "~02:10" for 01:35; "~01:40" for 01:29;
"11-11 ~18:40 UTC" for 17:48. answers.md: "Correction (user, 11-11 ~17:55 UTC)" for 17:29.
state.md 4c "~01:50 UTC" and 4d "~02:20 UTC" were committed at 01:30 and 01:36. Times mix
Bangkok without a label ("11-12 ~08:00 87 %") and UTC, against charter 1.1 ("All times ...
are Asia/Bangkok"). Friction 19 (invented clock times) is back, and point 2 shows the harm.
Evidence: the transcript times above; `git log` of bddad18, f860d98, 339226b.
Change: times come from `date -u` (or the transcript) and carry the zone; one zone for the
log files (UTC, since the hooks and the transcript use it; the status page shows Bangkok).
Guard: `check_log.py` rejects a new line in state.md, decisions-now.md or answers.md with a
time later than the commit time or without a zone.

### 7. HIGH (efficiency). The PM's own turns cost as much as the builders of two looks

Finding: the PM spent 42.1 M units in 24 h, 28 % of local spend, with 828 calls. From 06:00
to 15:59 it ran at 549-900 k context: every call read up to 90 k weighted units of cache.
Much of it was bookkeeping: 201 session-log commits, 47 of them `state` or `decisions`
(eight `decisions` commits between 01:26 and 01:43). One fix agent sent 12 "waiting on its own
background work" notifications in 8 minutes (11-11 06:52-07:00), each a PM wake at about 550 k.
Evidence: transcript usage per hour; `git log --since` counts; task notifications.
Change: one log commit per wake (state, decisions, answers and status together);
agents run gates in the foreground with a timeout, never "start in background and wait";
the PM reads only the verdict and the MAJOR rows of a review (reports are ≤ 15 lines to the
PM, details in the file); the PM keeps its context small: after the hooks are fixed
(point 2) compaction is safe, so lower the auto-compact threshold to about 300 k if the
harness offers a setting, else end long idle stretches with a compaction.
Guard: `tools/lanes.py` prints the PM context from the transcript at each watchdog wake;
above 400 k it says "compact now". Brief template line: "run every command in the
foreground; never wait on a background task".

### 8. HIGH (efficiency). Three UI rounds per look; fix rounds cost three times the build

Finding: mc v2: build 3.9 M, fix rounds 6.4 + 4.7 M, reviews 2.8 + 1.6 + 0.5 M, code fixes
1.4 M. Tiling: build 5.0 M, fixes 4.0 + 2.2 + 1.2 M, reviews 1.0 + 2.2 + about 0.5 M. Round 1
of tiling found 13 MAJOR, many mechanical (gaps counted wrong, text past its box, contrast).
Round 3 (confirm-only, 0.5 M, 78-80 lines) is the cheap pattern that works.
Evidence: subagent usage; review-tiling-v2.md "MAJOR (13)"; review-mc-v2-r2.md "six empty
texts on top of each other".
Change: before review 1 the builder runs the look's gate with hud-audit and the capture
matrix, and fixes what the audit reports; review 1 judges taste and the look document only;
the fix round is a fresh sonnet agent with the finding list; rounds 2 and 3 are confirm-only
on the fix diff and new crops, ≤ 40 lines. Target: a look for ≤ 10 M units.
Guard: the UI review brief refuses to start (first line of its checklist) without a green
hud-audit log for the commit under review.

### 9. HIGH (honesty, steering). The status page contradicts itself and the log

Finding: header "Updated Wed 11 Nov, 18:25 Bangkok" above entries of Thu 06:30; "Release
candidate: est. Wed 18 Nov" in the header and "about 19 or 20 Nov" under Next; "HUD and helper
chat 13 of 13 ... colours ... are merging now" (merged 11-11; and "a visible entry in every
look" is an open RC requirement); "Next: ... flaky tests, CI and release dry run" while
state.md says "Parked ... flakes, CI dry run"; "Moving windows ... by key, command, drag and
voice" without saying that voice runs on a fake ASR and a keyword stand-in (judge 1, rc 2);
"The full showcase with screencasts of every feature is being recorded now" while the tour
worker builds construct-v2; the rejected xp capture (point 4). "Needs you" misses the cloud
setup question (questions 7), which the user just answered.
Evidence: status/index.html lines 103, 105, 165, 176; state.md "Now".
Change: the header time, "Running", "Next" and "Needs you" are generated by a small script
from state.md (Now, Queue) and questions.md, not edited by hand; each picture states the
build it shows and "reviewed" or "unreviewed".
Guard: the generator runs in the session-log pre-commit when status/ or state.md changes;
`check_log.py` fails when state.md records a merge and status/ has no change in the same
commit.

### 10. MEDIUM (strategy). The Plan of 11-12 throttled work and parked the biggest risk

Finding: "Budget about 93 % ... Light work only until then" (usage.py), against the user's
87 % and "Don't be too paranoid" (11-10 15:20, again 11-12 01:42). It parks CI and flakes,
although design/rc.md (after reflection 1) moved "a draft pull request (CI) and a release
dry run" to 11-12 because CI has not run on this work for 13 days. The weekly limit will
stop everything for about 43 h anyway; whatever is not spent by then is lost.
Evidence: state.md "Plan of 11-12" line 1 and "Parked"; design/rc.md "After the 11-11
reflection".
Change: spend the 13 points now on work that is ready and cheap to steer (point 12), and use
the 43 h stop as the boundary: the resume task fires Saturday with a ready Queue.
Guard: budget rule (point 11).

### 11. MEDIUM (rules). The budget rule has three meters and no plan

Finding: pm-card says "estimate with usage.py (about 2.8 M weighted units per percent)",
decisions-now (01:42) says "budget lines use the user's readings", charter 10 says a daily
`total_cost_usd` cross-check (failed silently, friction 33), and charter 10 "Runs: 40, 35 and
25 % of 14 %" describes three runs that no longer exist. Cloud workers are not counted by
usage.py at all (the tour worker alone: 321 M cache read, $61).
Evidence: pm-card line 20; decisions-now "Budget and models"; charter lines 702-711.
Change: one rule: the meter is the user's latest reading; between readings the PM adds the
rate measured between the last two readings (now 1.8 points per hour) and labels it
"estimate"; the plan states when the limit will stop the week and what the Queue holds at
that moment. No throttle while the week has room; the only cap is the 85 % swarm limit.
Guard: `check_log.py` fails when state.md Now lacks the line `budget: <reading> at <time>,
rate <x>/h, stop about <time>` or when its reading is older than 24 h.

### 12. MEDIUM (strategy). The next 3 days

The order of the week is right: the four looks first, then per-area code review, CI and
packaging, persona round 4, rc 11-18 to 11-20, then the user test round. Two corrections:
the known main-path flaws of merged looks (point 4) come before new look steps (charter 3
"Priority": known defects of merged code first), and CI and packaging start before the stop.

Thu 11-12, until the weekly stop (about 15:00-16:00 Bkk):
1. PM, 30 min, one commit: rewrite state.md (point 2), fix the times and `screened-through`,
   fix `pm_context.py` (Now and Queue, unrecorded first), replace hud-xp.png on the status
   page, rewrite the prompt of "Resume PM after weekly reset" (it names tiling-v2, mc-v2 and
   "xp-v2 waiting": it must say only "read state.md Queue"), disable the old three-runs task
   `trig_012VGJoVzsJ81hFa6AufBc9S` (the hourly watchdog replaces it; its prompt starts a full
   run start with a workspace build), remove the merged worktrees (f9, hud, k5, keys,
   grants2, hud-start, mc-v2, tiling-v2, wobble3d) for disk.
2. Quick test of the updated cloud environment (the user asked): one small worker runs
   `setup.sh` and `cargo build`; if it works, two lanes move to cloud workers.
3. Four lanes until the stop: (a) construct-v2 finish and merge train 3 with claude/pkg;
   (b) the chat tab per look (the white tab on the tiling bar; mc Reading panel), fresh
   sonnet; (c) xp-v2 steps 2, 4, 10, 13 in a fresh sonnet agent; (d) tooling: hud-audit in
   every look's gate, red-first and pre-commit range in merge-gate.sh, a worker gate mode,
   and a local replica of the CI jobs (msrv, docs, pre-commit) so Saturday's draft PR is
   not the first run.
4. Before the stop: the weekly report `reports/week-1.md` (charter 11b, due Friday, the user
   returns Saturday; a sonnet agent from the log files, ≤ 2 M units) and the Saturday briefs
   written into the Queue, so the resume costs no planning.
Fri 11-13: probably stopped. If not: UI pass 2 on the merged looks with the audit.
Sat 11-14 from 11:05 Bkk (reset, Fable, the user back):
- one status tab "Approve the looks" (rc 1b): each look at 1x and 2x, idle and chat open,
  one line per look; the draft CI pull request as the one outward question.
- Fable, up to two lanes: the per-area code review, core and the security paths first (rc 6),
  read-only, then the fixes of HIGH findings; the rc reflection loop (the user wants it on
  Fable).
- sonnet: fix lanes by area for swarm and review findings; haiku swarm 6 on the four v2
  looks while it finds new issues.
Sun 11-15 to Tue 11-17: fix lanes, flakes F10 and F11, three greens in a row, packaging
smoke, persona round 4. rc 11-18, likely 11-19 or 11-20.

The reflection loop the user asked for (01:43 "until no new findings apear"): a second round
is a confirm-only check of the applied diff (≤ 1 M units); stop when a round finds only
LOW points or after three rounds; the deep loop runs on Fable after the reset.

### 13. MEDIUM (rules). Five layers of rules, no single precedence; dead text remains

Finding: rules live in the charter body, "Rules of the 11-11 reflection", "UI reviews",
"Design per look", "Rules of reflection 2", decisions-now.md, pm-card.md, dev-rules.md,
design/rc.md and design/product-briefs.md. The charter was last changed 11-11 11:41; none of
friction 33-38, decisions-now, the hooks, the watchdog or the product briefs is in it.
Contradictions now in force:
- reflection 2 rule 4 and rc.md 1b ("The user decides taste") against the user 17:50 ("Not
  deciding anything now") and 05:22 ("Decide such things on your own").
- charter 3 "Workers ... At most 2 to 3" against charter 10 "four heavy lanes".
- charter 8 "each scheduled run starts two or three short haiku-5-5 personas" against the
  user 16:53 "Only keep testing as new data arrives".
- charter 1.4 "No ... pull requests" and rc.md 4 "which the user must allow" against the user
  10:31 "You may released when all clean".
- "Model use revised (user, 11-11 ~11:00)" ("burn Fable this week") against 06:13 "no more
  fable for this week".
- charter 1.20 foundation gate against the user's order of 10:30 (UX clean, foundation,
  review, rc).
Dead text: 1.13 and 11a (runs at 07:23, 12:23, 17:23), 10 "Runs" shares and the cost
cross-check, 11b per-run reports (none since 11-09), 11c plan-data.md (last change 11-10),
guards.md (last change 11-09; no rule since has a row), 7 "Design (Monday)", 6 sprints and
panels (frozen until the rc), 13 HANDOFF.md (does not exist), routine.md, ledger.md and
tokens.py, 15 starting backlog, the prose merge gate of 3 (the script has 17 steps),
"Disk: each worktree builds into its own target", the `--fast` rule. questions.md Q10 still
plans a Nix flake that the user cut on 11-10.
Change: replace the charter by a rules file of at most 20 numbered rules with guards
(below), a short "user decisions" file (decisions-now.md, the top precedence), and reference
pages that hold no rules (setup, product briefs, rc criteria, look documents). The old
charter stays in git history.
Guard: `check_log.py` fails when the rules file has more than 20 numbered rules, a rule
without a "Guard:" line, or when a commit adds a rule without removing one (the count of
rules may not grow: "one rule in, one rule out").

### 14. MEDIUM (efficiency). Lanes: local disk is the bottleneck, cloud is idle, worktrees pile up

Finding: 15 worktrees exist, 9 of them merged or abandoned (grants2 and hud-start were
already 280 commits behind on 11-11); 9.9 GB free. xp-v2 waited for disk on 11-11. Cloud
workers could not set up since friction 37; the user updated the environment at 01:41 and
asked for a quick test. The construct-v2 lane runs on a screencast worker whose 497 k
context carries the tour history, so the extensive screencast the user asked for at 15:08
(design/rc.md "Screencast and swarm") is not running.
Evidence: `git worktree list`; `df -h`; get_session of session_016iS3kNcWCmF1YP1YaxYmP6.
Change: remove a worktree and its target in the same step as its merge; a new lane gets a
fresh worker, never a reused one with another task's history.
Guard: `lanes.py` lists worktrees whose branch is merged into the session branch and fails
when one is older than 2 h.

### 15. MEDIUM (rules). Reflections get one-line answers, and repeated points do not reach friction

Finding: charter 11a: "The PM then adds an answer under each point". The 11-11 judges have
no answers in their files; answers.md has one line each ("all five top changes adopted").
Stale state.md was raised by the 11-10 judge (finding 9), the 11-11 judge ("Recovery
files") and now (point 2): three days in a row, but no friction entry, against 11a ("A point
raised on two days in a row goes to friction.md").
Evidence: reflections/2026-11-10-judge.md line 69; 2026-11-11-opus-judge.md line 39.
Change and guard: as point 5 (an "Answer:" per point, checked); `check_log.py` also greps
the two previous reflections for the same file names in "Finding" lines and asks for a
friction entry.

### 16. LOW (scope). The product briefs added RC requirements nobody asked for

Finding: the shared RC requirements of design/product-briefs.md (lock, power off, volume,
screenshot and clock in every look) and per-look items (xp F1 sheet, tiling power menu and
screenshot keys) became 19 `feat` commits in a polish week, each new review surface for rc
criterion 6. Some answer swarm findings; the rest is scope the PM set.
Change: mark each RC requirement with its source (user, swarm finding, PM); PM-only ones move
after the rc unless they close a swarm finding.
Guard: manual (guards.md row): the merge message of a UI lane lists the RC requirement ids
it closes.

## Proposed rule set (replaces the charter's rules; one rule in, one rule out)

Precedence: the user's words in decisions-now.md, then these rules, then reference pages.
Each rule names its guard; `check_log.py` and `lanes.py` are the two new scripts.

1. **Truth.** state.md has Now (time from `date -u`), Queue, Blocked; rewritten at every event;
   one log commit per wake. Guard: check_log.py (sections, size, Now after the last merge).
2. **User words.** Every user message goes verbatim into decisions-now.md the same turn, with
   its transcript time; ≤ 9,000 characters. Guard: check_log.py against the transcript;
   the decisions hook lists unrecorded messages first.
3. **Times.** Every written time comes from the clock and names its zone. Guard: check_log.py.
4. **Resume.** The hourly watchdog and the reset task point only to state.md Queue; at a stop
   a resume task exists. Guard: lanes.py checks list_triggers output saved by the watchdog.
5. **Never idle.** While the week has budget, at least two lanes run; the PM never ends a
   turn with a Queue item that needs no user. Guard: lanes.py idle check at each wake.
6. **Lanes.** At most four heavy lanes; one area per lane; builders sonnet, ≤ 3 steps or one
   fix round per agent, ≤ 250 k context; gates in the foreground. Guard: lanes.py.
7. **Merge.** Only through merge-gate.sh: lock, red first, pre-commit over the range, the
   full gate once per train; workers run the worker mode. Guard: the script.
8. **UI.** Before review 1 the look's gate with hud-audit is green; an independent reviewer
   rates every main-path flaw of the look, any lane; ≤ 3 rounds, rounds 2 and 3 confirm-only.
   Guard: merge-gate refuses an open main-path row.
9. **Status page.** Generated header, Running, Next and Needs you; a UI merge ships its
   captures marked reviewed. Guard: generator in pre-commit; check_log.py.
10. **Budget.** The user's reading plus the measured rate; no throttle below the 85 % swarm
    limit; the plan states the expected stop. Guard: check_log.py budget line.
11. **Questions.** Only security policy, licensing and outward acts (PR, tag, publish, posts)
    go to the user, batched in Needs you; taste is the PM's until the user takes it back.
    Guard: manual, guards.md row.
12. **Scope.** Until the rc tag only rc criteria and RC requirements with a source merge.
    Guard: merge message lists the ids (manual check in the train).
13. **Commits.** Stage by name; no attribution; never `--no-verify`; the user's author.
    Guard: existing hooks plus the range check in merge-gate.
14. **Reflection.** Each point gets "Answer:" built (commit), scheduled (date) or rejected
    (reason) within 24 h; a point raised twice goes to friction. Guard: check_log.py.
15. **Rules.** At most 20 rules here, each with a guard; a rule in means a rule out. Guard:
    check_log.py counts.

Out (dead or contradicted, delete from the charter): the three daily runs and their shares;
the cost cross-check; per-run reports (the status page and the weekly report replace them);
plan-data.md; the ledger, tokens.py and routine.md; HANDOFF.md as a running file (write it on
11-30); the 3D design of Monday; sprints, panels and continuous haiku runs (after the rc);
the prose merge gate; "Workers at most 2 to 3"; "each worktree builds into its own target";
the `--fast` rule; reflection 2 rule 4 and rc 1b's "before builders start"; the "burn Fable
this week" paragraph; section 15.

## Answers of the PM (2026-11-12, UTC)

1. Answer: built in part (rules.md 5, 9, 11; watchdog; lanes.py and check_log.py in the tooling lane), scheduled: steering count as the judge's headline metric from the next round. Accepted.
2. Answer: built (state.md rewritten as Now, Queue, Blocked; pm_context.py prints them and lists unrecorded messages first; screened-through set from the transcript; decisions split in two hooks). check_log.py in the tooling lane (it already blocks bad commits).
3. Answer: built as rules.md 6 (sonnet builders, at most 3 steps per agent, fresh fix rounds, 250 k flag in lanes.py). Accepted; the opus HUD builder of 11-11 was a PM choice without a record.
4. Answer: scheduled 11-12: both flaws were fixed after the merge (host series B of 59df16e moved the neutral chat rows below the bar; mc fix round R placed the Reading panel), to be confirmed in the tour-v3 stills; rules.md 8 now rates main-path flaws of any lane. hud-xp.png replaced on the status page today.
5. Answer: scheduled in the tooling lane (merge-gate --worker, red-first, pre-commit range, hud-audit in every look's gate, ci-local.sh); "adopted" now means built and guarded (rules.md 14). The window audit cell is rejected until after the rc (reason: no windows.audit API; the handover matrix covers lost windows).
6. Answer: built: times in decisions-now.md corrected to transcript times; rules.md 3; check_log.py guards it (it caught a new invented time at once).
7. Answer: built in part: one log commit per wake (rules.md 1), gates in the foreground (rules.md 6); lanes.py prints the PM context and says "compact now" above 400 k (tooling lane).
8. Answer: built as rules.md 8 (audit before review 1, rounds 2 and 3 confirm-only); target 10 M units per look recorded here.
9. Answer: scheduled 11-12: the status header, Running, Next and Needs you are rewritten from state.md today; the generator script goes to Saturday's tooling lane (reason: the PM edits it by hand until then, guarded by check_log.py merge-without-status).
10. Answer: built: no throttle (rules.md 10); CI and flakes are in the Queue (ci-local.sh today, CI draft PR Saturday).
11. Answer: built as rules.md 10 with the budget line in state.md Now.
12. Answer: built: the Queue of state.md follows this plan, with the four lanes and the Saturday items.
13. Answer: built: rules.md (15 rules, each with a guard) wins over charter.md, which becomes reference; the pm-card reads rules.md instead of the charter. Deleting the dead charter text is scheduled for Saturday's retro (charter 12).
14. Answer: built: 9 merged worktrees removed today; cloud workers used again (environment verified ready); rules.md 7 removes a worktree with its merge. Accepted that a new lane gets a fresh worker; the tour worker keeps construct-v2 only because it is set up and holds its context, and ends after tour-v3.
15. Answer: built as rules.md 14; this section is the first answer in the file.
16. Answer: accepted: RC requirements in design/product-briefs.md get a source tag on Saturday; PM-only items move after the rc unless they close a finding.
