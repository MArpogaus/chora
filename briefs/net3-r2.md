# net3 r2 (F7, F5): fix the review findings of net3 r1

Model: sonnet, effort high (named test-harness fixes with cause and fix given by review r1)
Lane: cloud, branch claude/net3 at bc54896a (continue it). One round, stop at 200 k. Cost about 1 %.
Run `git fetch --unshallow` first. Read reports/net3.md "## Review r1" on claude/session-log.
1. Red first, one commit: a cell per finding that fails on bc54896a where a cell can show it:
   rw-1 a held-45-degree construct act (pause `cube.turn` halfway) with an I13 check that fails when a ring
   neighbour of a shown face is missing or has no corners; rw-2 a moved camera keeps the I7 crop; rw-3 a modal
   above its fullscreen parent is no bar; rw-4 a partly clipped image node keeps its runs unsqueezed; rw-5 a
   second overprint in the tiling key list is not hidden by net3-t1; rw-6 an accepted notice never shown fails I9.
2. Fixes as the review says (rw-1 to rw-6, rw-8 the logging test that fails beside other tests; rw-7 NIT only
   if small). If rw-1 goes red on the head, add co-21 to tests/acts/known-red.toml with the named breach, do not
   fix the construct (that is another lane). No look change, no new dependency.
Done when: the cells red then green; `cargo test -p chora ipc::` 134/134 twice; `cargo test -p chora-scene
-p chora-draw -p chora-xp -p chora-notify`; `python3 tests/test_acts.py`; `tests/merge-gate.sh --only acts,cube`
green (poll in the same turn). No full gate (the train runs it).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append "## r2" to reports/net3.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
