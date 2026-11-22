# split-b

## r1

Head: claude/split-b at 2af7801c (base 3b14c825; origin/claude/autonomous-dev-session-3w-i33pg7 already contained, merge a no-op).

Commits (one per file):
- e07376cc agents.rs: moved to new services/voice/src/agents/text.rs the text helpers (cut, redact, redact_texts, redact_line, title, summary; from lines 276-363) and to agents/permission.rs the Permission struct and the permission-question helpers (question, outcome, is_physical, unshowable, detail, asked, plan_line; from lines 788-795 and 836-965). Visibility pub(super) only; title and summary re-exported pub.
- 2af7801c core.rs: moved to new core/confirm.rs the question and answer methods (ask, confirming, needs_key, answer, always, yes_no, confirm_key, confirm_button, unshowable, too_soon, answered) and to core/frames.rs (played_level, sounding, barge_in, frame, interim), each an `impl Core` in a child module, methods pub(super).

Production lines before and after: core.rs 1420 to 1136; agents.rs 1410 to 1189. Largest file in the tree stays under 1500; tools/size-ratchet ok. cargo fmt reflowed a few long signatures after the `pub(super)` prefix; no body changed. Tests stay in place in core.rs and agents.rs (they reach the moved items through `use super::*`).

Checks: cargo clippy -p chora-voice --all-targets clean; cargo test -p chora-voice 257 passed + 1.

Gate: `tests/merge-gate.sh --jobs 1` (full) ran 39 cells, all PASS, then the harness 2 h cap killed it inside `keys` (no red cell). The cells after `keys` (keys, pixels, mc-sofa, mc-column, mc-handover, launcher-click) did not run. Run separately with --only: mc-paths, voice-acp, voice-events, agent, e2e, all PASS. So the full gate is NOT completed in one run; the PM should run the remaining cells (keys, pixels, mc-sofa, mc-column, mc-handover, launcher-click) or the whole gate on a longer budget.
