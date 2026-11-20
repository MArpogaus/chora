# confirm-guard

## r1

Result: no code change; the premise of sh-26 no longer holds on `claude/confirm-guard` (79c8d2b5). No red commit, no fix commit, no gate run (nothing changed).

1. 400 ms guard in the mc and construct confirm cards: neither plugin draws a confirm card any more. d86ee4ca (mc) and 6a5cfc1d (construct), "chora-assistant is the only voice presenter", removed them. mc `follow("voice.confirm")` only keeps its helper line (`question_line`, text without buttons; test `the_host_draws_the_voice_status`); construct `follow` only sets `asked` to hide the hints. Neither has a Yes click, key or voice path that could answer. The one card is chora-assistant's, built on the kit `ask::Card`, whose `press`/`click` already refuse inside `GUARD` (400 ms) after the card shows or moves; voice keeps `BUTTON_DELAY` and `KEY_DELAY`. Kit tests: `a_click_inside_400_ms_of_the_card_is_refused`; assistant card cells at card.rs:1419, 1569 (`Outcome::TooSoon`). A kit guard "to put in once" is already there; a red test would have to invent a card.
2. Sanitize half: `Question::from_event` runs prompt, detail and keys through `chat::sanitize`. Pinned already by `ask::tests::hidden_and_reordering_characters_never_reach_the_card` (U+202E, U+200B, ESC in prompt and detail) and `a_hidden_character_in_a_key_never_reaches_the_card`. A duplicate test I wrote was dropped.
3. sh-26 row: not changed to `fixed`; no fix sha exists. Suggest the PM close it as `stale: fixed by d86ee4ca, 6a5cfc1d, aeb13ece (kit ask is the only question model)`, citing the tests above. Its line cites (mc agent.rs:693, construct hud.rs:1074) are not confirm code on this branch.

Step 0: `git fetch --unshallow` and `cargo check --workspace` pass. Side note: `cargo test -p chora-kit --test perform` does not compile on the unmodified branch (`chora_kit::testing` is gated behind the `testing` feature); not touched.
