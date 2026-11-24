# sim-long

## r1

Plan slice 23 (SIM-LONG, leaks and stalls over hours). Head of
`claude/sim-long`: `ce451499` (results in `tests/battle/RESULTS.md`,
section "sim-long 2026-11-22"). Build green on `7a6c0eb8`. Gate
`tests/merge-gate.sh --only pre-commit,ci-check`: PASS.

**Run:** 06:53:47Z to 09:58:47Z, **3 h 05 min**. W9, W10, W11, W12 and
W16 one after another, detached, own RUN log and SHOTS dir per pass.
89 workflow passes; W16 pass 18 did not start (the 3 h 05 min deadline).
Each pass starts a fresh core, which lives 35 s to 3 min.

| workflow | passes | rc 0 | rc 1 (core death) |
|---|---|---|---|
| W9 tiling | 18 | 18 | 0 |
| W10 xp | 18 | 18 | 0 |
| W11 mc | 18 | 18 | 0 |
| W12 cross | 18 | 0 | 18 |
| W16 voice | 17 | 17 | 0 |

**RSS growth per session** (chora, the core; per-session, mean of
(end - start) / start; start and end slopes are linear fits over the
hours of the run, in % of the mean per hour):

| workflow | process | start slope %/h | end slope %/h | in-session |
|---|---|---|---|---|
| W9 | chora | -0.5 | +0.9 | +236 % |
| W10 | chora | -0.2 | -0.1 | +31 % |
| W11 | chora | -0.4 | +0.1 | +73 % |
| W9 / W12 | chora-tiling | -0.1 | +0.0 | +9 % |
| W10 | chora-xp | +0.0 | +9.4 (see sl-6) | +19 % |
| W11 | chora-mc | -0.0 | -0.4 | +122 % |

The start RSS is flat over the run for every process. Per-hour growth
cannot be measured for the core across hours (no core lives that long).
The fds sample has one row per process per tick, so it gives no fit.

**Deaths:** W12, 18 of 18 passes: the core is aborted by its own hang
watchdog (SIGABRT). No other death.

**Stalls:** W12, 18 of 18: the 5000-nested-groups step took 15.0 s;
W10, 16 of 18: the core watchdog warns "frames are slow ... the watchdog
drops all effects".

**Findings** (look, cell, log line; details in RESULTS.md):

- **sl-1**, W9, growth: chora RSS rises within each session, mean +236 %
  (154 MB to about 520 MB), 18 of 18. Start RSS flat. Log:
  `run-w9-pN.log`, "FINDING: w9: chora RSS grew from ...".
- **sl-2**, W11, growth: chora +73 % (139 MB to 243 MB) and chora-mc
  +122 % (15 MB to 31 MB) within each session, 18 of 18. Log:
  `run-w11-pN.log`, "FINDING: w11: chora RSS grew from ...".
- **sl-3**, W12, death: the core dies in 18 of 18 passes. Core log
  (`shots-w12-pN/chora-w12.log`): "the main loop stalls ... (stalled
  3 s)", then "the main loop hangs; the watchdog aborts chora with a
  core dump (stalled 30 s)", then SIGABRT. Trigger: the 5000-nested-
  groups step (15.0 s) and the 10000-creates transaction (timed out),
  then a 1 MiB line. Cause not pinned by these logs.
- **sl-4**, W16, breaks: nine voice-path findings in 17 of 17 passes
  (no tool-process restart in the log; reload not used by the tool
  call; Alt+y and the key yes do not answer or change the setting;
  push_to_talk speech without the key reaches the ASR; no final answer).
  No stall or death. Fakes vs flow not decided.
- **sl-5**, W10, stall: "frames are slow; no client alone reaches the
  owner budget; the watchdog drops all effects load=197" in 16 of 18
  passes (`shots-w10-pN/chora-w10.log`). No step timed out there.
- **sl-6**, W10 chora-xp, growth above 5 %/h by the fit (+9.4 %/h of
  end RSS), not confirmed: the raw end values fall from 39 MB to 23 MB.
- **sl-7**, method: sessions last 35 s to 3 min, so a leak that shows only
  over hours inside one core cannot be seen here. Follow-up: one core
  kept alive for 3 h on the same host.
- **sl-8**, environment noise: every core log has `sh: 0: cannot open
  /waybar/*.sh` (tag.sh, helper.sh, volume.sh).

**Next:** sl-3 is a reproducible core death in the confined-agent
scenario (18 of 18). Its trigger and the hang need a fix owner; this
lane does not change product code.

## PM review r1 (10:06 UTC, 7a6c0eb8..ce451499: tests/battle/RESULTS.md +90, no product code)
- Checks: run 3 h 05 min, 89 passes, tables match the per-pass logs named; brief done criteria met (report, findings with log lines, no product change).
- Findings go to the ledger: sl-3 MAJOR (core abort, 18/18, any client can trigger it: core must stay safe); sl-1 MAJOR (W9 core RSS +236 % in under 3 min, 18/18); sl-2 MEDIUM; sl-4 MEDIUM (voice, nine items, fakes vs flow open); sl-5 MEDIUM; sl-6 LOW unconfirmed; sl-7 method LOW (3 h single core next round); sl-8 LOW env noise.
- Verdict on the branch (doc only): MERGE-OK; joins t109. Next: CORE-DIAG r1 (opus high) pins causes of sl-3 and sl-1 with repro and red-test plan; fix round after split-c merges (same core files).

## r2

Head: `claude/sim-long-r2` 84c8c5d2 (on `230ea0e5`; scripts `081a9890` frame-p99, `84c8c5d2` soak-core). Build: `cargo check` and `cargo build --workspace` (debug) green on `230ea0e5`. Not run: the merge of `5b34d4b1` and the gate (PM: skipped). Raw data: `/tmp/sim-long-r2/` (not in git).

**Repros** (r1 method, 5 runs each):

| repro | runs | result | r1 |
|---|---|---|---|
| repro-sl3: 10000 creates in one scene.apply | 5 | 5/5 green, 0.8 to 1.1 s, no watchdog line | red, 15.0 s and core abort 18/18 |
| repro-sl1: RSS after last close vs start (bar +60 MB) | 5 | 5/5 green, start 149 to 152 MB, after last close 159 to 168 MB (+10 to +19 MB) | after last close 243 MB (+89 MB) in the r1 fix lane |
| repro-sl1: peak, 12 windows, cycle 3 | 5 | 206 to 211 MB; anon 94 MB, other 101 MB, shared 7 MB | 376 MB peak, bar 266 MB |

repro-sl3 covers only the 10000-create step; the 1 MiB line and the 5000-nested-groups step from r1 are not in it.

**Full workflows, end RSS of chora:**

| workflow | passes | start | end | core-diag / r1 | in-session growth |
|---|---|---|---|---|---|
| W9 | 3 | 152.6 to 153.2 MB | 240.1, 240.1, 240.3 MB | 288 MB (core-diag) | +57 % |
| W11 | 3 | 136.8 to 142.1 MB | 215.1, 215.7, 220.0 MB | 243 MB (r1) | +52 to +55 % |

**Frame time p99** (`chora-probe live 400`, its frame callback intervals, 3 passes each; source `tests/probe`, `live`): relayout (set_layout cycling over 12 steps) p99 107.6, 109.9, 109.9 ms (mean 24 to 26 ms). Master-width sweep 0.30 to 0.70 by `settings.set` (no pointer drag injected) p99 25.8, 23.1, 28.6 ms (mean 19.3 to 19.6 ms). No baseline in r1 or core-diag; recorded only.

**3 h soak** (sl-7 follow-up): one core for the whole run, one session (chora-tiling on output 2, chora-mc with `--manage-windows`, confined client box0). Rounds of W9/W11/W12-style steps, 85 rounds, 10:10:22 to 13:19 UTC; sampler by pid every 600 s to `/tmp/sim-long-r2/samples.csv`. Not in it: W10 (xp) and W16 (voice), which need their own session. Method flaw, found during the run: the churn step's 6 extra windows were never closed, so 6 windows stayed open per round (6 at round 1, 510 at round 85). The run used the old script; the fix is in `84c8c5d2`.

| process | first sample (0 s) | last sample (11301 s) | linear fit over the run |
|---|---|---|---|
| chora RSS | 173 MB | 1838 MB | +449 MB/h (mean 1178 MB) |
| chora fds | 32 | 1050 | +291 fds/h |
| chora-tiling RSS | 13.5 MB | 27 MB (round 85: 26.8 MB) | not fitted |
| chora-mc RSS | 17.6 MB | 33 MB (round 85: 29.3 MB) | not fitted |

Round-end RSS of chora (after every window is closed in the script; includes the windows still open, see above): round 1 273 MB, round 10 366 MB, round 20 616 MB, round 30 975 MB, round 40 1249 MB, round 50 1533 MB, round 60 2001 MB, round 70 530 MB, round 80 1686 MB, round 85 1838 MB. The round 70 drop is not explained: the core stayed alive and the rise resumed (2.1 GB by round 74).

First hour vs last hour, chora RSS samples (MB): first hour 169, 592, 485, 1096, 1364, 450, 462; last hour 1667, 887, 2493, 1019, 2221, 1795. The series is a sawtooth, not a monotone rise.

**Deaths:** none; the core is alive to DONE. **Stalls:** the core log has 114 "frames are slow" lines. The first: `chora::safety`, 10:10:23 (host busy, took 28 ms). The first watchdog drop: 10:10:57 "frames are slow; no client alone reaches the owner budget; the watchdog drops all effects load=2377". Tiling is disconnected by the watchdog (code 6) at least 11 times, and `chora-mc` drops its effects repeatedly (first at 10:13:02). Also `chora::mesh`, 10:10:34: "layers exceed 67108864 pixels in a frame; some stay out". Client errors on tiling restart: "socket: Broken pipe (os error 32); I come back" (13 times), "Connection reset by peer (os error 104)". Soak wrong findings: `set_layout tile_left refused (round 70)`, `settings.set gap refused (round 72)`; neither is verified.

**Findings:**

- **sl-1** (W9 growth), open. Repro closed (5/5 green, +10 to +19 MB). W9 end 240 MB is below core-diag 288 MB, but in-session growth is +57 % (over the 5 % bar). Look: W9, log `/tmp/sim-long-r2/run-w9-N.run`.
- **sl-2** (W11 growth), open. W11 end 215 to 220 MB is below r1 243 MB, but in-session growth +52 to +55 % and chora-mc +95 %. Look: W11, log `/tmp/sim-long-r2/run-w11-N.run`.
- **sl-3** (W12 core death), closed for the 10000-create step (repro-sl3 5/5 green, 0.8 to 1.1 s, no abort). Open for the 1 MiB line and the nested-groups step, which are not reproduced here.
- **sl-9** (soak, MAJOR, new): core RSS and fds grow over the 3 h soak: linear fit +449 MB/h and +291 fds/h, round-end RSS 273 MB to 2.25 GB by round 67 (over the 5 %/h bar many times over). Look: soak in one core, log `/tmp/sim-long-r2/chora-soak.log` (first error lines above), samples `/tmp/sim-long-r2/samples.csv`. Attribution not settled: the RSS rise tracks the count of open windows, which the soak itself left open (a script fault, now fixed in `84c8c5d2`), and the round 70 drop coincides with tiling being dropped by the watchdog (not verified). The next soak run must use `84c8c5d2`, and then the RSS slope per hour with the windows count fixed is the measurement that closes or confirms sl-9.
- **sl-4..sl-8:** not in this round.
- **frame p99:** recorded only (no bar yet). Relayout p99 of about 108 ms on a debug build under llvmpipe is the number to beat.

## soak r3

Head `claude/soak-r3` `39283655` (script `0e41f658`, results `39283655`; session head `1ec04694`). Build: `cargo check` and `cargo build --workspace` (debug) green on `1ec04694`. Gate: `tests/merge-gate.sh --only pre-commit,ci-check` PASS. Table and verdict in `tests/battle/RESULTS.md` "## soak r3 2026-11-23".

3 h soak, one core, 911 rounds, 16:49:01 to 19:49:08 UTC, probes ended each round (`end_probes`). Raw data, not in git: `/tmp/sim-long-r3/samples.csv`, `/tmp/sim-long-r3/soak.run`, `tests/battle/chora-soak.log`.

- Core RSS, first vs last hour (sample means): 244.7 to 306.6 MB; sampled fit +21.5 MB/h (about 7 % of the mean). Round-end RSS after all closes: 246 MB at round 1, 208 MB at round 911, max 249 MB.
- Core fds: 44 to 45 (fit +0.7/h); 30 at the end, 63 peak.
- chora-tiling and chora-mc: RSS and fds flat (fit 0.1 MB/h and 0.0/h).
- Windows open after `end_probes`: 0 in all 911 rounds.
- Deaths: none. Findings in the run file: none. Watchdog disconnects: 0.

**sl-9** (r2 window and fd leak, MAJOR): closed. Windows and fds flat, round-end RSS flat.

**sl-10** (new, sampled core RSS, severity for the PM to set): open. Sampled fit is over the 5 %/h bar with windows flat. The sampled series is a sawtooth (212 to 496 MB); the rise rests on the peaks at 5401, 6601, 7801 and 8401 s. Next: sample at a fixed round phase after the window close, same round count.

Note: the first commit of this lane (`0e41f658`) is flagged Unverified by a stop hook because of the author identity; kept as Marcel Arpogaus per the brief.

## PM review soak r3

Diff 1ec04694..39283655: 30 lines (soak-core.sh end_probes plus a per-round window count; RESULTS.md). Test-only, no product fix, so no red commit due. Check: the added lines run after the kill loop and log `windows open N after end_probes` per round; the run file shows 0 in 911 rounds (report). sl-9 / sh-33 closed on round-end RSS and fds flat. sl-10 = sh-36 MINOR: the rise rests on sampled peaks while round-end RSS falls; not a leak by the round-end measure. Verdict MERGE-OK 39283655, train120.

## soak r4

Head `claude/soak-r4` `f0c45fcf` (script `47da935a`: settled and peak core RSS per round; results `f0c45fcf`). One core kept alive for 3 h (`SOAK_MIN=180`), 1008 rounds, 20:31:16 to 23:31:18 UTC. Merge gate `--only pre-commit,ci-check` on `f0c45fcf`: PASS.

Measure: each round logs the settled RSS (after `end_probes`, 0 windows open) and the round's peak (VmHWM, reset at the round start through `clear_refs`) for chora, chora-tiling and chora-mc, at fixed round phases. The sampled 600 s series is kept but is phase-dependent.

| process | settled first h | settled last h | settled fit /h | peak first h | peak last h | peak fit /h |
|---|---|---|---|---|---|---|
| chora | 249.4 MB | 258.0 MB | +1.7 % | 479.6 MB | 474.9 MB | -0.7 % |
| chora-tiling | 14.4 MB | 14.4 MB | 0.0 % | 14.5 MB | 14.5 MB | 0.0 % |
| chora-mc | 18.4 MB | 18.4 MB | 0.0 % | 18.4 MB | 18.4 MB | 0.0 % |

- Core fds: 30 to 56, fit +2.9 per hour (flat). Window count after `end_probes`: 0 in all 1008 rounds.
- Deaths: none. Watchdog disconnects: 0. Panics in the core log: 0. Findings in the run file: none.
- Box0 creates and removes: ok in all 1008 rounds; `settings.set` refused in all 1008.
- Raw data, not in git: `soak.run`, `samples.csv` (session scratchpad, `sim-long-r4/`); core log `tests/battle/chora-soak.log` (git-ignored).

**sh-36** (sampled core RSS rise, peaks or growth): **closed**, under the bar on both measures. Settled +1.7 %/h and peak -0.7 %/h against the 5 %/h bar, window count flat. The sampled series still fits +18 MB/h, but its samples land at random round phases, so that fit follows the phase of the sample instants; it is not the measure of growth.

Caveats: the per-hour fits assume uniform round time (1008 rounds at 336 per hour). The core settled block means stay 246 to 263 MB; single-round excursions of 40 to 130 MB above the mean do not climb block by block.

Note: a stop hook flags commit `47da935` (and `0e41f658` in r3) as Unverified because the author is Marcel Arpogaus. Kept as the brief requires.

## PM review soak r4 (2026-11-24 00:08 UTC)
39283655..f0c45fcf, 2 files, +73 -2, test script and RESULTS only (no product code): PM review under 200 lines.
Checks: per-round settled RSS only at 0 windows (round_rss), peak via VmHWM reset with clear_refs (refusal logged, not silent); 1008 rounds, 0 windows left, 0 deaths, 0 panics; core settled +1.7 %/h, peak -0.7 %/h, both under the 5 %/h bar.
Findings: none. sh-36 CLOSED. Verdict (computed): MERGE-OK f0c45fcf, joins train123.
