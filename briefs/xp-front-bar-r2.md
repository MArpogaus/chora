# xp-front-bar r2 (rc 1, 2; rxf-1 MAJOR process): split red test and fix into two commits

Model: haiku, effort medium (git history split, same tree; a test run; no design)
Lane: cloud, claude/xp-front-bar-r2 at d588d573. Cost at most 0.3 %; stop at 100 k context. One area: plugins/chora-xp (no code change beyond 40292419).
Why: reports/xp-front-bar.md "## PM review r1": rxf-1 MAJOR, red test and fix of rcf-11 share commit 40292419 (rules.md 8: a fix has its red test commit first). The content is reviewed and correct.
Step 0 (no setup script): `git fetch origin claude/xp-front-bar`, `git fetch --unshallow`; a failure: send_message the PM "xp-front-bar-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Commit A (red): `git checkout 40292419 -- plugins/chora-xp/src/wm.rs plugins/chora-xp/src/wm/hosted.rs`; commit "test(xp): the taskbar tooltip lies in the front bar (rcf-11)". If the fmt or another hook fails, fix the format (cargo fmt) and commit again, never --no-verify.
2) Show red: `cargo test -p chora-xp <the new hosted test name>` fails on commit A; paste the failing line.
3) Commit B (fix): `git checkout 40292419 -- plugins/chora-xp/src/wm/dispatch.rs`; commit "fix(xp): the taskbar tooltip follows the front face (rcf-11)". The same test passes; paste it.
4) `git diff 40292419 HEAD` is empty (paste "empty"); `tests/merge-gate.sh --red-first-check` passes on d588d573..HEAD; `cargo clippy -p chora-xp --all-targets -- -D warnings` clean. No full gate: the tree equals 40292419, whose full gate was green (63 PASS).
Done when: A red and B green with shas, the empty diff, the red-first check and clippy pasted.
Report: reports/xp-front-bar.md "## r2" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/xp-front-bar-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
