# mc-tidy r2 (F7; sh-34 triage input): gate green on the new head, and why keys is red in your container

Model: sonnet, effort medium (routine gate and one environment check)
Lane: cloud, same worker, branch claude/mc-tidy a3a9e8ef. Cost at most 0.5 %. No product change.
1) Merge origin/claude/autonomous-dev-session-3w-i33pg7 (bc153929, t115 green here, keys PASS in the PM container).
2) sh-34 evidence (no fix): `command -v foot; foot --version`; `tests/merge-gate.sh --only keys` 3x on the merged head; on a red, the lines of the keys log around gate-construct-keys-persona.sh:88 and what the Foot row spawned (process list, launcher log). Say: environment (foot missing or other version) or product.
3) Full `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (background, poll in the same turn); push claude/mc-tidy.
Report: reports/mc-tidy.md "## r2" (merged sha, foot version, keys 3x rate with log lines, gate count) and findings/shared.md sh-34 one evidence line; send_message the PM at most 5 lines.
Rules: as r1 (identity, no attribution, never --no-verify, push only claude/mc-tidy and claude/session-log, no system packages).
