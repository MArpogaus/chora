# Tour recorder

Tools that record a slow, scripted tour of the four looks (xp, tiling, mc,
construct) with `choractl record`: 30 fps, 1280x720, `--pointer arrow`, one
MP4 per chapter and one joined MP4, a still per act, a timeline table.

| File | What |
|------|------|
| `human.py` | `Human(Tour)`: pointer paths with overshoot and a stop before the click, typing with pauses and one corrected typo, seeded so a run repeats |
| `tour.py` | `Tour`: acts with pace, pointer moves, typing, drags, stills, one segment per `choractl record`, chunks joined to an MP4 |
| `launch.py` | headless sessions with a look, companions, a stand-in `wpctl`, the fake voice agent; `SceneScript` runs `demo/scene-*.sh` |
| `dual.py` | a recorder with the requests of `choractl record` for two outputs (chapter 7) |
| `multi.py`, `cons.py` | the four-screen session; the construct session with the demo panes |
| `slides.py` | the chapter slide, a PNG of 1280x720 |
| `ch1.py` to `ch7.py` | one script per chapter, v3: real programs (foot, nano), the visible doors (Start, bar, column, key list, chat), one slip and its recovery per look; `ch3.py` takes `MC_THEME=dark` |
| `ch7b.py`, `ch8.py` | v2 scripts, not part of v3 (ch8 is the fake voice agent) |
| `assemble.py` | the timeline table and the joined MP4 from the rows and the notes |
| `notes/` | what happened per still, v1 and v2 tours, read by `assemble.py` |
| `sheet.sh`, `ip.py` | contact sheet of stills; poke a live session |

Run: `TOUR_REPO=/path/to/checkout` with built binaries, `TOUR_MOVE`,
`TOUR_CPS` and `TOUR_LEAD` set the pace (see `env.sh`: 0.55, 9, 0.5), then
`python3 chN.py`. The scripts write to `TOUR_OUT` (default
`/home/user/log/media/2026-11-16/tour-v3`); `assemble.py` reads it too and
takes the build name from `TOUR_BUILD`. The notes `/home/user/notes` are
rewritten at each start. Needs foot,
ffmpeg, Pillow, `/usr/share/chora/flavors` linked to the checkout, and
`CHORA_FRAME_BUDGET_US=500000` (set by `launch.py`).
