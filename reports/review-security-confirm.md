# Security review: confirm text in mc and construct (opus, read-only, fe8ba6d0, 11-17 04:53)

Verdict: MAJOR (code read, not reproduced end to end). Finding sh-26.

1. Reach: only the voice connection emits voice.confirm (mc main.rs:1135, construct input.rs:186; D65). The text is steerable by prompt injection: voice LLM tool calls (chat.rs:181 joins run_program argv raw; JSON Display keeps bidi/zero-width) and ACP sub-agent permission requests (agents.rs:1202, :879 detail(), :893 asked(): toolCall.title raw). Default voice_hud = Own, so the default install uses this path.
   Path: core.rs:670 Effect::Confirm -> main.rs:533 -> mc.rs:189 confirm() -> :205 fallback HUD (raw), :208/:220 emit raw -> mc main.rs:1103 -> agent.rs:663/692 set_text raw; construct voice.rs:40 -> hud.rs:1059-1074 -> :1258/:1266. cosmic-text applies bidi: U+202E reverses on screen.
2. No sanitize on this path; kit ask.rs:67 Question::from_event uses chat::sanitize (chat.rs:98); only chora-assistant uses it. xp and tiling do not draw voice.confirm.
3. Keys guarded (client.rs:805 drops injected; core.rs:1135 KEY_DELAY 1 s). Buttons: injected refused (agent.rs:145, hud.rs:1357) but no time guard; core.rs:1146 confirm_button checks only shown == id. A sub-agent can replace the question at any time; the click meant for the old one answers the new one.
4. MAJOR: reachable in default config, shown command can differ from the one that runs. Not BLOCK: needs model output plus a user click.
5. Fix: (a) sanitize prompt, detail, name in voice mc.rs:189 confirm(); refuse a tool call when sanitize(detail) != detail (next to too_long, chat.rs:1162, agents.rs:1203); defence in depth in mc agent.rs:663 and construct hud.rs:1059. (b) button guard on shown_at in core.rs confirm_button. mc.rs:351 fallback truncates the detail: refuse instead.
   Red tests: voice mc.rs a_question_emits_no_bidi_or_hidden_character; agents.rs/chat.rs a_detail_with_hidden_characters_is_refused; chora-mc a_confirmation_shows_no_bidi_character; construct hud.rs a_question_shows_no_bidi_character; voice core.rs a_button_too_soon_after_the_question_counts_as_nothing.

Refactor proposals: mc agent.rs:97-133 and construct hud.rs:956-1000 -> kit ask::Question and Card (M, about 80 lines); both answer() through Card::press (S); core.rs:1131-1157 one guard for key and button (S); chat.rs:168-190 and agents.rs:879 one detail builder (M); mc.rs:351 refuse instead of truncate (S).
