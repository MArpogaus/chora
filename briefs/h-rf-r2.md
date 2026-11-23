# h-rf r2 (rc 3, process; rule 8 guard): fix the review r1 findings of red-first per id

Model: sonnet, effort high (harness shell change with selftests; routine but exact)
Lane: cloud, claude/h-rf at fff3f1ab (r1 head). One round. Stop at 150 k context; cost at most 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `git fetch origin claude/autonomous-dev-session-3w-i33pg7`, `cargo check --workspace`; a failure: send_message the PM "h-rf BLOCKED: <line>" and stop.
A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/h-rf.md "## Review r1" (rhr-1..rhr-6, item 4).
Scope: tests/red-first.sh, tests/merge-gate.sh, tests/merge-gate-selftest.sh, CONTRIBUTING.md only. Each fix: its selftest case red in one commit first, then the fix.
1) rhr-1 MAJOR: cut the message file at the scissors line (`# ------------------------ >8 ------------------------`) before reading ids; selftest with a diff below the scissors.
2) rhr-2 MINOR: a word followed by `.` and a digit is no id, and a short denylist (utf, sha, base, ipv, gate, gtk); selftest cases utf-8, sha-256, gate-0.8, gtk-3.0.
3) rhr-3 MINOR: one per-id verdict function in tests/red-first.sh used by the hook and the gate step (no second loop in merge-gate.sh).
4) rhr-4, rhr-6 NIT: split glued ids where a digit run ends (`sh-33-sh-34`); correct the count in reports/h-rf.md "## r1" via your r2 section.
5) In flight (PM decision 11-23 15:00): commits committed before the rule lands are checked by the old rule (a test in the fix or before it), not per id; a constant in tests/red-first.sh holds the cut time (committer date of your r2 head); a selftest proves an old pair passes and a new one does not. No reword, no history change on any branch. After: rerun item 4 over claude/xp-readable, claude/guard-3, claude/co-reach, claude/tiling-disc, claude/leak heads: all pass.
Gate: `bash tests/merge-gate-selftest.sh` green; full `tests/merge-gate.sh --jobs 2` on claude/h-rf merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7, detached: `setsid nohup bash -c '(tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll /tmp/gate.log in the same turn; never end a turn to wait. A red cell you did not cause (keys, xp-run, e2e, launcher-click with fuzzel < 1.11: sh-34; acts stalls: fs-r6): name it with its log line; do not rerun to green.
Done when: rhr-1..4 fixed red first, in-flight rule with its selftest, item 4 rerun all pass, gate green or foreign reds named.
Report: reports/h-rf.md "## r2" on claude/session-log (shas per id, selftest counts red and green, item 4 table, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/h-rf and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
