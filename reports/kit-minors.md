# kit-minors

## r1

Lane: cloud, branch claude/kit-minors from 219d5afe, head 88f441e4 (pushed).
Gate at 88f441e4: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test`
all PASS (pre-commit 4s, build 123s, test 187s, ci-check 77s; run
/tmp/merge-gate.FakA8X).

| Finding | Red | Fix | Test |
|---|---|---|---|
| kit5 r2-1 (notify main.rs:246, restart loop) | 1c80f7d7 | 8348e8c1: `another_serves(err)` ends 0 on NAME_TAKEN of the core name alone | `a_second_notify_ends_quietly_on_the_core_name_alone` (chora-notify) |
| kit5 r2-2 (mc agent.rs:50, critical ids uncapped) | 0886713f | af8a176f: `CRITICAL_NOTICES` = 64, oldest first | `the_critical_ids_are_capped_at_one_more` (65 ids, 64 kept; chora-mc) |
| kit7 helper.rs:195 answered -> working | 11267d2a (green at once, a gap test) | none needed | `the_answer_of_a_helper_holds_while_voice_listens_again` now asserts Working after `coder done`, `coder working` |
| kit7 helper.rs:205 new utterance clears answered | 11267d2a (red) | 2d6808f2: voice recording or transcribing sets the answered helpers to idle; thinking alone does not | `a_new_utterance_clears_the_answer_of_a_helper` (chora-kit) |
| kit7 helper.rs:142 seed() report order | 952836f7 (red) | 88f441e4: voice core lists `agents` in report order (the kit seed already keeps the order it is given) | `the_agents_of_the_record_keep_the_order_of_their_reports` (chora-voice), `a_late_reader_keeps_the_order_of_the_reports` (chora-kit) |
| nit docs/api.md rewrap | docs only | 95858506 (also 2d6808f2 names the utterance rule) | none |

Not done: nit kit5 r2-3 (card.rs:274, card over a chat wider than the room beside it):
a fix needs the output size in `aside()` and a vertical-fallback rule, not small; the
reviewer allows leaving it for a 400 px chat panel. Also open: kit7 "no voice-alone cell
per look" (not in the brief).
Note: thinking of the voice agent alone keeps an answer (the existing cell says so); only
the person's own utterance (recording, transcribing) clears it.
