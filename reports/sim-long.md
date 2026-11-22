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
