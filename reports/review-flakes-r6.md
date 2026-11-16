# review: flakes-r6 (e3fb7d46, base 6a067e39)

Reviewer: independent, read-only. Date 2026-11-16.

## Verdict

MERGE-OK (no open BLOCK or MAJOR; 0 BLOCK, 0 MAJOR, 2 MINOR, 2 NIT).

## Findings

- rv-1 MINOR plugins/chora-assistant/src/chat.rs:1067-1075 (`question_asked`). F28 fix is minimal and
  correct: the list (Mode::List, `helper()` None) never holds Enter; chats still do. Esc, close, card
  and Enter-in-chat paths unchanged (`question_asked` still returns Where as before; open still sets the
  hold from the chat helper). Residual, already in the report: a chat of helper B holds Enter while helper
  A asks. Not a regression. Evidence: `cargo test` 2147 pass; gate-chat-keys.sh rc 0 (two-helper step ok).
- rv-2 MINOR crates/chora/src/words.rs:203-210. F17 is not a fix: not reproduced, only the stuck line
  now names the state (the `?` fallback and `state_of` refactor are sound, test asserts " ms in the state
  S"). Proposal "close F17" rests on 0 of 385 runs; keep it open (or close with the rule "reopen on a line
  with the state") until one sighting on a post-4a5ddb40 tree.
- rv-3 NIT plugins/chora-assistant/src/chat.rs:868-876. F26 fix: `closed` of the field now runs `hide`
  (sends `link.hide()` to a field that already hid itself on Esc); idempotent, closes the race where a late
  `place` re-shows it. Cause is plausible (capture + event stream + log line, read in the report) and the
  fix is smaller than the old `close_view` (removed, one path). Not reproduced here (needs cache drops);
  doors gate 3 of 3 incl. mc screenshot-door and volume-door.
- rv-4 NIT report and brief name commits `bdc2184f` and `bb2adbd5`; the real ones are `bdc2184a` and
  `bb2adbd6` (`git log`). Red is first in each pair (9bff0429 < 3e7e400d, bdc2184a < bb2adbd6,
  80ead5b5 < 4c27ea0f). Red commits not re-run by me (read-only, one build at a time); the F28 red
  commit carries the gate change (FAKE_ACP_ASK_AFTER) that the report says failed 3 of 3.
- Test change e3fb7d46 (tests/doors/tiling.sh:49 `keys_open` also needs `.opacity == 1`): honest. It
  waits for the state the key list is really open in (per main.rs `toggle_keys`), no timeout touched, the
  comment says why. It does not hide a product defect more than the report already lists (an Escape
  within the measuring step is lost; open, proposed MINOR). Cell passes 3 of 3 here. No red commit
  (gate only), as the report says.

## Commands run

All in /home/user/wt/rev-flakes, CARGO_TARGET_DIR inside it, one at a time.

| Command | Result |
|---|---|
| `cargo build --locked`, then `--workspace --locked` | ok |
| `cargo test --workspace --locked` (covers chora-assistant, chora) | 2147 passed, 0 failed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | clean |
| `BIN=target/debug tests/gate-chat-keys.sh` | rc 0, all steps ok |
| `BIN=target/debug tests/gate-doors.sh` x3 | 3 of 3 rc 0; 20 of 20 rows pass in each run (xp, tiling, mc, construct); about 335 s each, no load |
| Not run: full merge-gate.sh, pre-commit, ci-local.sh, the red commits, loaded runs (no cache drops) |

## Conflicts

Trial `git merge --no-commit --no-ff 208c657a` (train30 head): no conflicts (auto-merged
plugins/chora-assistant/src/chat.rs); no "# Steps:" conflict (this branch does not touch
tests/merge-gate.sh). Aborted. The merged tree was not built or tested: the train gate must cover
chat.rs after the merge (train30 also holds tiling-look r1-r2 and the xp card hint from aba97fec; the
chat-keys gate was run here only on the 6a067e39 base).
