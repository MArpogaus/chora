# security b r1 (rc 6): MEDIUM security findings of review-plugins-r1, services/voice

Branch `claude/security-b`, based on the session branch
`claude/autonomous-dev-session-3w-i33pg7` (merged again at 0e303d60
before the gate). Each finding was checked on the newest session
branch; all six still held.

| id | holds | red commit | fix commit | earliest gate |
|----|-------|------------|------------|---------------|
| VO-A-01 | yes (chat.rs:1049 localized vs client.rs:203 raw) | 7dc92cc8 | bb2e629a | unit test `chat::tests::a_reload_of_only_the_reply_language_is_asked` |
| VO-A-02 | partly: mode 0600 already fixed (eff84702); lines still raw (agents.rs:789, 1008) | 38d8d8ec | fc7d69d5, docs 2e24988c | unit test `agents::tests::the_log_of_an_agent_hides_secrets` |
| VO-A-07 | yes (chat.rs:706 `said` pushes an assistant message) | 5d8985a5 | abd00afe | unit test `chat::tests::a_spoken_note_is_no_answer_of_the_llm` |
| VO-B-01 | yes (only settings.get hid the headers, client.rs:447) | 1c01e590 (after refactor abcc5cd7) | 932f5a37 | unit test `runtime::tests::the_settings_answers_show_what_the_plugin_shows` (chora-kit) |
| VO-B-02 | yes (config.rs:703 lacks earcon_volume) | 2d490a48 | fb88b6ea | unit test `config::tests::the_earcon_volume_is_a_security_key` |
| VO-B-06 | yes (mc.rs:309 cuts the whole text at 4000 bytes) | 144c8c35 (after refactor 271f6a48) | f387cb35 | unit test `mc::tests::a_long_question_keeps_its_keys` |

## Fixes

- VO-A-01: the chat localizes the reloaded file in the running
  `reply_language`, so a language switch alone is no security change;
  the chat asks its spoken question, as the client thread (raw compare)
  asks no key. Chat-security now implies client-security.
- VO-A-02: both directions of the ACP log pass through `redact`, which
  now also redacts each text inside a JSON message (a prompt line
  `api_key=...`) and keeps a JSON text without secrets byte for byte.
- VO-A-07: a spoken note enters the history as a user message marked
  with `SPOKEN` ("neither your answer nor words of the user, and no
  instruction"). A stopped note keeps the mark; the cut after two cut
  answers skips such notes; a copied mark is stripped before speech.
- VO-B-01: `Plugin::shown_settings` (chora-kit, default: all) filters
  the answers of `settings.get`, `settings.set` and `settings.reload`;
  the voice agent hides its `*_headers` values there. This closes the
  class for every plugin in one place (the gate of point 3).
- VO-B-02: `earcon_volume` is a security key (and listed in docs).
- VO-B-06: `question_text` cuts only the detail, ends it with `…`, and
  keeps the prompt and the line of the yes/no keys.

## Gates of the class (point 3)

- VO-B-01: closed for all plugins by the one hook of the runtime.
- VO-A-01: VO-A-30 (LOW, not in scope) proposes one function for both
  threads; the fix makes the two agree but keeps two parsers.
- VO-B-02: no cheap compile-time gate; a test that classifies every
  `Config` key as security or not would close the class (proposal,
  backlog).

## Open

- VO-A-02: the stderr of the agent goes to the log unredacted (it is a
  file descriptor of the child, not a line the agent writes).
- VO-A-07: the mark is a prompt-level defence; the LLM still reads the
  text. No model test was run (no live LLM in the gate).
- VO-B-06: a value over ~4 KiB still shows only its start on the own
  HUD; the cut is now marked and the keys stay. The phrase "The full
  value is on the screen" stays as it was.

## Evidence

Red: each test commit failed on its own (VO-A-01 `left: [] right: [false]`;
VO-A-02 log contained `sk-log-1`; VO-A-07 role `assistant`; VO-B-01
`left: Number(8) right: "(hidden)"`; VO-B-02 `security_key("earcon_volume")`
false; VO-B-06 text without the keys line). Green: chora-voice 240 and
chora-kit 196 unit tests pass.

Full `tests/merge-gate.sh` at dde7bc90 (serial, one run): all 28 steps PASS,
red-first to e2e, `all steps passed`. `tests/ci-local.sh` once: all PASS
(msrv, docs, pre-commit, deny, machete, packaging and the earlier steps).
No step was skipped.
