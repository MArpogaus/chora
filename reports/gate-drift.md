
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

### PM review r1 (16:20 UTC; PM not the builder; small harness diff, +46 lines)
Diff a84effda..96a91f1f: tests/merge-gate.sh preflight_tools and tool_minimums, selftest +17. Checks: red commit first (6e21c4e6); version compare by sort -V (1.11 vs 1.11.1 passes, 1.9.2 fails); missing and unknown versions SKIP, never PASS; --push still refuses on SKIP without --allow-skip; the PM machine has fuzzel 1.11.1, so trains stay unchanged. Findings: MINOR rgd-1, merge-gate.sh preflight_tools calls `command -v` twice; one call is enough. Verdict (computed): MERGE-OK; harness branch, so it trains first (t102).
