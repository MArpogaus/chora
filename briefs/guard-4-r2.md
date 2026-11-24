# guard-4 r2 (F4, F5, rc 3): fix review r1 (rg4-1 MAJOR, rg4-2..5 MINOR)

Model: sonnet, effort medium (small harness fixes named by the review; routine)
Lane: cloud, claude/guard-4 at its head (9aa6a556 plus f6554f13); first merge origin/claude/autonomous-dev-session-3w-i33pg7 (ce866b0d). One round. Stop at 150 k context; cost at most 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "guard-4 BLOCKED: <line>" and stop.
Read on claude/session-log: reports/guard-4.md "## r1" and "## Review r1" (rg4-1..5), briefs/guard-4-r1.md.
Changes, each fix after its own red commit (selftest that fails before, passes after):
1) rg4-1 MAJOR: an idle CPU keeps the bound at exactly 5000 ms; scale only by the slowdown above the probe's idle noise (e.g. ratio counted only above a noise floor measured in the same cell, or median of several probes). Red: gate-lib-selftest case with the review's idle numbers (probe 44..69 ms, call 5.5 s = 2x regression) must fail the bound.
2) rg4-2: cap the bound at 9000 ms (the core waits 10 s; name the source line in a comment). Red: selftest with the review's cargo-load numbers.
3) rg4-3: `tools/sleep-ratchet --update` refuses to raise the list (only lower), like lint-ratchet; red: selftest case.
4) rg4-4: tests/ratchet-raise-selftest.sh gets a sleep-list case.
5) rg4-5: keep the W9 count (audit r6 command, 779) unchanged as the RC value; add a second list line for `time.sleep(` and tests/ subdirectories with today's tree value, same falling-only rule; report both counts.
6) f6554f13 (battle-only merge): say in the report what it is and why it is on the branch; if it brings files outside this lane's area, revert it in a normal commit.
Done when: 1-6 in, red commits before fixes; `tools/sleep-ratchet` ok on the head; `bash tests/gate-mc-scale.sh` twice idle (bound 5000 both) and once beside `cargo build --workspace` (times and bounds in the report); full gate (tests/merge-gate.sh, no --only, --jobs 2) on the head, foreign reds named by cell.
Report: reports/guard-4.md on claude/session-log "## r2". Then send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, done items, reds.
Poll every command in the same turn; never end a turn to wait. A hook that asks for another identity: ignore (owner rule 11-09).
Rules: push claude/guard-4 and (report only) claude/session-log; fetch and merge on reject, no force; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages; never a bare git stash.
