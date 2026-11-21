# p2-harness-l r1 (rc 2): the persona harness limits before the tiling and construct rounds

Model: sonnet, effort medium (Python and shell under tests/persona; no product code)
Lane: cloud, claude/p2-harness-l at a84effda. Cost at most 0.95 %; stop at 150 k context. One area: tests/persona (round.sh, check.py, test_check.py, contracts) and tests/persona-selftest.sh.
Why: plans/week-3.md row 13 (claude/session-log). rc 2 needs persona rounds whose red contracts are product faults, not harness limits: reports/p2-r3-tiling.md lines 10-18, 49-52, 75 (L1, L2, L3 red as harness limits), backlog.md 299 (ph-1, ph-2), 362 (RPM-1). P2-CHECK is done (reports/mc-fix.md item 4): skip it.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "p2-harness-l BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
First check each item on the head; a closed one: say so with the line that proves it, skip it. Each fix: a selftest that is red before it (own test: commit) and green after.
1) L1 (tiling, Kemal): contract "expected 1 windows gone, got []". Find whether the begin step or the contract reads the wrong state. If the window really stays (Super+q closes nothing in tiling), that is a product fault: do not fix it, name it as a finding with the log line for the PM.
2) L2 (tiling, ricer): red by the empty config and the focus text; make the begin give the persona the real default config, or mark the contract `verify` with the reason, never green by loosening.
3) L3 (tiling, ricer): "expected the focus with tiling, holders [xp]" after the give goal; the contract must read the holder after the run's last handover, not at a fixed point.
4) ph-1: round.sh probe waits a fixed 3 s; wait for the probe window in the scene (timeout 20 s, then fail with a message).
5) ph-2: check.py reads only the construct state for I1/I3, so a cube face's xp window counts x21; read the child look's windows.
6) RPM-1: tests/persona/test_check.py gets a selftest of with_children (child look windows count as shown).
7) Checks: `tests/persona-selftest.sh`, `python3 -m pytest tests/persona -q` (if the repo runs it so), `pre-commit run --files <changed>`.
8) Gate: the full gate `tests/merge-gate.sh --jobs 1` (no --only) green on the branch merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Start runs in the background and poll in the same turn; never end a turn to wait.
Done when: each item fixed with a red-then-green selftest, skipped as closed, or named as a product finding; full gate green.
Report: reports/p2-harness-l.md "## r1" on claude/session-log (item, state, red sha, fix sha, selftest name; product findings; gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with the head sha.
Rules: push claude/p2-harness-l and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
