# xp-idle

## r1

Branch claude/xp-idle: red cc3c170e, green db7a7acd (on bd5af4b3). Base already contains origin/claude/autonomous-dev-session-3w-i33pg7 (6b01c976).

Cause (test race, not product): gate-xp-chat-idle.sh started the assistant in the background and sent `voice.state` with only a bare `sleep 1.5` between. The assistant subscribes to `voice.*` inside `Runtime::start` (plugins/chora-assistant/src/main.rs:392); a named-helper state sent before that is lost, and its retry `ask_state` (voice.rs:122) asks the fake voice, which answers only a nameless state (`{}`), so the header keeps "No helper". Load alone (4 and 16 x `yes`) did not reproduce in 12 runs; an assistant started 0.8 s late does, deterministically. Same bare-sleep shape at gate-xp-helper.sh:90 and :98 (xh-1).

Red (cc3c170e): cell starts the assistant after 0.8 s: `FAIL: the header does not say Working while the helper works: [{"id":"hud",...`. Green (db7a7acd): the cell polls `state.get` of assistant until linked, then emits the state; waits for Working, then for Idle with no Working or step; the 0.8 s late start stays as a permanent stress. xp29-click and xp29-none wait on hud-figure before the click. No bare sleep is the only wait, bar unchanged.

Counts under 4 x `yes`: xp-chat-idle 10/10, gate-xp-helper.sh (all cells) 10/10.

Gate: `tests/merge-gate.sh --jobs 2` on db7a7acd: 53 PASS, 1 FAIL, EXIT 1. xp-chat-idle PASS, xp-helper PASS. Foreign red: mc-scale: `FAIL: settings.set of the write time beside 300 panels takes 5118 ms, more than 5000 ms` (a timing bound, no xp code; not rerun).
