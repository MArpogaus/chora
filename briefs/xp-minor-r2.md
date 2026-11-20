# xp-minor-r2 (RC 2, shared bar "honest failure states"): Run... closed versus failed by the class of fuzzel's line (PM review r1 pr1-1)

Model: sonnet, effort medium (small fix in the kit launcher)
Lane: cloud, branch claude/xp-minor at 22884a79. One round, stop at 250 k. Cost about 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "xp-minor r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/xp-minor.md "## r1" and "## PM review r1"; findings/xp.md row xp-36.
Item 1 (pr1-1, MAJOR): crates/chora-kit/src/launcher.rs `ask`: fuzzel exits 1 on Escape and on every error, so the exit code cannot tell them apart. Decide by the line: Failed with the first stderr line whose trimmed text starts with "err:" (fuzzel's log prefix " err"), with that text after the prefix; else Closed (warn, info or no line), whatever the exit code; no exit status at all (a signal) with no err line is Closed too. Red first: restore a_launcher_that_fails_says_why_once to exit 1 with " err: config.c:17: [colors].prompt: invalid key" on stderr and its notice; keep a_warning_then_escape_stays_silent ("warn: ..." then exit 1); add a cell for an err line after a warn line. Then the fix. Update the doc comment of `ask`.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,xp,xp-question,launcher-click,doors`; poll it in the same turn; never end a turn to wait.
Done when: red and fix commits pushed to claude/xp-minor; the gate list passes; reports/xp-minor.md "## r2" on claude/session-log names commits, cells and gate.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/xp-minor and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
