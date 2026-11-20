# lint-down-r2 (F4 guard: the lint ratchet counts right and keeps the trains green)

Model: sonnet, effort high (one shell script, its tests and a list; the PM reviews)
Lane: cloud, branch claude/lint-down, continue at 0d1394f1. One round, stop at 250 k. Cost about 0.4 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "lint-down BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/lint-down.md "## PM review r1" (rv-1..rv-3).
Scope: tools/lint-ratchet, tools/lint-ratchet.list, its test (tests/ new file or tests/test_*.py as the repo does it), tests/merge-gate.sh ci-check line only. No Rust changes.
Items:
1. rv-2: no-reason counts an `#[allow(`/`#[expect(` attribute as without reason only when the whole attribute, up to its closing `)]` over any number of lines, has no `reason`. Red first: a test over a fixture tree (one wrapped attribute with a reason on a later line, one without reason, one eprintln) that the old count gets wrong; then the fix. Keep POSIX sh plus awk; no new tool.
2. rv-1: the list holds the counts of a scratch merge of origin/claude/autonomous-dev-session-3w-i33pg7 with these MERGE-OK shas, in this order: 741d6bca 5dbd0ccf a767790d 591e98f8 91e6a114 17d9a798 40b61f28 (fetch each by sha from origin; a conflict: stop and report it). Run `tools/lint-ratchet --update` there and commit that list on claude/lint-down. Report the counts of the base and of the scratch merge.
3. rv-3: the help says: no-reason and eprintln only fall; a new allow with a reason raises `allow` in the list in the same commit and the commit message says why.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy`; poll it in the same turn; never end a turn to wait. Show a valid commit is not blocked.
Done when: items 1-3 with shas, the test red then green, gate cells pass, reports/lint-down.md "## r2" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/lint-down and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
