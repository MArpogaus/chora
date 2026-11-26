# xp-blink

## r1

F7 (rv9kb-1 MINOR): the paperclip blink ran on `Instant::now()`, not on the scene clock the tour steps, so stills could catch shut eyes.

- Base d3d28cfa (xp-minor r1 head); head c2625c0f7 on claude/xp-blink-r1.
- Red c4af42b1c: `the_blink_follows_the_stepped_clock_xp_blink` (held clock; the figure's due time is real time, not the held clock). Fix c2625c0f7.
- Fix: chora-assistant exports `clock.step` (draws one frame of the figure) and every production `Instant::now()` of the plugin (main loop, voice, context, chat host) is `clock::now()`, so blink, move and fade follow the stepped clock; no new option.
- Diffstat: 4 files, 67 insertions, 36 deletions (main.rs, voice.rs, context.rs, chat/host.rs).
- Tour: `run.py xp` twice (own TOUR_OUT each), 0 red, 0 broken; x7 (01-08, 01-10), x10 (01-10d, 01-10e), x11 (01-10f): eyes open in both takes. Crops: media/2026-11-25/xp-blink-r1/eyes-take1.png, eyes-take2.png; full still x10-help-take1/2.jpg.
- Gate: `all steps passed; no --push, nothing is pushed` (full, no step filter, no foreign flake).
