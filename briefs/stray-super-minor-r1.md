# stray-super-minor r1 (rc 1, 9: the kit's swallow handover; close rss-12 and rss-13 of review stray-super r3)

Model: sonnet, effort high (kit fix with unit tests, routine)
Lane: cloud, claude/stray-super-minor-r1 at 84f66d43 (session head, t133). One round.
Step 0: `git fetch --unshallow`; `cargo check --workspace`. A failure: send_message the PM "stray-super-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.8 %.
Findings (full text: reports/stray-super.md "rss-12", "rss-13" on claude/session-log):
- rss-12 MINOR crates/chora-kit/src/swallow.rs:146: `say_swallow` sets `said` before the emit and drops a refusal (core refuses > 32 emits/s per client, crates/chora/src/ipc/emit.rs:16); a lost `on` or `off` brings rss-1 or rss-7 back. Fix: set `said` only on success and say again at the next grab set (or a short retry); unit test with a refused emit.
- rss-13 MINOR crates/chora-kit/src/runtime.rs:416: a joiner logs one "X is grabbed already" line per key (75 per tiling join). Fix: one summary line per grab set at `end_grabs` (count, asked again when free); unit test on the log lines.
- rss-11 (construct) stays open: construct has the open co-79 MAJOR (rules.md 6).
Steps:
1) Red first: per finding a commit with the failing test before its fix (rules.md 8).
2) No new `sleep` line anywhere. tools/sleep-ratchet, tools/size-ratchet, tools/lint-ratchet pass on your head (counts never raised).
3) One full tests/merge-gate.sh (no --only) on your head.
Done when: both findings have a red test commit and a fix, the three ratchets pass, full gate PASS on one run.
Report: reports/stray-super.md "## minor r1" on claude/session-log (finding | red sha | fix sha | test, ratchet output, gate run dir). send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, ratchets, gate.
Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/stray-super-minor-r1 and the report on claude/session-log (fetch and merge on reject, no force); author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
