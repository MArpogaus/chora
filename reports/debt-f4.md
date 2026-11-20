# debt-f4 (F4: workarounds 13 > RC 9, hygiene 16 > RC 14)

## r1

Code: claude/debt-f4 at fc404a90 (from 79c8d2b5; the session branch had nothing newer). Step 0 green (`cargo check`,
`cargo build --workspace`). Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,gate-lib,build,test,clippy,layout,hud,construct-extra,mc-handover`
all PASS (run /tmp/merge-gate.vtfJLv; pre-commit 3 s, gate-lib 70, build 23, test 192, clippy 16, hud 79, ci-check 95,
mc-handover 10, construct-extra 36, layout 183). After the last commit (merge-gate.sh): pre-commit, gate-lib, ci-check PASS again;
construct-extra PASS 3 of 3 (36, 36, 37 s). The step `construct-hud` was not in the brief's list; its script changed only in a header line.

| row | state | proof |
|-----|-------|-------|
| B4 | open, narrowed: 3 of 6 in the gate | mcpane, nested 3/3 green and K1 fixed: cell `construct-extra` (b0a5aed6, 2bbbf4ee). k2a, 3d, windows stay out: findings co-27, co-28 |
| N5 | open, now a finding | co-29: without the pointer move Tab opens no list at 1280x720@1.5 (2 of 2 red; with it green). The workaround stays, its header says why (2bbbf4ee) |
| H8 | closed | 8c28bb04 (11-19) already holds the exact text; normalised text of README "Use of LLM Coding Tools" equals the quote in git-conventions "Rules for agents". The debt-r report was older than it |
| H10 | open, narrowed | the step back of the pull gate asserts (938187b4, cell pull green); the margin is not derivable: see below |
| H11 | closed | 037d1282 (df under 6 GB, drift warning), fc404a90 (shallow clone, R18); red first, see below; cell gate-lib PASS |
| H12 | closed | d0dc121e on this branch: dev-rules.md "Decision records" (debt-down r1). The brief's path docs/dev-rules.md does not exist; the file is dev-rules.md at the root of claude/session-log. Not added twice |

### B4 runs (3 each, alone, on 79c8d2b5, debug build)

| script | runs | result |
|--------|------|--------|
| gate-construct-mcpane | 3 | 3/3 green (8-9 s) |
| gate-construct-nested | 3 | 3/3 green (14 s) |
| gate-construct-k1 | 3 | 3/3 red, line 73 "the output differs from the window alone: 9512 206" |
| gate-construct-k2a | 3 | 3/3 red, "the drag flew into a pane" |
| gate-construct-3d | 3 + 1 | red, "the frames of a recording show the walk part of its way", "inject ... position []" |
| gate-construct-windows | 3 + 1 | red, "own: the panes are " |

K1 and the first line of k2a were stale checks, not faults, and are fixed in 2bbbf4ee: (1) the 9512 pixels are the `Alt Esc Step back` hint
(hints.rs FLOWN 3 s), now the compare waits until the hint fades; (2) both scripts expected opacity 0.0 of a pane surface after the fly-out,
but the pane stays drawn: K1 now waits for the focus of construct-look and for the pose to come back, k2a checks the focus. K1 then ran 3/3
green alone and 3/3 in the cell. k2a's next line fails on a real fault, the same one as windows: an overlay `construct.load` makes no pane of
its own program (finding co-27; flavor start is green). No script tests a removed feature, so none was deleted. Known-red lines are in the
headers of the three scripts and in the merge-gate.sh header. Findings co-27 (MAJOR), co-28 (MINOR, not diagnosed), co-29 (MAJOR) are in
findings/construct.md.

### N5

`SIZES=1280x720@1.5 tests/gate-construct-hud.sh` without the `pointer_move [4, 300]` before the last Tab: "FAIL: light-1280x720@1.5: Tab
opened no list" twice; the unchanged script: green. So the workaround hides a Tab fault and is kept.

### H10

Ten runs alone of tests/gate-mc-handover.sh (WITHIN_MS=5000, all green): give 224-412 ms, key 388-434, drag-dropped 401-456, drag-held 1-2.
Pooled over the three waiting cells (30 samples) p95 = 448 ms; 2 x p95 = 896 ms is above the cap of 500, so the limit cannot go to 500 with a
margin. The file's limit is already 820 ms (not 1000 as the debt report read), which stays; the header records the measure (25345676). The
cells take about 400 ms alone, probably the polling and the capture of the harness: a lane that wants a limit under 500 must first cut that.

### H11 red first

Before the change, `tests/merge-gate-selftest.sh` failed 6 checks (df 4, drift 2; output in the lane scratchpad); after: ok. Later 3 more for
the shallow clone, red, then ok. The checks: 5 GB free on / exits 1 and prints df and "only 5 GB free on /, 6 GB are needed"; exactly 6 GB
passes; --list is not checked; a local session branch whose head differs from origin prints "WARNING: drift: ..." and the gate goes on; the
gate fetches that branch first. The drift check is silent when the branch does not exist in the container (a lane).

### Counts (rows of reports/debt-2026-11-20.md sections 2 and 6, a thing without proof counts as open)

| category | RC | before | now | at or below RC |
|----------|----|--------|-----|----------------|
| workarounds | 9 | 13 | 13 | no (+4) |
| hygiene | 14 | 16 | 13 | yes |

Workarounds: open B2, B3, B4 (narrowed), B6, W7, W8, W9 = 7; new N1-N6 = 6 (N5 is now a finding with a run). No row closed, since B4
and N5 are findings. Hygiene: open H2, H3 (narrowed), H4, H5, H6, H7, H9, H10 (narrowed), H13, H14 = 10; new 3 = 13; closed H1, H8, H11,
H12.

### Open and not done

Voice scripts (H4) and the other findings belong to other lanes. Refactor proposals: gate helpers (`foot_pane`, `until_ok` loops that wait
for a pose) into gate-lib.sh (S, tests/gate-construct-*.sh); the construct gates K1 and k2a poll for fixed hints and animation times: a
`wait_pose` helper would remove 4 sleeps (S).

## PM review r1 (2026-11-20, PM is not the builder)
Range: 79c8d2b5..fc404a90, 9 files +121 -17, tests only (merge-gate.sh, selftest, construct and mc gates).
Checks: free-space (6 GB, D90), shallow (R18) and drift (warn only) checks run before the lock, with selftest cases; construct-extra chains K1, mcpane, nested (3/3 per report); k2a, 3d, windows stay out with header lines naming co-27, co-28; the K1 fixes replace stale opacity checks by the focus check; pull asserts the stand-back before reading the plane; mc-handover keeps 820 ms with the measure in the header.
- rv-1 NIT: warn_on_drift fetches origin of the session branch inside a train; harmless (it only moves a remote ref), but the gate now needs the network for that line; it times out at 30 s and goes on.
- rv-2 NOTE: construct-extra adds three gates to every train; its run time goes into the next train's log for the --jobs 2 budget.
New findings: co-27 MAJOR, co-29 MAJOR (construct): rule 6 blocks construct MINOR work until they close; lane co-majors r1.
Verdict (computed): no open MAJOR in the diff: MERGE-OK fc404a90. Harness branch: own train (t84) after t83.
