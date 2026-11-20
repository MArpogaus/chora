# docs-f8e

## r1

Lane: cloud, sonnet, branch claude/docs-f8e from ab4cbb4f. Brief briefs/docs-f8e-r1.md.
Step 0: unshallow, `cargo check --workspace` green.

| item | commit | note |
|---|---|---|
| 1 DOCS-F8e kit line | 8ac9ad02 | CONTRIBUTING.md: `swallow` line (what, when, `grab_swallowed` after own grabs, before `end_grabs`) |
| 2 D118 | d3dbaa22 | docs/decisions.md, table form of D117, accepted by the PM after review r2 of co-p2-fix, train 85 |
| 3 CM-R1 test | ff69a833 | `kept_nodes_stay_visible_during_the_resume_grace` (crates/chora/src/ipc.rs): green on the head, no behaviour finding |
| 3 CM-R1 doc | c529242b | docs/api.md Window managers, resume: kept nodes stay shown during the grace |

Test note: the query filter `visible: true` means shown on an output and this core has no output, so the test reads the `visible` prop of each node of `owner:wm` (4 nodes): all true before, within the grace (GRACE - 1 ms) and after the resume hello; the list is equal each time.
Gate: merge of origin/claude/autonomous-dev-session-3w-i33pg7 was already up to date. merge-gate --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib: all 6 PASS (partial run, nothing pushed by the gate).
Head: c529242b (pushed to claude/docs-f8e).
