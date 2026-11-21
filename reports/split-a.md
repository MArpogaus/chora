## r1

Lane claude/split-a, base 892d035c (merged with origin/claude/autonomous-dev-session-3w-i33pg7, which had nothing newer). Head a3c14590.

| sha | change |
| --- | --- |
| b59aeeae | mc: `impl Mc` block of the region, event and navigation methods (main.rs 873-1194, 322 lines) moved to plugins/chora-mc/src/methods.rs |
| a3c14590 | assistant: `impl Session` block from `voice_event` to `pull` (chat.rs 1042-1411, 370 lines) moved to plugins/chora-assistant/src/chat/dialogue.rs |

Production lines (size-ratchet rule): mc main.rs 1,432 to 1,112; assistant chat.rs 1,419 to 1,049. Both at most 1,200.

Pure moves: no item renamed, no body changed. Visibility widened only as the move needs: `pub(crate)` on emitted, name_theme, page_key, press, region_method, sync_keys, take_free_keyboard, voice_changed, follow (mc); `pub(super)` on label and helper_state (assistant). Imports adjusted (an explicit `use`, no wildcard; MAX_LINES moved from main.rs imports to methods.rs; Line and Measure imported in the chat.rs test module).

Checks: cargo test -p chora-mc 189 ok, -p chora-assistant 243 ok; clippy --all-targets clean; tools/size-ratchet ok (list empty, nothing to lower); lint-ratchet ok (allow 360, no-reason 0, eprintln 131).

Gate: `tests/merge-gate.sh --jobs 1` (no --only) all steps passed, 49 PASS, 0 red.

## Review r1 (PM, 2026-11-21 20:38 UTC; builder: lane split-a)
Range 892d035c..a3c14590, 2 commits (b59aeeae mc, a3c14590 assistant), owner identity, no stray commit. `git diff --color-moved=plain`: every non-moved line is an import, a `mod` line, a doc header of the new module, a visibility change (pub(crate) in mc methods.rs, pub(super) in assistant chat/dialogue.rs) or a rustfmt reflow of a moved signature; no body changed. Lane gate 49/49 green on the merged tree. 0 BLOCK, 0 MAJOR. Verdict: MERGE-OK; joins t104.
