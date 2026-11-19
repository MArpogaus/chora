# mv3 r1 (F4): moves without logic in core and chora-scene

Branch claude/mv3 (from 27bd60c7), head 917df9d0, pushed. Six commits (four splits, one ratchet; the first scene commit message was shortened by the hook).

| file | production lines | new module | lines |
|---|---|---|---|
| chora-scene scene.rs | 2,033 -> 1,214 | scene/checks.rs (limits, check_* fns), scene/commands.rs (apply_one .. release_camera), scene/tree.rs (create .. reset) | 373, 274, 207 |
| core render.rs | 1,857 -> 1,180 | render/picking.rs (surface_under .. layer_hit), render/textures.rs | 515, 192 |
| core input.rs | 1,728 -> 1,221 | input/grabs.rs (impl State grabs), input/plane.rs (plane drag, PlaneGrab) | 240, 291 |
| core ipc.rs | 1,702 -> 1,324 | ipc/hello.rs, ipc/query.rs, ipc/grabs.rs | 144, 198, 74 |

Ratchet: tools/size-ratchet.list lowered with --update; only cube.rs 1,641, region.rs 2,271, voice chat.rs 1,531 remain over 1,500 (ok: 3 files). Counted with tools/size-ratchet.

Non-move lines (git diff --color-moved=dimmed-zebra --color-moved-ws=allow-indentation-change 27bd60c7..HEAD):
- `mod`, `use` lines, `pub use` re-exports in render.rs and input.rs, `//!` module docs, `impl State {`/`impl Scene {` wrappers.
- `pub(super)` on moved private fns, consts and structs.
- rustfmt re-wrapped signatures that got longer by `pub(super)` (tree.rs attach, hello.rs ipc_admin, query.rs lineage, textures.rs texture_scale).
- 30 inline `//` comments (hook no-inline-comments rejects moved lines) became extra `///` lines on the enclosing fn, as in mv2-tiling: scene 6, picking 7, ipc 17 (counts of lines moved into docs). No code changed.
- `use super::*` was replaced by the explicit lists clippy gave (wildcard_imports is pedantic).

Gate (head 917df9d0): one full run stopped at pull ("192 px wide for 200", the F21 width read; PASS twice alone). All steps before it PASS. Then the skipped steps in one run: construct-hud FAIL once ("light-1280x720@1.5: no claim of the indicator", the known flake of mv1 r2), then construct-hud, pixels, mc-paths, mc-sofa, mc-handover, e2e all PASS in the next run (e2e PASS, so F27 did not hit here). Cargo test and clippy PASS in the gate.

## Refactor proposals (ranked)
1. input.rs 1,221, ipc.rs 1,324 are still big: ipc.rs impl State dispatch/accept (about 500 lines) to ipc/accept.rs, S.
2. gate-construct-pull.sh width read once after sleep (F21) and construct-hud indicator claim: two flakes seen in one run; until_ok fix, S.
3. Parent files now re-export moved items (`pub use picking::...`): switch callers to the child path and drop the re-exports, S.
4. render.rs context/plane element builders (context_elements .. plane, about 300 lines) to render/planes_build.rs, S.

## PM review 04:33 (independent; builder was the lane)
`git diff --color-moved=plain --color-moved-ws=allow-indentation-change 27bd60c7 917df9d0 -- '*.rs'`: the
non-moved lines are only `mod`/`use`, `pub(super)` on moved items (Pin, Target, Hit, Found, check_*,
command fns), `impl Scene {` wrappers and rustfmt reflows of signatures. 15 files, all in chora-scene and
the core; voice untouched (its ratchet line 1535 to 1531 is a recount). Ratchet list drops four files.
Verdict MERGE-OK (0 BLOCK, 0 MAJOR). Train 56 after train 55. F4 still open: cube.rs, region.rs, chat.rs.
