# Results of the 0.6 review loop and battle test (2026-10-19/20)

Review: `git diff v0.5.4..HEAD`, seven round reports in
`target/review-loop/round-*.md`, each audited (session JSONL: diff calls,
no repo mutations). Two consecutive CLEAN rounds closed the phase.

Fixes that came out of it (all on `dev`, nothing pushed):

- `2c2125e` fix(transaction): re-check the remove target at the end of a wait
- `683145f` fix(headless): cap the live virtual outputs at 16
- `6d492ab` fix(protocols): drop early surface requests beyond 64
- `77db138` fix(choractl): let clients up go on past a name in its grace time
- `1dbc003` test(probe): ask for server-side decorations with --server-deco

Battle: four passes of W1-W8; pass 3 and 4 were clean (THE END per TODO).
One finding: "tag moves landed on 3 windows" — proven a harness race by a
unit test over the tiling core (fix.log), the workflow tolerance is >= 3.
Everything else green, including 40 windows through 108 layout changes,
50 hotplug rounds and 20 plugin restarts, snake at 16.4 tick/s, the
17th-connection refusal, the watchdog as the de-facto node limit at
~14.6k nodes, and every sandbox escape attempt refused.

Design friction with API/doc consequences: see [FRICTION.md](FRICTION.md).

# 0.7 battle re-run (2026-10-21 .. 2026-10-23)

Same W1-W8 suite, rebuilt on the 0.7 primitives (`choractl wait`,
`input.inject` dry run and `hold`, compact `spawn`). Driver
`tests/battle/battle.sh` under the `local-pi-session` heartbeat.

Two consecutive passes with no new distinct finding closed the phase.
W5 reproduced its stress finding in the first of those passes; the
finding title was already known and judged, so the pass still counted
clean under the distinct-title judge (see FRICTION backlog).

- harness: `b6f00af` (wait the WM handoff after drags, first-block
  parse), `79f74b6` (fix agents got starved by stdin hijack)
- product: `a55a764` fix(ipc) - a refused client reads its error line
- open, by judgement not by time: silent core death after the budgets
  probe under load, W7 tiling silent after SIGUSR1, watchdog
  tolerance against stress clients -> FRICTION.md

# prep-0.8 validation (2026-10-24)

Branch `prep-0.8` = `dev` + `b3a6d84` (WM plugins reconnect after the
emergency reset). Full battle suite: two clean passes, THE END at
02:56 UTC. W7 (SIGUSR1) passes without a finding for the first time;
the W5 stress finding did not appear at all on this branch.

# 0.8 agent read model (2026-10-24)

Plan `plans/0.8.md` on `dev`, nine tasks done, three open (see
`plans/0.8-report.md`). Battle suite after the last task: two clean
passes with zero new finding titles, THE END at 14:53 UTC. The open
0.7 findings (silent core death, snake stress) appeared nowhere; the
owner budget of D57 (`2e4934a`) is the likely reason. Ten more W5
iterations ran inside plan task T11, also clean.

## sim-long 2026-11-22

Plan slice 23 (SIM-LONG): W9, W10, W11, W12 and W16 one after another,
detached, on `7a6c0eb8` (build green: `cargo check` and `cargo build
--workspace`). Run 06:53:47Z to 09:58:47Z: **3 h 05 min**. Each pass
starts a fresh core, so no core ran longer than a pass (35 s to 3 min).
RSS and fds of the core and its plugins were sampled every 10 min
(44 rows; the raw csv stays out of git).

Passes per workflow:

| workflow | passes | rc 0 | rc 1 (core death) |
|---|---|---|---|
| W9 tiling | 18 | 18 | 0 |
| W10 xp | 18 | 18 | 0 |
| W11 mc | 18 | 18 | 0 |
| W12 cross | 18 | 0 | 18 |
| W16 voice | 17 | 17 | 0 |

W16 pass 18 did not start: the 3 h 05 min deadline came first.

RSS per session, linear fit of start and end values over the hours of the
run (`%/h` = slope relative to the mean; `in-session` = mean of
(end - start) / start, per session):

| workflow | process | sessions | start kB first -> last | start %/h | end kB first -> last | end %/h | in-session |
|---|---|---|---|---|---|---|---|
| W9 | chora | 18 | 161876 -> 154444 | -0.5 | 529496 -> 517204 | +0.9 | +236 % |
| W9 | chora-tiling | 18 | 11520 -> 11536 | -0.1 | 12628 -> 12708 | +0.0 | +9 % |
| W10 | chora | 18 | 147768 -> 144664 | -0.2 | 199308 -> 191496 | -0.1 | +31 % |
| W10 | chora-xp | 18 | 27096 -> 27224 | +0.0 | 39340 -> 23308 | +9.4 (see sl-6) | +19 % |
| W11 | chora | 18 | 144760 -> 138720 | -0.4 | 243728 -> 242224 | +0.1 | +73 % |
| W11 | chora-mc | 18 | 15012 -> 15136 | -0.0 | 31332 -> 31300 | -0.4 | +122 % |

W12 has no end values (the core dies before its end). W16 logs no RSS.

fds: one sample per process per tick, so no per-process fit is possible
(each pid appears once). Values seen: chora 25 (W9), 26 (W16), 37 (W11),
54 (W12, one sample); the plugins 5.

Deaths: **W12, 18 of 18 passes**: the core is aborted by its own hang
watchdog (SIGABRT). No other death in the run.

Stalls (a step over its bound, or the core watchdog acting):

- W12, 18 of 18: the 5000-nested-groups step took 15.0 s (its client
  timeout is 10 s); the 10000-creates transaction timed out.
- W10, 16 of 18: the core watchdog warns "frames are slow; no client
  alone reaches the owner budget; the watchdog drops all effects".

Findings:

- **sl-1** (W9, growth): chora RSS rises within each session, mean
  +236 % (154 MB to about 520 MB), in 18 of 18 sessions. The start RSS
  is flat over the run (-0.5 %/h), so the growth is not carried from one
  session to the next. A per-hour rate cannot be measured with sessions
  of 35 s. Log: `run-w9-pN.log`, "FINDING: w9: chora RSS grew from ...".
- **sl-2** (W11, growth): chora +73 % (139 MB to 243 MB) and chora-mc
  +122 % (15 MB to 31 MB) within each session, 18 of 18. Log:
  `run-w11-pN.log`, "FINDING: w11: chora RSS grew from ...".
- **sl-3** (W12, death): the core dies in 18 of 18 passes. Sequence in
  the core log (`shots-w12-pN/chora-w12.log`; the repo copy
  `tests/battle/chora-w12.log` is overwritten per run): the 5000-nested-
  groups step takes 15.0 s; the 10000-creates transaction times out; a
  1 MiB line breaks the pipe; the next connect is refused. The core log
  shows "the main loop stalls ... (stalled 3 s)", then "the main loop
  hangs; the watchdog aborts chora with a core dump (stalled 30 s)", then
  SIGABRT. The cause is not pinned by these logs. The script also reports
  "a confined client got through a refusal" in 16 of 18 passes.
- **sl-4** (W16, breaks): nine voice-path findings in 17 of 17 passes:
  no restart of the tool process in the log; a tool call after the
  reload did not use the new `chora_mcp`; Alt+y and the key yes did not
  answer or change the setting (`a security setting waits for the user`);
  push_to_talk speech without the key reached the ASR; the turn did not
  end; the words said with the key held did not reach the LLM. No stall
  or death. Whether the fakes or the flow cause it is not decided here.
- **sl-5** (W10, stall): the core watchdog fires in 16 of 18 passes:
  "frames are slow; no client alone reaches the owner budget; the
  watchdog drops all effects load=197" (e.g. `shots-w10-p10/chora-w10.log`).
  No step timed out in those passes.
- **sl-6** (W10, growth above 5 %/h by the fit): chora-xp end RSS fits
  +9.4 %/h, but its raw end values fall from 39 MB to 23 MB, so this is
  not confirmed growth. Watch it on a longer run.
- **sl-7** (method): sessions last 35 s to 3 min, so a leak that shows
  only over hours inside one core cannot be seen here. Follow-up: one
  core kept alive for 3 h on the same host.
- **sl-8** (environment, noise): every core log has `sh: 0: cannot open
  /waybar/*.sh` (tag.sh, helper.sh, volume.sh): the bar config points at a
  missing directory. Not a core fault.
