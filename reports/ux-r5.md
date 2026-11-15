# ux r5 (rc 1): mc-31, sh-19, sh-20

Lane: opus, cloud, branch claude/ux-r5 from 0e303d60. Pushed at 52249372 (merge of the harness train
2d9f306f), not merged. Full `tests/merge-gate.sh` on 147c4845 and again on 52249372: 28 steps PASS, exit 0 (e2e: the
five listed skips of tests/e2e-skips.txt: session-units, fkeys option, screencast, x11, xp-token).
`tests/ci-local.sh`: msrv, docs, pre-commit, deny, machete, packaging PASS (no SKIP).

| Item | Red commit | Fix commit | Cell | State |
|---|---|---|---|---|
| mc-31 w16 step 1 | — (not a product bug) | 08ffcdc8 (test: script) | w16 step 1 | not real; script fixed |
| sh-19 header jump | 477610c9 (unit), 1778cc81 (cell) | 8246eba2 | unit `stop_and_close_keep_their_row_for_no_task_a_short_task_and_a_long_one`; gate-hud `chat-long-task` | fixed |
| sh-20 capped cost | 6c9da5ef | 147c4845 | unit `a_long_paste_without_spaces_is_measured_only_for_the_lines_shown`; chora-draw `wrap_most_gives_the_first_lines_of_wrap` | fixed |

## mc-31: not real (the battle script was stale)
Reproduced on 0e303d60: w16 fails from step 1 on, as logging-r2 says. Cause, two script assumptions that the
product changed on purpose:
- mc draws no voice question by default: `voice_hud: host` since 81f63cd1 (plugins/chora-mc/mc.yaml:16,
  plugins/chora-mc/src/main.rs:1070-1077); chora-assistant draws it with `hud-card-yes`/`hud-card-no`
  (gate-hud `card-names-file`). w16 runs no chora-assistant, so no No button anywhere.
- An injected click answers no question (plugins/chora-mc/src/agent.rs:145-147, manage.rs:196; the host card
  the same, gate-hud `card-injected-click-refused`).
The "lag" follows: the physical question stayed open, `Run true.` was an unclear answer, the next request
declined it, and every scripted LLM turn of the fake queue then answered the step before.
The person can always say no: the No button of the host card (or of mc with `voice_hud: own`), Alt+n, or a
spoken "no" after the question (core.rs:1115-1126). Answers do not go to the wrong question in the product.
Fix 08ffcdc8: step 1 sets `voice_hud: own` in mc, needs mc's No button, needs an injected click to answer
nothing (question stays), waits until the question is spoken (a "no" over its own speech is an echo,
core.rs:1091), declines by voice and needs the button gone. The speech cut on a press is the unit
`an_answer_cancels_the_speech_of_the_question` (services/voice/src/core.rs:2328).
Runs: before, every step 1-6 fails; after (08ffcdc8), steps 1, 5, 6 pass, no lag.
Open (not in this brief): w16 steps 2, 4, 4b press Alt+y by `input.inject`, which voice never takes as an
answer (services/voice/src/client.rs:796-803, by design), so they cannot pass headless; step 3 (push to
talk: "the state record does not tell push_to_talk", speech without the key reached the ASR) fails for
another reason, not investigated. Both are finding candidates for the PM. w16 is not in the gate.

## sh-19: fixed band of two header lines
`tops` (plugins/chora-assistant/src/chat/view.rs:530) always reserves `HEADER_LINES` in plain and bubble;
glass and xp had fixed bands already. One transcript row less under a one-line header (accepted).
docs/hud.md says it. Red: unit `Plain None [0, 0, 1280, 770]: [(40, 74), (40, 74), (40, 74), (60, 94)]`
(no task, short task, short task with (3/5), 1800 chars); gate-hud `chat-long-task` on the pre-fix
build: missing, Stop `[1072,72]` with the 1800-char task, `[1072,52]` with `Tidy notes`. Green: 222/222
units, both cells pass, Stop/Close `[1072,72]`/`[1166,72]` in every state.
Gate gap: `voice.state` with `task: ""` keeps the shown task in the host, so the cell's no-task step shows
`Tidy notes` again; the no-task case rests on the unit.

## sh-20: wrap only the lines that show
`chora_draw::text::wrap_most` stops once it has the asked lines (each line equals that of `wrap`; unit over
broken words, paragraphs, empty lines, `most` 0..n+1); `capped` asks for `most + 1`. Glass goes through
`capped` too. Red: 20,033 width calls for 20,000 chars without spaces; green: under 500 (the bound of the
unit; about 90 with a 7 px font at 200 px). No timing measured in release: the brief allows the bound.

## Self-check
No flag, grab, timer or mode added. No skip apart from the five listed e2e skips. Commit types: fix only
after a red test commit (red-first PASS). CHANGELOG untouched. Docs: docs/hud.md; no glossary or nav change.

## Captures at 1x (media/2026-11-15/ux-r5/)
chat-long-task-1800.png and chat-long-task-short.png (plain: Stop and Close at y 72 for both, a free second
header line for the short task), chat-long-task-short-before.png (pre-fix: Stop at y 52),
construct-chat-long-task-short.png (glass: unchanged band).
