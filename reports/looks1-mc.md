

## r1

Branch claude/looks1-mc, head b0453e19 (commits a3037cb2 contrast + test, 31f53dc9 words, b0453e19 gate-script string), merged with origin/claude/autonomous-dev-session-3w-i33pg7 at 7a6c0eb8 (it had not moved).

Rows: all 10 mc rows deleted, none kept. known-red count 189 -> 179 (`grep -c '^\[\[red' tests/acts/known-red.toml`).
- I6 contrast (7 rows, a3037cb2): the doors and the chat head were text nodes under the chat box or under the card of a question (contrast 1.00 on the capture) -> `door_items` gives none while the chat is open, `chat_items` none while a helper asks (card stands over it); idle and asleep signs in `muted`, not `faint` (1.64:1). Unit test the_doors_leave_while_the_chat_is_open.
- I10 words (3 rows, 31f53dc9): scene title, header head, footer line and the HUD line "Agents  ·" now say Helpers (scenes/agents.yaml, agent.rs, docs/configuration.md, manage.rs tests). b0453e19: tests/gate-mc-paths.sh named "→ 3  Agents"; my rename broke that cell (mc-paths mouse-back), fixed.
- Red first: with the 10 rows deleted the mc acts were red (2 blocking: I10 footer-next, I6 agents-0-sign, the rest on the same cells) before the fix, green (43 pass, 0 blocking) after.

Captures (media/2026-11-22/looks1-mc/): contrast: before-the_chat_of_the_helper_opens.png / after-the_chat_of_the_helper_opens.png, before-the_helper_asks.png / after-the_helper_asks.png; words: before-a_second_Right.png / after-a_second_Right.png. (The doors and the chat head were invisible in the old capture too; the fix removes the hidden nodes, the picture itself is nearly the same.)

Gate: cargo test -p chora-mc 190 ok; clippy -D warnings ok; pytest tests/test_acts.py 219 ok. Full merge-gate --jobs 1: run 1 (head 31f53dc9) green to pixels, mc-paths red (mine, fixed in b0453e19). Run 2 (head b0453e19) green through doors, then acts RED, foreign: construct I6 "PANES" (construct-help-body-head-6) 11 px vs 14 px, and construct act "the hint Keys of a face names a key that opens the key list": "RuntimeError: the hint keys names no key" (/tmp/merge-gate.gyZP1N/acts.log lines 475-477). Run 1's acts on the same construct code passed (1494 s), so it looks flaky; not rerun to green. The steps after acts then ran with --only on b0453e19 (mc-paths mc-sofa mc-column mc-handover launcher-click voice-acp voice-events agent e2e): all PASS. last-words..pixels passed in run 1 (before the one-line script change). The gate is therefore not fully green: acts blocked by the foreign construct cells.
