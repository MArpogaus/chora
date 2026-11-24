# review stray-super r3 (rc 1: rss-7 MAJOR; rss-8, rss-9 MINOR; rss-10 NIT): seeded review of claude/stray-super-r3 before its merge

Model: opus, effort high, because the diff changes the shared key path in chora-kit (swallow yield, D118 joiner rule) for all four looks, a core-adjacent review; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/stray-super-r3 head 73c6b19c, base 1eb30643 (9 files, +266 -31). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-stray-super-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, run 2, EXIT 0, 65 PASS).
Read on claude/session-log: briefs/stray-super-r3.md (done criteria), reports/stray-super.md "## Review r2" (rss-7..10) and "## r3"; `git diff 1eb30643..73c6b19c`.
Checks, each with its command and files:
1) Red first: ff9218a9 (gate-join-keys tiling+construct --overlay cell, kit and xp unit reds) before bb9d4f69, e92b0b4d; 9cdef94c before 0bf236a6 (`git log --oneline --stat 1eb30643..73c6b19c`). Does 73c6b19c change behaviour or only tests? A fix with no red test before it is MAJOR.
2) The joiner rule (crates/chora-kit/src/swallow.rs, runtime.rs; D118 in docs/decisions.md): a look yields only to a manager that connected after it and emitted NAME.swallow {on} at its first grab_swallowed; off at an end_grabs without one. Edge cases: two joiners, one swallowing and one not; a swallowing joiner leaving while a non-swallowing one stays; a look reloading; swallow {on} arriving before client.connected; a joiner that crashes without off (does the earlier look take its swallow back on client.disconnected?). `cargo test -p chora-kit`.
3) rss-8 (plugins/chora-xp/src/wm/startup.rs): subscribe before the first grab, as tiling and mc; rss-9: construct EVENTS has input.ungrabbed; rss-10 gone. `cargo test -p chora-xp -p chora-mc -p chora-tiling -p chora-construct`.
4) xp-46, mc-60, rss-1, rss-2 stay fixed (gate-join-keys cells in the lane's gate log, or `bash tests/gate-join-keys.sh` only if cheap).
5) Stated costs: about 75 "grabbed already" log lines per join in tiling (r2: 15-19) and the 1 changed mc test line: is the mc test weakened? Is the log noise a MINOR (POLS for a ricer reading logs) and can it be one line per join?
6) No core change (crates/chora untouched); no new sleep (sleep-ratchet 779); D118 text matches the code.
Known, not findings: sh-42 (tiling-join keyboard loss; SH42-FIX r1 running on the core focus path); rpx1-* other than xp-46; mc-61; rv9j-1..5; rth-1.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rss-N from rss-11, severity, file:line, text, fix in one sentence). Append to reports/stray-super.md on claude/session-log under "## Review r3".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
