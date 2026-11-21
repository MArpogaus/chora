
## r1

Branch claude/gate-drift: red 6e21c4e6 (selftest: fuzzel 1.9.2 stub, step not SKIP), green 96a91f1f (preflight_tools, 29 lines in tests/merge-gate.sh). Merged with origin/claude/autonomous-dev-session-3w-i33pg7: already up to date.

Version table (tool_minimums in tests/merge-gate.sh, one table, one function):

| tool | minimum | steps | source |
|---|---|---|---|
| fuzzel | 1.11 | launcher-click | findings/mc.md mc-47 (1.9.2 ignores clicks, 1.11.1 works); tests/gate-package-deps.sh (fuzzel >= 1.11) |

Checked, no known minimum, so out of the table: foot, grim, wtype, ydotool, pre-commit, cargo-deny, cargo-machete (no finding, changelog or test names a minimum). Cells of other steps (mc-sofa, xp-fields) already SKIP on old fuzzel through need_fuzzel in tests/gate-lib.sh and stay as they are.

Output: `SKIP: fuzzel 1.9.2 < 1.11` in the step log, summary line `skipped: launcher-click (fuzzel 1.9.2 < 1.11)`; a missing tool says `fuzzel missing < 1.11`; --push refuses unless --allow-skip names the step.

Selftest: 3 red before (reason text, SKIP line, --push refusal), all ok after.
Gate: tests/merge-gate.sh --jobs 1, no --only: 49 steps PASS, 0 SKIP, 0 FAIL, "all steps passed".
