# xp-blink

## r1

F7 (rv9kb-1 MINOR): the paperclip blink ran on `Instant::now()`, not on the scene clock the tour steps, so stills could catch shut eyes.

- Base d3d28cfa (xp-minor r1 head); head c2625c0f7 on claude/xp-blink-r1.
- Red c4af42b1c: `the_blink_follows_the_stepped_clock_xp_blink` (held clock; the figure's due time is real time, not the held clock). Fix c2625c0f7.
- Fix: chora-assistant exports `clock.step` (draws one frame of the figure) and every production `Instant::now()` of the plugin (main loop, voice, context, chat host) is `clock::now()`, so blink, move and fade follow the stepped clock; no new option.
- Diffstat: 4 files, 67 insertions, 36 deletions (main.rs, voice.rs, context.rs, chat/host.rs).
- Tour: `run.py xp` twice (own TOUR_OUT each), 0 red, 0 broken; x7 (01-08, 01-10), x10 (01-10d, 01-10e), x11 (01-10f): eyes open in both takes. Crops: media/2026-11-25/xp-blink-r1/eyes-take1.png, eyes-take2.png; full still x10-help-take1/2.jpg.
- Gate: `all steps passed; no --push, nothing is pushed` (full, no step filter, no foreign flake).

## Review r1 (PM, diff 103 lines < 200; builder is not the reviewer)

Base d3d28cfa, head c2625c0f7, 4 files +67 -36 (chora-assistant only). Red c4af42b1 before fix c2625c0f: yes. The figure takes the clock of a recording (chora_kit::clock, as the other plugins), exports clock.step and draws one frame per step; tour xp 2 takes 0 red, gate all passed.

Findings
- rxb1-1 MINOR plugins/chora-assistant/src/voice.rs:1031,1045 and main.rs ask/chat retries: every Instant::now() moved to clock::now(), also timers that are not animations (the card's TooSoon guard, the state ask retries, chat_retried); clock.rs says the stepped time is for animations. While a recording holds the clock (at most STALE 2 s per step) a card press reads TooSoon. Fix: keep guards and retries on Instant::now().
- rxb1-2 MINOR main.rs:625-640 serve loop: while the clock is held, a due time at or before the held time gives wait 0, so the loop polls every 1 ms between steps. Fix: while stepped, wait for the next event or step (IDLE), not the due time.

Verdict (computed): 0 BLOCK, 0 MAJOR, MERGE-OK; rxb1-1, rxb1-2 to the backlog (rules.md 6).

## r2

rxb1-1, rxb1-2 (MINOR): only animations follow the scene clock; the serve loop does not spin while the clock is held.

- Base c2625c0f7 (xp-blink r1 head); head 312a5ff10 on claude/xp-blink-r2. Diffstat: 4 files, 99 insertions, 30 deletions (chora-assistant main.rs, voice.rs, chat/host.rs; chora-kit clock.rs +7: `clock::held()`).
- Red 965954350 before fix 0b3b6cb8f: three tests fail on c2625c0f7 (`a_held_clock_leaves_the_card_guard`, `a_held_clock_leaves_the_ask_retries`, `a_held_clock_does_not_spin_the_serve_loop`, all `_xp_blink_r2`).
- Fix: the card guard (new_card, settle, press_card), the state-ask retry and chat_retried are on `Instant::now()`; blink and other animations keep `clock::now()`. `serve_wait` leaves the animation dues out while `clock::held()`.
- Finding on the way, fixed in 312a5ff10: with the held wait at IDLE (60 s) a blink caught shut at the end of a step never reopened (the clock goes stale after 2 s and runs real again, nobody woke the loop); x10 (01-10d help) showed shut eyes in two takes. The held wait is now capped at 500 ms (`HELD_RECHECK`), still no spin. The brief said IDLE; this deviates for that reason.
- Tour: `run.py xp` 0 red 0 broken; x7 (01-08, 01-10), x10 (01-10d, 01-10e), x11 (01-10f) eyes open in the last take (x10 shut in the two takes before 312a5ff10). Crop: media/2026-11-26/xp-blink-r2/eyes-r2-fixed.png (92 KB).
- Gate: first full run (`--jobs 2`, no step filter) was green through last-words, then `cube` FAIL at "a probe window of a program of the face stays after its end: s3 xp VIRTUAL-1 172 84 200 200" (cube.log), and the run stopped there. FLAKE-CUBE-PROBE-END (not on the known list): `--only cube` alone PASS (125 s); the steps after it (acts, pull, placed, construct-hud, construct-extra, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-*, launcher-click, voice-*, agent, tiling-*, handover, mc-scale, watchdog-starved, leak, xp-run, demo, tiling-disc, mc-host, e2e) all PASS in a second run. No `chora --headless` left.

## Review r2 (PM, diff 129 lines < 200; builder is not the reviewer)

Base c2625c0f7, head 312a5ff10, 4 files +99 -30. Red 96595435 before fix 0b3b6cb8 and 312a5ff1: yes. rxb1-1 fixed (card guard, ask retry, chat_retried on Instant::now()); rxb1-2 fixed for the animations (serve_wait drops their dues while held, cap HELD_RECHECK 500 ms; the deviation from IDLE is reasoned: a shut blink reopened). Gate: cube foreign flake named with its line, cube alone PASS, every later step PASS.

Findings
- rxb2-1 MINOR plugins/chora-assistant/src/chat/host.rs:511-519 with main.rs serve_wait: chat_due still adds FIELD_POLL to the held clock time and serve_wait measures it against the real clock, so with the chat open during a held recording the wait falls to 1 ms again (the spin of rxb1-2 on the chat path). Fix: chat_due on the real clock as the other timers, or in the held branch drop dues at or before the held time.

Verdict (computed): 0 BLOCK, 0 MAJOR, MERGE-OK 312a5ff10; rxb2-1 to the backlog (rules.md 6). Train 149.

## r3

- Base 312a5ff10, head 3fd3a3bba (branch claude/xp-blink-r3, merged session head 6daaaab8 first).
- Red commit 1c6764a52 (serve_wait with the clock held and the chat open: 100 ms to HELD_RECHECK, failed before the fix); fix commit 3fd3a3bba (chat_pulled, chat_due and chat_step on Instant::now()).
- Diffstat vs 312a5ff10: plugins/chora-assistant/src/chat/host.rs 4 +-, src/main.rs 30 +++- (2 files, 29 insertions, 5 deletions).
- rxb2-1 fixed. `cargo test -p chora-assistant`: 267 passed.
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`: all steps passed, no FAIL, nothing pushed.
- Tour: `demo/tour/run.py xp` once: 31 acts, 30 green, 1 known, 0 red; eyes open in x7, x10, x11; captures in media/2026-11-26/xp-blink-r3/ (each under 500 KB).
