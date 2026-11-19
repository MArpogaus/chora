# wm7a r3 (F3): merge the session branch into wm7a and join the two cube act sets

Model: sonnet, effort high (a named merge of two reviewed parts; no new design)
Lane: cloud, branch claude/wm7a at 066ae80d (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first. Train 61 left wm7a out: it conflicts with net3 (train 59, co-21 cube acts).
1. `git fetch origin claude/autonomous-dev-session-3w-i33pg7 && git merge FETCH_HEAD` (train 61 head or later).
2. Resolve so both hold:
   - tests/acts/harness.py: Snapshot keeps `boxes`, `blocks` (net3) AND `cube` (wm7a); the snapshot fills all three.
   - tests/acts/known-red.toml: every red of both sides (net3-t1, net3-m1, co-21, wm7a-a1, wm7a-a2); none dropped.
   - tests/acts/looks/construct.py: the cube opens once. One sequence: wm7a's hosted cube acts (through `cube.list`,
     MANAGER "xp") AND net3's I13 acts (stand back; held halfway through a 4 s turn; at rest). Keep wm7a's rw-1
     mid-state asserts. Neither side's act or assert dropped; name any act that cannot keep its wording.
3. No other change. A cell that breaks: fix in the merge, name it in the report.
Done when: `cargo test -p chora-construct -p chora-kit --features chora-kit/testing`;
`tests/merge-gate.sh --only acts,cube,pull,construct-hud,xp` green (poll in the same turn). No full gate (the train runs it).
Push claude/wm7a. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines: head sha,
how each file was resolved) and append "## r3" to reports/wm7a.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
