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
