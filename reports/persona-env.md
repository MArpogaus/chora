# persona-env

## r1
Branch claude/persona-env, head 370495ad (from a2f50532; the merge of claude/autonomous-dev-session-3w-i33pg7 was already up to date).
Red commit 6814eebd (selftest cells), change commits 3b5293a7 and 370495ad (follow-up: Notepad by setting, lock logs only, README).

| Item | Selftest cell (tests/persona-selftest.sh) | Change |
|---|---|---|
| 1 shared scripts and recorded stand-ins | "shared scripts installed under the session, lock leaves a mark" | session.sh: CHORA_SHARED_DIR is a copy of flavors/shared under the session dir (the round.sh snapshot has no flavors/ beside the binary, so xp said "not installed"); `swaylock` stand-in logs; check.py T4:lock accepts `swaylock` or `loginctl lock-session` (power.sh runs `swaylock -f`) |
| 2 GUI editor | "mousepad stand-in starts a window Untitled - Notepad" (click Start, Notepad) | chora-probe `--title`; `mousepad` stand-in in the session PATH; the session xp.yaml sets `editor: mousepad` (lowriter is installed here and comes first in the chain) |
| 3 probe window | "the probe window has a title bar and a drag moves it" | `persona-probe` stand-in (chora-probe window, `--server-deco`, title "Probe window") |
| 4 helper answers | "the helper works on hello, rests and answers" | fake-voice.py: FAKE_VOICE_ANSWER makes `atlas` answer `agent_say` (user entry, working 3 s, agent entry, idle); `atlas` is in `agents` of `state`, and reports idle 2 s after start |
| 5 selftest | all four cells, red first | tests/persona-selftest.sh |

Decisions: the real swaylock is not run any more (loginctl stand-in too): a real lock makes `screen.capture` fail ("the session is locked") and the checker cannot read the end state. The persona sees no lock screen; T4 lock measures the call.
Gate: merge-gate --only pre-commit,ci-check,build,test,clippy,gate-lib all PASS (before the follow-up commit, which touches only tests/persona*); `bash tests/persona-selftest.sh` PASS on 370495ad.
Smoke (round.sh snapshot, xp, scripted driver, no persona): T1 pass, T4:lock pass, X2 pass, T3 pass, breaches 0 each. First scripted try failed on my own driver (logical pixels after a look) and on the lock/lowriter points above.
Not done: `tests/persona/smoke.sh` itself was not run (it is the tiling driver smoke); the xp smoke is the scripted run above, not a committed script.

## PM review r1 (2026-11-20 00:57 UTC)
Head 370495ad, own diff a2f50532..370495ad: 6 test files, +143 -13. Red 6814eebd before 3b5293a7 and 370495ad.
Checked: session.sh stand-ins log only (swaylock, loginctl), mousepad and the probe are chora-probe windows, shared flavor copied, editor set to mousepad in the session copy of xp.yaml only; check.py lock accepts swaylock (power.sh calls it); fake-voice answers by FAKE_VOICE_ANSWER; no product code.
Findings: none BLOCK or MAJOR. Note (MINOR): the xp smoke driver is not committed; P2 r2 runs it through round.sh.
Verdict: MERGE-OK; joins train 80.
