# p2-harness-l

## r1

Lane: cloud, claude/p2-harness-l, head 1bf67e98 (the session head was already merged, nothing to merge). Step 0 green (unshallow, cargo check).

| Item | State | Red sha | Fix sha | Selftest / proof |
|---|---|---|---|---|
| 1 L1 | closed on the head | - | 82d16ac8 | `test_no_window_closed_is_red_and_never_says_gone` (test_check.py); the contract is `verify_closed` against the begin plus the toplevels seen in the run; the text "1 windows gone" no longer exists in check.py. Not a product fault: Kemal's close only by `exit` (Super+q nothing) stays the known ti-28 row |
| 2 L2 | closed on the head | - | 82d16ac8 | `test_a_missing_config_is_not_measured`, `test_an_empty_config_is_not_measured`: check.py prints "task L2 not measured (no config)" (end(), has_config), never pass, never red by the empty config. A begin with the real default config is not done: the persona session has none to give |
| 3 L3 | closed on the head | - | 82d16ac8 | `test_a_holder_that_holds_a_given_window_does_not_count`, `test_only_the_given_window_holds_the_focus`, `test_a_holder_of_a_window_that_was_not_given_still_counts`: `verify_focus_kept` reads the end state and skips holders of given-away windows |
| 4 ph-1 | fixed | e97b433a, 0b1a6eb9 | 1bf67e98 | `WaitForApp` x3 (test_check.py) and the `check.py wait-app` line in tests/persona-selftest.sh; round.sh probe waits up to 20 s for the chora-probe window in the scene and fails with "the probe window did not show within 20 s" |
| 5 ph-2 | closed on the head | - | 9c3434c1, 13552608 | `read_states` (check.py) reads the xp child in a construct session; `ManagerChild` tests (`test_the_state_of_the_xp_child_is_read_in_a_construct_session`) |
| 6 RPM-1 | fixed (test only, the code existed) | - | 13071a4c | `WithChildren` x5 in test_check.py (front face shown, other face not, hidden stays hidden, non-manager not read, other look reads none) |

Product findings: none new.

Checks: `tests/persona-selftest.sh` green (with the wait-app line), `python3 -m pytest tests/persona -q` 40 passed, `pre-commit run --files` on the four changed files clean. Note: the red-first hook counts only tests/test_*.py, tests/*-selftest.sh and gate-*.sh as tests, so tests/persona/test_check.py alone does not satisfy it; the ph-1 red commit touches persona-selftest.sh for that.

Gate: `tests/merge-gate.sh --jobs 1` (no --only) on the branch (merged with origin/claude/autonomous-dev-session-3w-i33pg7, already up to date): all steps passed, 49 PASS cells, exit 0, nothing pushed by the gate.
