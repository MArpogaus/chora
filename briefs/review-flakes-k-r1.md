# review flakes-k r1 (F5 flakes: AG1, IN1, TR1, HUD1, doors, SIG): seeded review of claude/flakes-k before its merge

Model: opus, effort high, because the diff changes core crates (crates/chora images.rs, inject.rs, ipc.rs, testing.rs) and a flake fix can hide a real race (rules.md 7: the builder never reviews itself)
Lane: cloud, read-only on code: claude/flakes-k head e2508ec1, base 79c8d2b5 (11 files, +216 -79). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran pre-commit, gate-lib, build, test, clippy, ci-check, hud, doors green).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/flakes-k`; `cargo check --workspace`; a failure: send_message the PM "review flakes-k BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/flakes-k-r1.md (done criteria), reports/flakes-k.md "## r1"; then `git diff 79c8d2b5..e2508ec1`.
Checks, each with its command and files:
1) Red first: each test commit fails on its parent and passes after its fix (d13da066/eef3b496, 7641a6f3/6192f696, b7eddbf9/92185802, fd0818fc/fa53ad43, 9967f062/1bbfdd4a, 90e9e743/2f069c98); say which red you reproduced and which you read only.
2) Product or test: for each fix say whether it changes product code or only a test; a fix that only widens a test (IN1 accepts [false,true]) must match documented behaviour (name the doc line) and not hide a real double fire.
3) TR1 (testing.rs, images.rs, ipc.rs -49): the subscriber before any core; is any ipc.rs behaviour removed with the 49 lines, not only moved? Release build unchanged?
4) AG1 (inject.rs, tests/agent.sh): the click as clicker, not user; can a real client still be told apart from the injector (hello order, press before inject answer)?
5) FLAKE-SIG (images.rs thread scan): a worker that ends between list and read; no panic, no leak; churn test 1 s still red on the old scan (e2508ec1).
6) Doors waits (tests/doors/lib.sh, xp, tiling, mc): bounded waits with a clear FAIL line, no sleep-only fix; construct unchanged (say why that is right or a gap).
7) HUD1 stays open with an evidence-only log (f27eb2e3): the log names late versus never; no assert loosened.
8) `cargo test -p chora --lib`, `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines.
Known, not findings: e2e red on gate-tiling-apps-click "Zebra row did not start zebra" (ti-1; the PM checks it in a train).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/flakes-k.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
