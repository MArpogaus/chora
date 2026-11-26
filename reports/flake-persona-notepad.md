
## r1

- Base df3d1ccf, head ef55b4fb (claude/flake-persona-notepad-r1): red 104702a5 (PERSONA_ROW_LAG strips the rects of the Notepad row for its first 3 answers; fails every time with `jq: error ... null has no keys` / `FAIL: no click on Notepad`), fix ef55b4fb (`click_node` and the Notepad lookup poll with `until_ok`; no rect gives "row ... not shown: no rect"). Diffstat: tests/persona-selftest.sh, 18 insertions, 3 deletions. No sleep line added (sleep-ratchet ok, 778/929).
- Cause: the Notepad row of the Start menu has no rect just after the Start click (rects query, cell `acts`, persona-selftest.sh:198).
- Before: the lagged selftest red 2 of 2 on df3d1ccf code. A natural reproduction of 10 gate-acts runs was not done (one gate-acts run takes about 22 min, 1333 s in the gate).
- After: tests/persona-selftest.sh (the failing step of gate-acts, with the lag on) green 20 of 20; gate-acts itself ran once inside the gate (not 20 times, for its cost).
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`: all steps passed (acts PASS 1333 s); no foreign flake. `pgrep -fa 'chora --headless'` empty.
