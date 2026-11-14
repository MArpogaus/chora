# Tour recorder

Tools that record a slow, scripted tour of the four looks (xp, tiling, mc,
construct) with `choractl record`: 30 fps, 1280x720, `--pointer arrow`, one
MP4 per chapter and one joined MP4, a still per act, a timeline table.

| File | What |
|------|------|
| `tour.py` | `Tour`: acts with pace, pointer moves, typing, drags, stills, one segment per `choractl record`, chunks joined to an MP4 |
| `launch.py` | headless sessions with a look, companions, a stand-in `wpctl`, the fake voice agent; `SceneScript` runs `demo/scene-*.sh` |
| `dual.py` | a recorder with the requests of `choractl record` for two outputs (chapter 7) |
| `multi.py`, `cons.py` | the four-screen session; the construct session with the demo panes |
| `slides.py` | the chapter slide, a PNG of 1280x720 |
| `ch1.py` to `ch8.py`, `ch7b.py` | one script per chapter (v3 versions; chapters 5 to 8 are the v2 scripts) |
| `assemble.py` | the timeline table and the joined MP4 from the rows and the notes |
| `notes/` | what happened per still, v1 and v2 tours, read by `assemble.py` |
| `sheet.sh`, `ip.py` | contact sheet of stills; poke a live session |

Run: `TOUR_REPO=/path/to/checkout` with built binaries, `TOUR_MOVE`,
`TOUR_CPS` and `TOUR_LEAD` set the pace (see `env.sh`: 0.55, 9, 0.5), then
`python3 chN.py`. The scripts write to a `OUT` directory that each file
names at its top; the paths are those of the recording container
(`/home/user/log/media/...`) and need changing elsewhere. Needs foot,
ffmpeg, Pillow, `/usr/share/chora/flavors` linked to the checkout, and
`CHORA_FRAME_BUDGET_US=500000` (set by `launch.py`).
