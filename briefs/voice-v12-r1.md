# voice-v12-r1 (RC 9 logging/flakes, vision "honest states"; backlog V12): the three voice scripts that fail on the session branch, then into the gate

Model: sonnet, effort high (triage of three test scripts and their causes; the PM reviews)
Lane: cloud, branch claude/voice-v12 from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace` and `cargo build --workspace`; a failure: send_message the PM "voice-v12 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: backlog.md line V12, reports/debt-rc.md (V12 line), reports/harness-r2.md (step 2 note). On the branch: tests/voice-acp.sh, tests/voice-events.sh, tests/agent.sh, tests/fake-voice.py, tests/merge-gate.sh (how a cell is added).
Known faults (V12): voice-acp.sh: a yes without the key counts as nothing; voice-events.sh: after voice sleep mc shows none; agent.sh step 2: xs_are 0 640.
Scope: services/voice, tests/voice-*.sh, tests/agent.sh, tests/fake-voice.py, tests/merge-gate.sh (new cells only). Do not touch plugins/chora-mc (lane mc-remote owns it) or crates/chora; lane lint-down edits eprintln and allow lines in services/voice, so keep your services/voice changes small and merge origin/claude/autonomous-dev-session-3w-i33pg7 before the gate. A cause in mc or core: no fix, a finding in the report with the exact line, the script marks that step `known: <cause> (mc lane)` in the way tests/acts/known-red.toml does it.
Work, per script:
1. Run it on the base, keep the red output (this is the red evidence; a script that passes on the base: say so, with output, and it is no red).
2. Decide the cause: script wrong (asserts what the design no longer says: name the design line), service wrong, or another area.
3. A service fault: red test commit first (cargo test in services/voice or the script itself), then the fix.
4. Each script passes three times in a row headless; then add it as a gate cell (voice-acp, voice-events, agent) in tests/merge-gate.sh, in the default set only when 3/3 green, else `--only` with the reason.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,voice-acp,voice-events,agent,matrix`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: each of the three scripts passes 3/3 or carries a named known step with a finding, the cells exist, gate cells pass, reports/voice-v12.md "## r1" on claude/session-log (per script: red output, cause, shas, cell, 3/3 runs).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/voice-v12 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
