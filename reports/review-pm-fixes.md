
## r1

Reviewed on a5b77488: (A) dcb77733, (B) 1499cfdb. Verdict: MERGE-OK (no change needed). BLOCK 0, MAJOR 0, MINOR 1.

- Step 0: `cargo check --workspace` green.
- Check 1 (A): all `Focus::holder` calls in chora-mc (main.rs:1101, 1131, 1652, 1705) pass `NAME`; the test at 1705 still covers a turn away and back while the program starts; `cargo test -p chora-mc`: 183 passed.
- Check 2 (B): every hunk of check.py changed by ^1 (LIST_CHORDS, CODES, CLOCK, MANAGER/read_states, asked_for_list/own_left, clock_nodes, scene_state/scene_line, run.save begin, docs) or ^2 (CHILDREN, with_children, CODES) is in the result; the one real conflict (`Run.snapshot`) keeps read_states and with_children, and with_children now takes `snap.states.get(child) or session.state(child)`. `tests/test_acts.py` 159 passed; `tests/persona/test_check.py` 32 OK (no `tests/test_persona*.py` exists).
- Check 3 (B), planted through check.py: a construct with faces xp@A (front) and tiling@B: xp window on A shown, on B not; tiling window not shown; when read_states already put xp in states the front rule still applies; a client that is no face manager is left out. Key list: chord 35 for xp, 53 for tiling count as asked (sheet stays in own_left); 30 for xp and 59 for mc do not (sheet dropped).

Findings:
- rpm-1, MINOR, tests/persona/test_check.py:~120: no selftest covers `with_children` (co-pane rule) on the merge; add a planted case like the one above.
