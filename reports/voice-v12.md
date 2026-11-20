# voice-v12

## r1

Branch `claude/voice-v12` from 79c8d2b5; the session branch had nothing new to merge (merge was "Already up to date"). No change to services/voice, mc or core: two scripts asserted old design, one passes.

### voice-acp.sh
- Red on base: `FAIL: mc shows coder as question, not done`; voice log `confirm: a yes without the key counts as nothing`.
- Cause: script wrong. The fake agent asks to edit a file (kind `edit`); docs/voice.md section 18 "Questions": such a question takes a key or a HUD button, and "a yes of `voice.confirm` count[s] as nothing". The service did what the design says.
- Fix (128a8660): the script now checks that `voice confirm yes` leaves the agent at `question`, then answers like a HUD: new tests/voice-hud.py connects as `assistant` (in `indicator_clients`), claims the indicator, and calls `confirm.answer {id, yes}` once the question showed more than 400 ms.
- Cell: `voice-acp` (ed4c7bf9), in the default set. 3/3 green before the gate, plus the gate run.

### voice-events.sh
- Red on base: `FAIL: after voice sleep, mc shows voice as none`.
- Cause: script wrong. docs/configuration.md ("The HUD host chora-assistant draws the voice HUD of mc"): mc takes no state of the voice agent itself and shows only helpers; mc unit test `the_host_draws_the_voice_status` asserts `!mc.agents.any(name == "voice")`. D65 text in decisions.md predates this.
- Fix (17e65d25): the script reads `voice.state` with `choractl events` (asleep after sleep, listening after wake) and asserts mc shows no agent `voice`.
- Cell: `voice-events`, default set. 3/3 green plus the gate run.

### agent.sh step 2
- Passes on base: 3/3 runs, steps 1 to 23, `PASS (logs and screenshots in ...)`. No red here, so no fix. The V12 text `xs_are 0 640` is not reproduced.
- Likely cause of the old report (not proven): the expected positions are `14 647` (default gaps); `0 640` is the no-gap layout and is still the example in the comment above `xs_are`. A stale chora-tiling binary or a gap setting from the host would give it. Not touched.
- Cell: `agent` (ed4c7bf9), default set; uses its own temp out dir.

### Gate
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,voice-acp,voice-events,agent,matrix`: all PASS (pre-commit 1s, gate-lib 66s, build 24s, test 196s, clippy 16s, voice-acp 12s, voice-events 1s, agent 42s, ci-check 105s, matrix 270s).

### Notes
- The pre-commit hook no-inline-comments rejected the first commit attempt; reasons moved into the script headers. Commits: 128a8660, 17e65d25, ed4c7bf9.
- A failed commit leaves its files staged and the next commit takes them; check `git status` after a hook failure.

## PM review r1 (2026-11-20, PM is not the builder)
Range: merge-base..ed4c7bf9 (128a8660, 17e65d25, ed4c7bf9), 4 files +141 -8, tests only (merge-gate.sh, voice-acp.sh, voice-events.sh, new voice-hud.py). Harness branch: own train.
Checks: both scripts were wrong against the docs, not the product: voice.md section 18 (an edit question takes a key or HUD button) and docs/configuration.md (the HUD host draws the voice state, mc unit test the_host_draws_the_voice_status). No product change, so no red-first fix commit applies; each script asserts the documented behavior plus its negative (a bare yes leaves the question open; mc shows no agent voice).
- rv-1 NIT: voice-hud.py answers after a fixed 1.2 s pause (the 400 ms button rule plus margin); a slow host could still race. 3/3 green; watch in flakes.
- rv-2 NOTE: agent.sh step 2 passes on base 3/3; V12 "xs_are 0 640" not reproduced. Close V12 for agent.sh as stale; the new agent cell guards it.
Verdict (computed): no open MAJOR: MERGE-OK ed4c7bf9; own train (harness) after t83.
