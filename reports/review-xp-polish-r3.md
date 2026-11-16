# Review xp-polish r3 (sonnet, read only)

Base 82c79e8 (merge of c949cc84), head edd00348 (full sha: edd00348 as in `git log`, r3 head; reviewed edd0034)
Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 0 MINOR, 1 LOW, 0 COSMETIC, 4 holds)

| # | sev | where | finding |
|---|-----|-------|---------|
| 1 | holds | tests/merge-gate.sh:9-10, 167, 178 | Merge 82c79e8 clean: "# Steps:" lists xp-question and pane-keys; both step rows present (167, 178); no conflict markers. |
| 2 | holds | voice.rs:345-348 | Cause confirmed: ensure_top set self.top but its commands were returned after chat_draw, so Move hud-top hit a node not yet created. Fix applies the top group first (self.apply(top)) then draws. Complete: the other two ensure_top callers (voice.rs:550 badge, :690 settle) push commands into a local vec and do not call chat_draw before applying; no other chat_draw caller is paired with ensure_top. Cell log (assistant-*.log) of all 5 runs has no "does not exist". |
| 3 | holds | chat.rs:1100-1112 question_of | Sets state "question" only when the open chat is the asking helper (or the question names none); other modes (tiling, mc, construct pane, no chat) unaffected. Next voice.state overwrites it (chat.rs:1083 status diff), so answer or cancel resets via the normal state flow. |
| 4 | holds | tests/gate-xp-question.sh | Cell now polls nodes up to 10 s, fails if hud-card shows without hud-chat-hint, sends the question state only after the card. Test commit 22ed0a0 precedes the fix. Red run at 22ed0a0 not rebuilt here (cost): builder's 3/3 and 9/10, 10/10 reds are unverified by me. |
| 5 | LOW | voice.rs:338 | question_of runs before the output() early return; if no output exists the state stays "question" until the next voice.state. Harmless (no card, hint hidden when no question card). Also the unit test needed question_of(None) added in the fix commit, not the test commit, so 22ed0a0 alone has only the cell as red. |

## Evidence (CARGO_TARGET_DIR=/tmp/rev-target, 19G free at start, deleted after)
- cargo build --workspace ok; cargo test -p chora-assistant: 231 passed, 0 failed.
- tests/gate-xp-question.sh at edd0034: SCALE=1 x3 ok, SCALE=2 x2 ok.
- tests/gate-chat-keys.sh: ok.
- Harness note: the gates need CHORA_BIN as well as BIN, and chora-assistant finds its huds only from <repo>/target/debug, so with a /tmp target I set CHORA_BIN and XDG_DATA_DIRS to a symlink of plugins/chora-assistant/huds (outside the repo). Without that the cell fails with "no HUD folder named neutral" / "chat.show is no exported method" (environment, not code).
- Not run: clippy, full merge-gate.sh, the r2 MINOR (restyle red test) skipped by the builder, not trivial.
