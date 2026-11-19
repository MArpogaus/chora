# mv4 r1 (F4): region.rs and voice chat.rs below 1,500 production lines

Branch claude/mv4 (from a17e7858), head 7e339880 plus the ratchet commit, pushed. No merge of the session branch was needed: it did not move.

| file | production lines | new modules (lines) |
|---|---|---|
| mc region.rs | 2,271 -> 1,284 | region/items.rs 618 (Item, Put, Plan, the plan builders), region/hits.rs 226 (hit and close nodes, clicks), region/fit.rs 228 (line fit, place, rect; 183 production) |
| voice chat.rs | 1,531 -> 1,049 | chat/gate.rs 595 (380 production: detail, always keys, allow lists, the confirmation questions), chat/notes.rs 212 (128 production: spoken, stopped and cancelled notes, announce and cut checks) |

Counted with tools/size-ratchet; its list now holds only cube.rs 1,641 (was three files). Whole-file `wc -l`: region.rs 1,490, chat.rs 2,391; the tests stay in the files, only the unit tests of moved functions moved with their code (mc 139 tests before and after, voice chat 49 before and after). Counts in reports/debt-2026-11-19.md.

Commits (owner identity, no attribution lines):
1. 91714434 refactor(mc): split region.rs into items, hits and fit
2. 17186311 refactor(voice): split chat.rs into gate and notes
3. da60b6cf ci: lower the size ratchet after the region and chat splits
4. 675be38d test(voice) red, 6c6f776e fix(voice): SB2, a dropped spoken note stays a "you:" line (the mark of the note used up the 160 chars of the "user:" line)
5. 890d952b test(assistant) red (two red, one control green), 7e339880 fix(assistant): UX-D1, a hide or Escape of a list that held text clears the field; docs/hud.md line adjusted

Non-move lines: `mod`, `use` and `pub use` lines, `pub(super)` on moved items, explicit import lists instead of `use super::*`, and the inline `//` comments (hook no-inline-comments) rewritten as `///` lines on the enclosing item or as `reason = "..."` on `#[allow]`. `cfg(test)` re-exports of close_id (mc) and DETAIL_BYTES (voice) for the tests that name them. No code changed.

Gate (head 7e339880, base session branch; MERGE_GATE_RUNS under the scratchpad, jobs 3): red-first, pre-commit, gate-lib, build, test, clippy, ci-check, 2d-guard, package-deps/lists, virtual, xp-faces, xp-paperclip, xp-fields, xp, xp-question, xp-notepad, hud, matrix, layout, chat-keys, tiling-audit, acts, last-words, doors, cube, placed PASS in the first run. construct-hud FAIL there ("light-1920x1080: no claim of the indicator", the known flake of mv1 r2 and mv3). The second run (out, winit, wm-pane, pane-keys, walk, pull, keys PASS) had construct-hud FAIL again, at another cell ("light-1280x720@1.5: no claim after the restart of voice", 35 ok before it). Third run: pixels, mc-paths, mc-sofa, mc-handover, e2e PASS (e2e 295 s). construct-hud then PASSED alone three times in a row (239 s each). So the full list passed in pieces; construct-hud is flaky at 2 of 5 runs here (both under jobs 3), not related to mc, voice or assistant code.

## Refactor proposals (ranked)
1. cube.rs (construct) 1,641 production lines is the last file over the limit: scene/pose helpers to cube/*.rs, S-M.
2. construct-hud claim flake (indicator claim, voice restart): until_ok on the claim reads, S. Seen in 3 lanes now.
3. region.rs still has Mc::regions/update/prepare/switch/region/arrange/stack/line (about 600 lines) that could go to region/layout.rs, S.
4. chat.rs Chat::request/learn/shrink/fit/save (history budget, about 300 lines) to chat/history.rs, S.
5. Parent files re-export moved items (`pub use fit::...`); switch callers (audit.rs, layout.rs, agent.rs) to the child path, S.
