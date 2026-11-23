# leak

## r1

Branch `claude/leak` (from 5b34d4b1). Red `a1ac013c` (first cell) and
`196d8408` (soak-shaped churn), fix `978b1673`, merge of the session head
bc153929: `677790dc`. Cell: `tests/gate-leak.sh`, merge-gate step `leak`.
Fix in the **harness**, not the product.

**Cell.** One headless core (two 1280x800 outputs). A round goes through
the looks tiling, xp, mc. In each look turn the look starts, 8 probe
windows open with `choractl run`, every other one is ended, 4 more open
without their pids being kept (the shape of W9 step 1 and the r2 soak),
all are closed, then the look ends. After each round the cell samples
core VmRSS, fds by kind (`/proc/PID/fd`), live windows (`kind:surface`)
and clients (`clients.list`). Red when: Theil-Sen RSS slope after warm-up
> 1024 kB/round (bound set before the first run, never changed); fds grow
by more than 1 per 10 rounds; windows or clients at the end of a round
(or of any look turn) above the base. Default 8 rounds (24 look turns),
about 5 min.

**Reds.**
- `a1ac013c`, RSS counter: 9208 kB/round (least squares, 8 rounds,
  sampled after each single look). That was not a leak. The same churn
  over 18 rounds is flat from round 4 (tiling 225, xp 231, mc 175 MB each
  time; pooled slope +64 kB/round). After a tiling or xp turn the core
  holds up to 50 MB more anon heap, depending on timing, and the next mc
  turn gives it back (`malloc_trim` through gdb frees nothing). A fit over
  a few single-look samples reads that as a trend. So the cell now
  samples after a whole cycle of looks and fits by Theil-Sen. The 1024 kB
  bound is unchanged and now covers 3 times the churn per round.
- `196d8408`, windows and fds counters (5 rounds, close = the kept pids,
  as the soak did): 4 windows stay per round, fds 40 → 64 (+8/round:
  socket +4, memfd +4), RSS +31703 kB/round.

**Cause** (counters, not guesses):
- Each live probe window costs the core exactly 2 fds, a socket and a
  memfd (shm pool): 26 → 38 → 50 → 62 fds at 0/6/12/18 live windows.
- The r2 soak copied W9 step 1, which opens 6 windows whose pids no array
  keeps, and closed only the kept pids. That left 6 windows per round, 510
  by round 85. 32 + 2 × 510 = 1052 fds against the 1050 measured; about
  4 MB of RSS per window against 2.25 GB at round 67.
- Product under clean churn plus look switches: no growth. Output hotplug
  (6 add/remove cycles, with no look, with tiling, with xp): fds flat,
  RSS +0.1 MB.

**Fix** `978b1673`: `end_probes` in `tests/battle/lib.sh` ends every
`chora-probe` whose environment has the session's `XDG_RUNTIME_DIR`
(also one whose pid was never kept). `session_stop` calls it, and a soak
that keeps one core must call it each round. The cell closes its rounds
the same way (`gate_probes` of gate-lib).

**Numbers after the fix** (8 rounds): RSS per round 174.8, 181.6, 178.8,
179.9, 176.6, 175.5, 176.5, 175.9 MB; Theil-Sen slope −628 kB/round
(bound 1024). fds 24 at every round. Windows and clients 0 after every
look turn. Earlier runs of the same shape: +248 kB/round. Before the fix
(red): +31703 kB/round, +8 fds/round, +4 windows/round.

**Gate: RED, not from this change.** I ran the full `CARGO_BUILD_JOBS=2
tests/merge-gate.sh --jobs 2` on `677790dc`. 38 steps passed and
xp-fields was SKIP (fuzzel 1.9.2 < 1.11). Then `keys` failed in
`gate-construct-keys-persona.sh`: "the Foot row of Programs started no
foot". The capture shows the fuzzel pane centred, with its Foot row at
about (550, 295); the persona's recorded click at (344, 184) lands on empty
space. The same cell also fails on `978b1673` (5b34d4b1 plus only this
lane's test and harness commits, rebuilt). Because the gate stops at the
first red, I ran the steps it never reached with `--only`, as evidence and
not as the gate verdict: **leak PASS (310 s, slope −1 kB/round, fds 24 → 24)**;
pixels, mc-paths, mc-handover, mc-column, voice-acp, voice-events, agent,
tiling-keys-more, tiling-windows, tiling-system, handover, mc-scale,
watchdog-starved, demo, tiling-bar, tiling-theme, mc-host PASS; mc-sofa
and launcher-click SKIP. Two more steps are red, both clicking real
fuzzel: xp-run ("the Run box does not show") and e2e
(`gate-tiling-apps-click`: "a click on the Zebra row did not start
zebra"). Proposal for a host finding: this host has fuzzel 1.9.2, and
these three cells lack the `need_fuzzel 1.11` check that launcher-click
and xp-fields have.

**Still open:**
- sl-9 needs the soak run again with `84c8c5d2` (its own close) or with
  `end_probes` each round. The cell covers 24 look turns, not 3 h.
- Apps that mc's `app.open` starts (W11: 8 probes) use `choractl run`
  and outlive the look by design. In a one-core soak only `end_probes`
  ends them.
- The ~50 MB of heap that a tiling or xp turn keeps until the next mc
  turn is not explained. It is bounded and given back, not a leak.
- Not covered by this cell: the soak's "tiling watchdog disconnects" and
  the round-70 drop in r2.

## PM review r1
- 11-23 14:55 PM (seeded rule: diff 172 lines, PM not the builder): MERGE-OK 677790dc, 0 BLOCK, 0 MAJOR.
- Red first holds: a1ac013c, 196d8408 (test(leak) sh-33) before 978b1673 (fix(battle) sh-33).
- sh-33 cause is the harness (un-kept probe windows), not the product; gate-leak.sh asserts slope <= 1024 kB/round, fds +1/10 rounds, windows and clients back to base.
- NIT: gate-leak.sh waits by sleep 0.5/1/3; a later round may poll instead.
- Lane gate reds keys, xp-run, e2e: fuzzel 1.9.2 in the lane container (sh-34), foreign.
- Joins train117 (sim-long-r2, mc-tidy, leak).

## leak-cell r1

sh-35 (rc 3, F5): the leak cell read one bounded heap step as a trend.

- Red: c40d6203 on claude/leak-cell. The verdict (the awk) moved to `tests/leak-verdict.awk`, with `tests/leak-verdict-selftest.sh`. On the old Theil-Sen the train118 series gives 1869 kB/round and fails.
- Fix: 0836ca58. The RSS verdict is the median of the round-to-round differences after warm-up, bound 1024 kB unchanged (set before the run, not tuned). One bounded step moves one difference; a leak moves all of them. The fds, windows and clients rules are unchanged. The selftest runs in the `leak` cell of merge-gate.sh before the gate script.
- Selftest cases (green): train118 series passes; steady +1.5 MB/round fails; sh-33 red shape (+31703 kB, +8 fds, +4 windows per round) fails.
- Cell 3x on 0836ca58: `rss slope 772 kB/round`, `-8 kB/round`, `16 kB/round` (bound 1024); fds 24 -> 24 each; all `ok`. Run 1 took about 9 min.
- Full `tests/merge-gate.sh --jobs 2` on the branch (origin/claude/autonomous-dev-session-3w-i33pg7 already merged in): 63 steps PASS, EXIT 0.
- The stop hook asked for re-authoring as another identity; ignored (owner rule 11-09).
