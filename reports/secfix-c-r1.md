# secfix c r1 (sh-26, head cd7a894, branch claude/secfix-c from fe8ba6d, 2026-11-17)

Commits: f7230b9 test (red) | e0129a6 fix: sanitize and guard the voice confirm question |
cd7a894 docs: voice.md (hidden characters refused, own HUD says too long, 400 ms button wait).
Red (f7230b9, all 8 failed): voice mc a_question_emits_no_bidi_or_hidden_character ("Run\u{202e} this?"),
mc a_long_question_keeps_its_keys (cut detail), chat and agents a_detail_with_hidden_characters_is_refused
(question asked / no refusal), chat a_question_too_long_for_the_hud_is_refused, core
a_button_too_soon_after_the_question_counts_as_nothing, chora-mc a_confirmation_shows_no_bidi_character,
construct a_question_shows_no_bidi_character (both drew U+202E). Green: voice 246, mc 126, construct 181 pass.
Fix: chat.rs unshowable() and agents.rs unshowable() refuse a detail that sanitize changes, next to too_long,
with an honest reason to the LLM and a spoken/noted rejection to the user; mc.rs confirm() sanitizes prompt,
detail and name before the HUD and the emit; core.rs too_soon() is the one guard for key (1 s) and
button (BUTTON_DELAY 400 ms), shown_at is reset by show() so a replaced question waits again; mc agent.rs
and construct hud.rs sanitize again (defence in depth). client.rs refusal text names "showed too briefly".
Gate: tests/merge-gate.sh, every step passed on cd7a894 (/tmp/merge-gate.dVzff8). First run failed in
2d-guard: shallow clone, fixed by git fetch --unshallow (not a code issue).
Deviation: mc.rs:351 does not answer no; the reporter thread cannot answer the core, and a 4 KB cap for
every question would refuse the sub-agent edits that chora-mc shows whole (64 KiB cap). The own HUD now
never cuts: it shows "(the command is too long to show here)" between prompt and keys. The red test
a_question_too_long_for_the_hud_is_refused was dropped in e0129a6 for that reason (said in its body).
Open: the spoken prompt of an ACP request (agents.rs asked(): toolCall.title) is still raw for TTS; only
its display is sanitized. A sanitize change by odd spaces (U+00A0 etc.) or > 2 combining marks also
refuses a detail; accepted as strict. Not checked end to end with a real LLM (unit level only).
Self-check: no state added beyond shown_at (set in show(), every path); no SKIP; CI checks ran in ci-check.
Cut: mc and construct use only chora_kit::chat::sanitize; no breach.

## Refactor proposals
- mc agent.rs:97-133 and construct hud.rs:956-1000 -> kit ask::Question and Card (M, ~80 lines, DB6).
- One answer path: mc and construct answer() through Card::press (S).
- chat.rs:168-190 detail() and agents.rs detail()/unshowable() -> one detail builder and one refusal
  with phrases from config (M; the English/German refusal texts now live in two places).
- core.rs Input::ConfirmButton open: Sender<bool> -> an enum (taken, not open, too soon) so the RPC
  error says which (S).
- mc.rs Reporter could get a way to answer the core, so a question that cannot show is a real no (S-M).
