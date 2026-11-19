# kit-connect r1 (F2, F4): one test connect helper in the kit

Model: sonnet, effort high (routine refactor across plugins, tests only)
Lane: cloud, branch claude/kit-connect at 103844da (kit-report 582ab506 + lint-reason 1f99a98f). One round, stop at 250 k. Cost about 0.8 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-connect r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: reports/kit-report.md "## r1" (step 4) and "## r1 PM review"; briefs/kit-report-r1.md item 4.
1. Find the test-only allows or expects whose reason says the test connects to its fake core (about 14 in 12 files, 6 parts). Add one `chora_kit::testing::connect` (or the name that fits the kit) that does that connect once, with its one reason; switch each site to it and drop the site's allow or expect.
2. Pin it: a kit unit test of the helper; the existing tests of each site stay green unchanged in what they assert.
3. tools/copies.toml (cut-check): a row for the old inline connect pattern, max 0, if the pattern is greppable; else say why not.
Counts in the report: `#[allow`/`#[expect` attributes before and after (per attribute, not per line); the total must fall by the sites removed, none added outside the kit.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy` green. Poll it in the same turn; never end a turn to wait.
Done when: items 1-3 in; gate green; reports/kit-connect.md "## r1" on claude/session-log with the counts; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-connect and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
