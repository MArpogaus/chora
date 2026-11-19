# moved-tests-xp r1 (F5): unit tests for the xp wm modules added since the RC

Model: haiku, effort high (tests of existing behaviour)
Lane: cloud, branch claude/moved-tests-xp from claude/integ 3619ccb3. One round, stop at 250 k. Cost about 1.25 %.
Step 0: setup (below), then `git fetch --unshallow` and `cargo check -p chora`; a failure: send_message the PM "moved-tests-xp r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/audit-f.md on claude/session-log (section 1 F5, section 2 rank 5, section 5).
Known: integ is red on two xp tests (`wm::state` a_give_shuts_the_start_menu, a_give_with_the_drag_shuts_the_start_menu); lane
integ fixes them, do not touch them; skip them locally with `--skip`. Lane focus-port changes xp wm/windows.rs and wm/dispatch.rs:
add tests in new files only (e.g. `wm/panels/tests.rs` or `wm/tests_press.rs`), never edit production code.
1. Measure: `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-xp -p chora-kit --features chora-kit/testing --summary-only`; list each xp
   module added since fe8ba6d0 (`git diff --name-only --diff-filter=A fe8ba6d0 HEAD -- plugins/chora-xp`) with its line %.
2. Write unit tests (FakeCore of chora-kit testing where the core is needed) for wm/{panels,press,pointer,keys,dispatch,choices,windows}
   and the other new modules until each is at 70 % lines or more, or list why not (drawing reached only by gates).
Done when: the table of step 1 before and after in the report; each module at 70 % or listed; `cargo test -p chora-xp` green except
the two known tests; `cargo clippy -p chora-xp --all-targets` clean; `tests/merge-gate.sh --jobs 3` once after merging
origin/claude/integ (poll it in the same turn); red-first does not apply (tests of existing code).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and write reports/moved-tests.md
"## xp r1" on claude/session-log. Poll every command in the same turn; never end a turn to wait.
Rules: push claude/moved-tests-xp and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
