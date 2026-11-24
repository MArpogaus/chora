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

## soak r3 2026-11-23

Head `claude/soak-r3` `0e41f658` (script change; session head `1ec04694`). One core for the whole run, one session: chora-tiling on output HEADLESS-2, chora-mc with `--manage-windows` (restarted every 5th round), confined client box0. `SOAK_MIN=180 SAMPLE_S=600`, started 16:49:01 UTC, DONE 19:49:08 UTC. Rounds: 911. Each round ends every probe of the session with `end_probes` (sl-9 fix). Raw data, not in git: `/tmp/sim-long-r3/samples.csv` (per 600 s: RSS kB and fd count by pid), `/tmp/sim-long-r3/soak.run` (round log), `tests/battle/chora-soak.log` (core log).

Per process, first hour (samples at t <= 3600 s) against last hour (t >= 7207 s); fit is a least-squares slope over all samples.

| process | RSS first hour | RSS last hour | RSS fit per hour | fds first hour | fds last hour | fds fit per hour |
|---|---|---|---|---|---|---|
| chora (core) | 244.7 MB | 306.6 MB | +21.5 MB (about 7 % of the 291 MB sample mean) | 44 | 45 | +0.7 |
| chora-tiling | 13.9 MB | 14.0 MB | +0.1 MB (0.4 %) | 5 | 5 | 0.0 |
| chora-mc | 18.3 MB | 18.5 MB | +0.1 MB (0.5 %) | 5 | 5 | 0.0 |

Core round-end RSS (after every window is closed, the r2 measure): round 1 246 MB, round 911 208 MB, mean 237 MB, max 249 MB; each 100-round block 226 to 234 MB. Core fds at the samples: 30 to 63, the 63 at 6601 s; the r2 run went 32 to 1050.

- **Window count:** 0 probe windows open after `end_probes` in all 911 rounds (`soak: round N: windows open 0 after end_probes`); no "windows left after close" line.
- **Deaths:** none. Core pid 23635 alive to DONE; no "core gone" line; no "did not come back" line for the mc restarts.
- **Watchdog disconnects:** 0 (`-- watchdog lines: 0` at the end; grep for `watchdog|disconnect` in `tests/battle/chora-soak.log`: 0 lines).
- **Findings in the run file:** none.
- **Not in this run:** W10 (chora-xp) and W16 (voice), as in r2.

Verdict against the bar (growth at most 5 %/h with the window count flat):

- **sl-9** (r2 window and fd leak, the soak's own fault): closed. Windows are flat at 0 over 911 rounds and fds are flat (fit +0.7 per hour, 30 at the end). The r2 round-end growth (273 MB to 2.25 GB) does not recur: round-end RSS is 246 MB at round 1 and 208 MB at round 911.
- **sl-10** (new, sampled core RSS): open. The sampled fit is +21.5 MB/h, about 7 %/h, over the bar, with the window count flat. The sampled series is a sawtooth between 212 and 496 MB, so the rise rests on the peaks at 5401, 6601, 7801 and 8401 s. The round-end series does not rise. Whether the sampled peaks grow with the round count is not decided by this run: they recur at the sample instants, which fall at different round phases. A follow-up samples RSS at a fixed phase of the round (after the window close) and keeps the round count.

Chora-tiling and chora-mc are flat on both RSS and fds.

## soak r4 2026-11-23

Head `claude/soak-r4` `47da935a` (script change: settled and peak RSS per round; the report is in the next commit). One core for the whole run, one session: chora-tiling on output HEADLESS-2, chora-mc with `--manage-windows` (restarted every 5th round), confined client box0. `SOAK_MIN=180 SAMPLE_S=600`, started 20:31:16 UTC, DONE 23:31:18 UTC. Rounds: 1008. Raw data, not in git: the run file `soak.run` and `samples.csv` in the session scratchpad (`sim-long-r4/`), the core log `tests/battle/chora-soak.log` (git-ignored).

Per round, the run file has two RSS sets (kB) per process, chora, chora-tiling and chora-mc:

- **settled:** after `end_probes`, read only when no probe window is open (`settled RSS kB:`; `n/a` otherwise). Window count was 0 in all 1008 rounds, so every round has a settled value.
- **peak:** VmHWM of the process, reset at the round start by writing 5 to `/proc/PID/clear_refs` (the kernel sets the peak back to the current RSS), so VmHWM read at the round end is the peak of that round (`peak RSS kB (VmHWM since round start):`). Checked on this kernel: a 216 MB peak reset to the 11 MB current RSS.

Fits are least-squares slopes against the round number, converted to MB per hour at the run average of 336 rounds per hour (1008 rounds in 3 h). First hour = rounds 1 to 336, last hour = rounds 673 to 1008. The per-round time is assumed uniform.

| process | settled first h | settled last h | settled fit per hour | peak first h | peak last h | peak fit per hour |
|---|---|---|---|---|---|---|
| chora (core) | 249.4 MB | 258.0 MB | +4.2 MB (+1.7 % of the first-hour mean) | 479.6 MB | 474.9 MB | -3.6 MB (-0.7 %) |
| chora-tiling | 14.4 MB | 14.4 MB | 0.0 MB (0.0 %) | 14.5 MB | 14.5 MB | 0.0 MB (0.0 %) |
| chora-mc | 18.4 MB | 18.4 MB | 0.0 MB (0.0 %) | 18.4 MB | 18.4 MB | 0.0 MB (0.0 %) |

Other measures:

- **Core settled series** by 100-round block (mean, max): 246 / 310, 255 / 356, 250 / 357, 248 / 311, 249 / 311, 251 / 341, 255 / 327, 257 / 341, 253 / 311, 263 / 379, 260 / 295 MB (the last block is rounds 1001 to 1008). Means stay between 246 and 263 MB. The maxima are single-round excursions of 40 to 130 MB above the mean; they do not climb block by block.
- **Core peak series** by block (mean, max): 485 / 647, 477 / 615, 478 / 617, 479 / 615, 486 / 679, 482 / 639, 476 / 648, 480 / 646, 472 / 618, 469 / 615, 462 / 582 MB. No upward trend; the last block is the lowest mean.
- **Core fds** at the samples: 30 to 56 (the 30 at the start and at the end); fit +2.9 per hour. Flat.
- **Sampled RSS** (the 600 s samples, the r3 measure): core fit +18 MB per hour (first hour 241 MB, last hour 278 MB). The samples land at random round phases, so this fit follows the phase of the sample instants, not the growth of the process: the settled and peak series, measured at fixed phases, do not rise. The sampled measure is kept for the record only, and the verdict below does not use it.
- **Deaths:** none. Core pid 23613 alive to DONE; no `core gone` and no `did not come back` line.
- **Watchdog disconnects:** 0 (`-- watchdog lines: 0`; `grep -ci 'watchdog|disconnect' tests/battle/chora-soak.log`: 0).
- **Panics** in the core log: 0.
- **Findings in the run file:** none.
- **Probe windows:** 0 open after `end_probes` in all 1008 rounds; no `windows left after close` line.
- **Box0 client:** the 200 creates and the 200 removes returned ok in all 1008 rounds; the 1008 `box0 settings.set: refused` lines are the expected refusal (no `TOOK` line).
- **Not in this run:** W10 (chora-xp) and W16 (voice), as in r2 and r3.

Verdict against the bar (growth at most 5 %/h with the window count flat; design/rc.md row 9):

- **sh-36** (sampled core RSS rise: peaks or growth?): **closed.** The settled fit is +1.7 %/h, under the 5 %/h bar; the peak fit is -0.7 %/h, under the bar and flat after the first hour; the window count is 0 in every round. The round-end level is flat, and the rise in the sampled series came from where the sample instants fell in the round, as the r3 note suspected. The sampled measure stays phase-dependent and is not a verdict basis.
