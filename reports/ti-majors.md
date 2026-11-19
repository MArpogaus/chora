# ti-majors

## r1
Branch claude/ti-majors, head c9ed1cfd (a61e8eaa + 2 commits; origin/claude/autonomous-dev-session-3w-i33pg7 merged: already up to date).
Reproduced on a61e8eaa with `BIN=target/debug tests/gate-tiling-look-r2.sh` (ink, paper): all three findings already hold; tour v4 predates the fixes.

- ti-22 Super+Print notice: held for success ("Screenshot saved" with the path, cell notice, capture tiling-print-notice.png in gate-tiling-print-notice.sh) and for failure ("Screenshot failed", capture tiling-screenshot-failed-<theme>.png). Gap: the failure notice had no reason. Red d4c00135 (grim stand-in prints "compositor refused the capture", cell wants it in the notice; run `CELLS=notice THEMES=ink`: "FAIL: the failure names no reason"), fix c9ed1cfd (flavors/shared/screenshot.sh keeps grim stderr as the body, removes the empty file). `tests/screenshot-notice.sh` still ok.
- ti-23 stale tooltip: gone, no code. Cell gate-tiling-look-r2 tooltip: the bar doors have no tooltip (c1adc900); cell green in ink and paper.
- ti-24 power icon and Chat pill: gone, no code. Cells door and power in gate-tiling-look-r2 (power icon at the right end, one bar helper word, no "Chat ..." row over a tile), gate-tiling-power (3 of 3 clicks open the menu): green in ink and paper.

Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,tiling-audit,tiling-look,keys,acts`: pre-commit, build, test (204 s), clippy, ci-check, tiling-audit, keys, acts PASS in one run; tiling-look (543 s) hit my 600 s foreground limit there and PASSED in a separate run on the same head.
Not run: the host's grim outside the nested session.

## PM review r1 (23:50)
Base a61e8eaa, head c9ed1cfd, 2 commits, 7+/5- in flavors/shared/screenshot.sh and tests/gate-tiling-look-r2.sh. PM review (small diff; PM not the builder).
- ti-22: red d4c00135 adds a reason check to cell notice (stub grim prints "compositor refused the capture"); fix c9ed1cfd captures grim stderr and passes it to the notice, removes the empty file. Holds: `fi ||` takes the status of the assignment, which is grim's.
- ti-23, ti-24: gone before (cells tooltip, door, power in gate-tiling-look-r2, gate-tiling-power green on a61e8eaa); ledger was stale from tour v4.
- rw-1 NIT: a multi-line grim error goes into the notice body as is; `| tail -n 1` would keep it short. Backlog.
Verdict: no open MAJOR: MERGE-OK c9ed1cfd, train 79.
