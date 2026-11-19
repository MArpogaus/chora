# Moved tests

## xp r2

Lane: moved-tests-xp (F5, rank 5 of reports/audit-f.md), branch claude/moved-tests-xp,
commit bf0b66eb on integ 0113008b. Tests: plugins/chora-xp/src/wm/tests_moved.rs (new,
on the fake core of wm.rs, 49 tests); production code unchanged; one `#[cfg(test)] mod
tests_moved;` line in wm.rs.

Line coverage of the xp modules added since fe8ba6d0 (`env -u XDG_RUNTIME_DIR cargo llvm-cov
-p chora-xp --features chora-kit/testing`; before = integ 3619ccb3, after = bf0b66eb's base
before the merge of integ 0113008b):

| module | before | after | at 70 %? |
|---|---|---|---|
| choices | 42.1 | 91.4 | yes |
| dispatch | 37.6 | 71.9 | yes |
| hosted | 70.9 | 73.3 | yes |
| keys | 24.9 | 79.8 | yes |
| lifecycle | 95.9 | 95.9 | yes |
| output | 69.2 | 99.1 | yes |
| panels | 10.3 | 40.7 | **no** |
| pointer | 23.5 | 60.4 | **no** |
| press | 17.0 | 71.9 | yes |
| startup | 89.7 | 89.7 | yes |
| state | 90.2 | 90.2 | yes |
| surfaces | 68.1 | 89.4 | yes |
| windows | 63.1 | 74.6 | yes |

Not at 70 %, and why this round stopped there:
- panels.rs: the dialog clicks (Display Properties list and apply, Effects, Date, Help topics,
  release_panel, tray and icon paths) need the node geometry of each dialog page; not reached
  in this round. Not gate-only.
- pointer.rs: pull_out (cube faces, virtual outputs with an owner), press_tray, press_icon,
  the drop on a far output and the launch paths (start_program spawns processes, left out).

Tests: `cargo test -p chora-xp` 247 passed on bf0b66eb with the merged integ; the two known
tests (wm::state a_give_shuts_the_start_menu, a_give_with_the_drag_shuts_the_start_menu)
pass there too. `cargo clippy -p chora-xp --all-targets`: 0 warnings. `cargo fmt --check`: clean.
merge-gate: see the line below.

merge-gate `tests/merge-gate.sh --jobs 3` on bf0b66eb: GATE-EXIT=0, 40 of 40 steps PASS (log: /tmp/merge-gate.mmWJGp/).

Not pushed with the author the hook asked for: bf0b66e is authored by Marcel Arpogaus as the brief requires; GitHub shows it Unverified (no signature, not a noreply@anthropic.com committer).
