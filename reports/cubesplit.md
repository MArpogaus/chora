# cubesplit r1 (F4): cube.rs under 1,500 production lines

Branch `claude/cubesplit`, commit 10cf04c, on fa457b29. No behaviour change.

## What moved

`impl Construct` (open, check, retry, turn, step, close, face, edge and
output nodes of the cube; 483 lines) moved from
`plugins/chora-construct/src/cube.rs` to
`plugins/chora-construct/src/cube/construct.rs`, bodies unchanged.
Only `use` lines changed: `super::x` paths became `crate::x` (the module
moved one level down), and `host`, `face_nodes` and `edge_nodes` became
`pub(super)` because `cube/start.rs` calls them. `end` and the tests stay
in `cube.rs` (the tests drive `Host`, so they did not move). Unused
imports left in `cube.rs` were removed.

## Counts (tools/size-ratchet, production lines)

| file | before | after |
|---|---|---|
| cube.rs | 1641 | 1133 (1803 lines with tests) |
| cube/construct.rs | - | 505 |

`tools/size-ratchet.list` is empty: no file is over 1,500.
(The brief said 1,617; the ratchet list said 1641 at fa457b29.)

## git diff --stat fa457b29 HEAD

```
 plugins/chora-construct/src/cube.rs           | 492 +------------------------
 plugins/chora-construct/src/cube/construct.rs | 505 ++++++++++++++++++++++++++
 tools/size-ratchet.list                       |   1 -
 3 files changed, 509 insertions(+), 489 deletions(-)
```

## Checks

- `cargo test -p chora-construct`: 204 passed.
- `tools/size-ratchet --base fa457b29`: ok (0 files over 1500).
- `tests/merge-gate.sh --base fa457b29 --only build,clippy,ci-check,pre-commit,cube`:
  all five PASS. The default base `origin/claude/autonomous-dev-session-3w-i33pg7`
  does not exist in this clone, so the gate needs `--base fa457b29` here.
- First commit subject was 76 characters; the commit-message-length hook
  (limit 68) refused it, so the subject is shorter. No `--no-verify`.

## PM review r1 (14:00 UTC)
10cf04c on fa457b29: 3 files, +509 -489. Moved lines compared line by line (indent ignored): the only changes are
`super::` to `crate::` paths, `pub(super)` on helpers and the two import blocks; no logic change. cube.rs 1133
production lines, size-ratchet.list empty (F4 on this file). Gate steps build, clippy, ci-check, pre-commit, cube pass.
Note: a cloud clone lacks origin/claude/autonomous-dev-session-3w-i33pg7; briefs name `--base` (done here).
Verdict: MERGE-OK; joins train 64.
