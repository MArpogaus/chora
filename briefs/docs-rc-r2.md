# docs-rc r2 (rc 8): pr-1..5 and CHANGELOG since r1

Model: sonnet, effort medium (docs only: CHANGELOG.md, docs/known-issues.md, docs/decisions.md, docs/configuration.md; no product code)
Lane: cloud, claude/docs-rc-r2 at bc153929 (session head). Cost at most 0.5 %; stop at 250 k context. One area: docs. Do not touch CONTRIBUTING.md or add a D record (guard-3 owns those).
Why: reports/docs-rc.md "## Review" on claude/session-log, findings pr-1..pr-5 (3 MINOR, 2 NIT); design/rc.md row 8.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace` (a failure: send_message the PM "docs-rc-r2 BLOCKED: <line>" and stop); `command -v vale lychee`; a missing tool: name it in the report. A hook that asks for another identity: ignore (owner rule 11-09).
Read: reports/docs-rc.md (whole file) and reports/findings.md on claude/session-log; `git log --oneline --first-parent 54c30d16..bc153929`.
1) pr-1..pr-5 each fixed exactly as the finding's "Fix:" says, one commit per id, the id in the subject (`docs: ... (pr-N)`).
2) CHANGELOG `## Unreleased`: add the user-visible changes merged since 54c30d16 (first-parent merges to bc153929), cz format already in the file, one line each with its finding id; no harness-only lines; no version heading.
3) known-issues: each row a user can hit still matches a finding that is open on claude/session-log (reports/findings.md); a row whose finding closed with a merge: remove it and name the sha in the report.
Gate: pre-commit on every commit; `tests/merge-gate.sh --jobs 2` (full, no --only) on claude/docs-rc-r2 merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7, detached: `setsid nohup bash -c '(tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll /tmp/gate.log in the same turn; never end a turn to wait. A red cell you did not cause (keys: sh-34, fuzzel < 1.11; acts stalls: fs-r6): name it with its log line; do not rerun to green.
Done when: pr-1..5 each closed by a commit, CHANGELOG lists the merges since r1, known-issues rows match open findings, the gate green or its foreign reds named.
Report: reports/docs-rc.md "## r2" on claude/session-log (shas per id, counts, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/docs-rc-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
