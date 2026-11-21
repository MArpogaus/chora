# deps

## r1

Head claude/deps a606a00f on 1c599c74 (already contains origin/claude/autonomous-dev-session-3w-i33pg7).

| PR | package | from | to | result |
|---|---|---|---|---|
| #108 | typos (pre-commit) | v1.52.0 | v1.52.2 | 15a26453 |
| #107 | ruff-pre-commit | v0.17.3 | v0.17.5 | 2320a4d9 |
| #94 | astral-sh/setup-uv | v10.3.2 | v10.3.3 | 3a5aaca6 |
| #88 | schemars | 1.2.2 | 1.2.3 | already on the head (Cargo.lock 1.2.3), no commit |
| #86 | serde_json | 1.0.152 | 1.0.154 | already on the head (Cargo.lock 1.0.154), no commit |
| #2 | actions/download-artifact | 8.0.1 | 8.0.2 | already on the head (pin 9000827c, v8.0.2), no commit |
| #1 | actions/upload-artifact | 7.0.1 | 7.0.2 | already on the head (pin cf430e03, v7.0.2), no commit |

Deferred majors: zbus 6 (#96), serde-saphyr 2 (#87), noted in docs/known-issues.md (a606a00f).

Checks:
- cargo check and build --workspace: ok. cargo test --workspace: 2793 passed, 0 failed.
- cargo deny check: advisories, bans, licenses, sources ok (warning: unmatched license allowance).
- cargo machete: clean.
- pre-commit run --all-files: all pass except shellcheck, SC2178 in tests/gate-injected-presses.sh line 54 (`probes=0`). Not caused by this branch (file untouched); the commit hooks and the gate's pre-commit cell pass.

Gate: tests/merge-gate.sh --jobs 1 (full): all steps passed, 50 cells, 0 FAIL.

### PM review r1 (builder not the reviewer)
- a606a00f on a84effda: 3 version lines, 3 doc lines. setup-uv pin 68559e99 matches Dependabot PR #94 exactly; ruff v0.17.5 and typos v1.52.2 match #107, #108. The four "already on the head" rows check out in Cargo.lock and the workflow pins.
- MINOR: "DEFERRED DEPENDENCIES" sits under the last look's section in the style of a severity heading; a later docs round may give it its own `##`.
- Out of scope, harness backlog: shellcheck SC2178 in tests/gate-injected-presses.sh:54 under `pre-commit run --all-files` (not from this branch).
- Verdict MERGE-OK (0 BLOCK, 0 MAJOR); joins the next train after t101.
