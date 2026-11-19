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

## Review xp r2 (PM, 15:20 UTC; tests only, PM not the builder)
Diff 0113008b..bf0b66eb: wm.rs +3 (cfg(test) mod), wm/tests_moved.rs +1032; production code unchanged. 49 tests.
- mt-1 MINOR: `a_node_asks_xp_for_an_update` (tests_moved.rs:211) asserts nothing; it only checks no error. Assert the reply or the recorded methods.
- mt-2 MINOR: `a_missing_shared_script_is_told_in_a_balloon` (:761) asserts no balloon; the name claims one.
- mt-3 MINOR: `the_desktop_fades_while_a_dialog_is_open` (:773) asserts only under `if wm.fade.is_some()`; vacuous when no fade starts. Assert the fade first.
Verdict (computed): BLOCK 0, MAJOR 0, MINOR 3: MERGE-OK; mt-1..3 go to the moved-tests rest lane. Merges after integ (based on integ 0113008b).

## moved-tests-ct r1

Lane: moved-tests-ct (F5, rank 5 of reports/audit-f.md), branch claude/moved-tests-ct from af5547c7
(integ head), commits b25a52f7 (tiling) and c4c4f5d5 (construct). Tests in new files only: tiling
src/methods/tests_moved.rs (7) and src/overlay/tests_moved.rs (10), construct src/cube/tests_moved.rs
(9); one `#[cfg(test)] mod tests_moved;` line in each module file. Production code unchanged.

Line coverage of the modules added since fe8ba6d0 (`env -u XDG_RUNTIME_DIR cargo llvm-cov -p
chora-construct -p chora-tiling --features chora-kit/testing --summary-only`; before = af5547c7,
after = c4c4f5d5):

| module | before | after | at 70 %? |
|---|---|---|---|
| construct cube/construct.rs | - | - | not at HEAD: no such file (cube.rs is modified since fe8ba6d0, not added); re-measure after the cube split |
| construct cube/hand.rs | 33.3 | 96.7 | yes |
| construct cube/role.rs | 71.8 | 74.4 | yes |
| construct cube/start.rs | 46.6 | 92.2 | yes |
| construct lifecycle.rs | 95.5 | 95.5 | yes |
| construct notice.rs | 100.0 | 100.0 | yes |
| construct state.rs | 97.8 | 97.8 | yes |
| tiling commands.rs | 87.6 | 87.6 | yes |
| tiling drag.rs | 84.5 | 84.5 | yes |
| tiling lifecycle.rs | 95.9 | 95.9 | yes |
| tiling methods.rs | 0.0 | 100.0 | yes |
| tiling overlay.rs | 30.9 | 74.7 | yes |
| tiling state.rs | 100.0 | 100.0 | yes |
| tiling windows.rs | 84.0 | 84.0 | yes |

Not reached in this round: overlay.rs apply_theme (writes the theme files of the user and signals
waybar or runs systemctl --user) and the answer branch of follow_system (it applies a theme), both
left out on purpose; the waybar branch of hide_bar_for_fullscreen needs a panel process.

Tests: `cargo test -p chora-tiling` 176 passed; `cargo test -p chora-construct` 202 passed.
`cargo clippy -p chora-tiling -p chora-construct --all-targets`: 0 warnings.
merge-gate `tests/merge-gate.sh --jobs 3` on c4c4f5d5: GATE-EXIT=0, 41 of 41 steps PASS (log in the
session scratchpad). The first run, on 75387408, failed in ci-check (gate-0.8-plugin-runtime): three
construct tests named the raw method `output.list` outside an item that allows
clippy::disallowed_methods. They now carry that allow, the commit was amended, the gate was green.
Commits carry the author Marcel Arpogaus and no attribution lines (the commit hook refuses them).
