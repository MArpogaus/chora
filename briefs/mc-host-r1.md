# mc-host r1 (F5, rc 3): mc-58, the field gets no keyboard after the click, red test first

Model: sonnet, effort high (input focus race in a gate cell; no new feature)
Lane: cloud, branch claude/mc-host at 1ec04694 (session head, t117 green). One round, stop at 250 k. Cost at most 0.8 %.
Why: tests/gate-mc-host.sh failed "the field gets no keyboard after the click" in two lane gates (co-reach r1 light on fe369799, xp-readable r3 dark on 51cd0bc0), neither touching mc; mc-57 was closed 11-23 by 00406cd8. An open MAJOR flake blocks rc 3 and F5.
Step 0: `command -v cargo sccache foot python3`, `git fetch --unshallow`, `cargo check --workspace`; no setup.sh; a missing tool or failure: send_message the PM "mc-host BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/mc.md mc-57, mc-58; reports/mc-tidy.md.
Runs take long: start in the background and poll in the same turn; never end a turn to wait.
1. Reproduce: run the mc-host cell alone 20 times (`tests/merge-gate.sh --jobs 2 --only build,mc-host` or the gate script directly); count red of 20, light and dark. Under load too: 10 runs with `cargo build` of the workspace running beside it.
2. Cause: product (keyboard focus not given to the field on click, a focus race in chora-mc or the core) or harness (the check reads before the focus lands). Name the file and line.
3. Red test commit first (a unit test, or a gate check that fails reliably with a forced delay), then the fix. A product fault is fixed in the product; no timeout raised as the fix; no known-red entry.
4. Gate: `tests/merge-gate.sh --jobs 2` (full, no --only) once; report exit and red steps; no rerun to green.
Done when: red then fix commits on claude/mc-host, pushed; mc-host cell 20/20 green plus 10/10 under load; full gate green or its red step named and foreign; reports/mc-host.md "## r1" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message: "mc-host DONE: <head sha>" plus cause and counts (at most 6 lines).
Rules: push claude/mc-host and claude/session-log only (fetch and merge on reject, no force; on session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
