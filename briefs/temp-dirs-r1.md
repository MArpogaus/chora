# temp-dirs r1 (F5): one counted temp-folder helper instead of name+pid folders

Model: sonnet, effort high (routine refactor with red tests, many call sites)
Lane: cloud, branch claude/temp-dirs from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "temp-dirs r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: backlog.md rows F23 ("Test helpers: ~20 temp folders by name+pid") and F24 (taskbar.rs:924 chora-xp-start-<pid> folder clash).
Goal: test code makes temp folders through one helper (name + pid + a process-wide counter, removed on drop), so two folders of one process never clash.
1. Find every test or test-helper folder named only by name and pid: `grep -rnE 'process::id\(\)' --include=*.rs crates plugins services`. List them in the report.
2. Red test commit first: a unit test that makes two folders with the same name in one process and expects two distinct paths, both removed on drop (fails until the helper exists).
3. The helper lives in chora-kit (crates/chora-kit/src/testing.rs, or the crate's existing test-support module if one fits better); crates that cannot depend on chora-kit get the same three lines with a comment naming the kit helper, and a tools/copies.toml row if the copy checker needs one.
4. Move each call site found in 1 to the helper, taskbar.rs:924 included. Do not change what the tests check.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` green. Poll it in the same turn; never end a turn to wait.
Done when: red test first; every site of 1 moved or named with a reason; gate green; reports/temp-dirs.md "## r1" on claude/session-log with the site list, the test and the head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Rules: push claude/temp-dirs and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
