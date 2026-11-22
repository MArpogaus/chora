# opt-in r1 (plan slice 20, OPT-IN, rc 3: the opt-in gates green once, early)

Model: haiku, effort high (test runs and a table; no product code)
Lane: cloud, claude/opt-in at 7a6c0eb8 (session head). Cost at most 0.8 %; stop at 200 k context. One area: tests (the scripts below, read-only) and tests/battle/RESULTS.md is not yours (lane sim-long); you write only your report. No product code, no script change: a fault goes into the report as a finding.
Why: plans/week-3.md slice 20 on claude/session-log (rc 3: opt-in gates each green once, no evidence found this week). Run now, before the candidate, so a fault has days to be fixed; the candidate run follows later.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo build --workspace`; a failure: send_message the PM "opt-in BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Run each once, one at a time, in the background, polling in the same turn (never end a turn to wait), each log to /tmp/opt-in/<name>.log: tests/gate-watchdog-starved.sh, tests/gate-mc-scale.sh, tests/gate-construct-keys.sh, tests/gate-construct-keys-persona.sh, tests/gate-demo.sh (ffmpeg missing: the clip part skipped is fine, say so), tests/gate-0.9-shader-hang.sh under `prlimit --as=6000000000` as its header says. Read each header first for its env and BIN. Clean up with tests/kill-session.sh only (never by process name).
2) A red script: run it a second time only to tell a flake from a fault; report both runs with their failing line. Do not change the script or the product.
Gate: none (no change to code); `git status` clean on claude/opt-in at the end.
Done when: each of the six scripts has a result (PASS, FAIL with its line, or SKIP with the reason the script itself prints), with durations.
Report: reports/opt-in.md "## r1" on claude/session-log (table: script, result, seconds, failing line; findings oi-1.. with look and log line); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines.
Rules: push claude/session-log only (fetch and merge on reject, no force; change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
