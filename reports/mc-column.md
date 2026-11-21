# mc-column

## r1

Lane claude/mc-column, from a76476be. Head **6dd2c843** (pushed). Commits: red 7fcf8364, fix 6dd2c843.

**Cause: the count was wrong, the product is right** (mc-54, cb-4).

Evidence (`demo/tour/run.py --check mc` with a temporary dump of every `-right-` node and `scene.list`
after the flush "column", after the first and after the second Super+n; the dump is not committed):

| moment | visible texts | with a rect (drawn) | ids |
|---|---|---|---|
| before | 72 | 24 | 24 `mc-r0-right-*` (KEYS, CHAT, doors, state) drawn; 48 `mc-r1-right-*` (ALL KEYS, the list Escape closed) visible:true, rects `{}` |
| after Super+n (hidden) | 24 | 0 | `mc-r0-right-*`, rects `{}` |
| after Super+n again | 24 | 24 | the same 24 `mc-r0-right-*` ids, no `mc-r1` node |

- The 72 stayed 72 for 6 s after the flush, so it is no cross-fade still running; scene.list is `agenda` at all three moments.
- The column comes back in full: same 24 ids, same texts, rects back (e.g. `mc-r0-right-chat-door` at 888,110 204x28).
- `column_count` (demo/tour/chapters/mc.py:64, before the fix) counted `kind == text and visible`. A hidden
  or closed region keeps `visible: true` and has no rect on any output; gate-mc-column.sh already reads
  `visible and (.rects | length) > 0` (tests/gate-mc-column.sh:`shown`). So the tour counted nodes that draw nothing.
- No stale region keeps drawn nodes. The `visible:true` with no rect of a closed list is how mc reports a hidden
  region (not a fault of this lane; not touched).

Red first: the brief asks for the case in tests/gate-mc-column.sh or a chora-mc unit test. Both read
`visible and rects`, so on a76476be they are green (the product is right). The case for the true fault, the count,
is `tests/test_tour.py` `ChaptersB.test_mc_column_counts_the_texts_that_are_drawn` (planted state: 24 drawn, 48
visible without rect; hidden 0; back 24). Red on a76476be 3 of 3 (`AssertionError: 72 != 24`).

Fix (6dd2c843, demo/tour/chapters/mc.py): `column_count` also requires `n.get("rects")`; the row "Super+n brings the
column back" loses its `known`. Green 5 of 5 (`python3 -I tests/test_tour.py`, 25 tests, OK x5).

Tour row: `demo/tour/run.py --check mc` twice: `chapter mc: 22 acts, 19 green, 0 inv, 3 known, 0 red`, 0 red, 0 broken
both times (before: 18 green, 4 known). The 3 known left are the I5 rows of acts 4 to 6, not this lane.

Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,mc-paths,mc-sofa,mc-column,mc-handover,doors,acts`
on 6dd2c843: all 11 PASS (pre-commit 0s, ci-check 88s, build 26s, test 220s, clippy 17s, doors 372s, acts 292s,
mc-paths 120s, mc-sofa 48s, mc-column 55s, mc-handover 11s). gate-lib left out as briefed. "partial run: nothing is pushed" is the gate's line for --only.

Notes: my first commit attempt of the fix failed the commit-message-length hook (subject 74 > 68); I shortened the message
and committed again, no --no-verify. A first gate run started before that commit landed was killed and not counted; the run above is the only one.
