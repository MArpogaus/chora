# xp-busy

## r1

F7 (rc 1 shared bar "honest states"): right after a send the xp chat header read "Idle" and Stop was grey until the helper's first state, so a sent request showed no busy state (tour-v9-judge r2b x5d..x6d; xp-v2.md build list 3).

- Base 3e3b1b2db (session head) plus the xp-blink r3 head 3fd3a3bba (merged `--no-ff`, 83d106f3c); head 88951ae4d on claude/xp-busy-r1.
- Where: the chat session is in `plugins/chora-assistant/src/chat.rs` (the brief named plugins/chora-xp/src/chat/, which does not exist); no chora-xp, no chora-kit change.
- Red 84a8565da before fix 88951ae4d: unit test `a_send_shows_working_and_stop_at_once_until_the_helper_answers_xp_busy` fails on the base (`left: ("Atlas — Idle", "")  right: ("Atlas — Working", "")`), and the gate case in tests/gate-xp-chat-idle.sh (a fake voice whose first state comes 2 s after the say, new `FAKE_VOICE_LATE`): `FAIL: xp-busy: the header says no Working within 200 ms of the send, while the helper has said nothing yet`. Two more unit tests pass on both sides on purpose: a refused send shows the failure and no Stop, a send without a helper keeps "No helper is set up" and is not busy.
- Fix: `Session::said` calls `busy_until_the_helper_says` after a send voice took: the state is "working" (task and step cleared) unless a prompt already runs; the first `voice.state` of the helper replaces it. A refused send keeps the failure line (`say_failed`) and no Stop. No option, no timer; the kit's State words unchanged.
- Diffstat vs the merge 83d106f3c: 3 files, 155 insertions, 6 deletions (chora-assistant/src/chat.rs +77, tests/fake-voice.py, tests/gate-xp-chat-idle.sh). The gate's poll loops share one `wait_until`, so the sleep count stays 778/929 (no ratchet change).
- State at send +1 frame (first poll after the injected Enter, 75 ms alone, 145 ms in the full gate): header "atlas — Working", Stop input `accept` (live). After the answer: "atlas — Idle", Stop input `pass` (grey). Before the send: Idle and `pass`.
- Captures (crops 500x400 of the 1280x800 session, 49 KB and 47 KB): media/2026-11-26/xp-busy-r1/chat-just-after-send.png ("atlas — Working", Stop and the sent line "hi"), chat-after-answer.png.
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (full, no step filter): `all steps passed; no --push, nothing is pushed`, 0 FAIL lines; xp-chat-idle PASS in it. No `chora --headless` left.
- Note: the gate poll bound is 200 ms back to back; under the full gate load the busy line showed at 145 ms, so a heavier host may need a look (no timer in the product, only the test's bound).

## Review r1 (PM, 2026-11-26, 155 lines, 83d106f3..88951ae4)
- Red first: 84a8565d (unit tests + gate case FAKE_VOICE_LATE, base log "FAIL: xp-busy: the header says no Working within 200 ms of the send") before fix 88951ae4. Holds.
- Fix: Session::said sets state "working" after a taken send (not after a refused one, not without a helper); the next voice.state replaces it. KISS: no timer, no option, kit State words unchanged. Holds.
- Tests: Working and Stop at once, Idle and grey after the answer; refused send shows the failure, not Working; no helper keeps "No helper is set up". Holds.
- Scope: the chat lives in plugins/chora-assistant/src/chat.rs (shared by all looks), not chora-xp/src/chat/ as the brief said (PM brief error). Effect: every look's chat shows Working at a send: the shared bar "honest states", wanted.
- Gate: full, all steps passed, no foreign flake; captures 49 KB, 47 KB.
Findings:
- rxbu1-1 (NIT): a voice.state idle already queued before the send would set Idle again before the helper starts; states carry seq, a later round may ignore a state older than the send.
Verdict (computed): no open MAJOR: MERGE-OK 88951ae4, joins train 152.
