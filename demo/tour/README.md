# Tour recorder

One set of scripts records a scripted tour of the four looks (xp, tiling, mc, construct) with
`choractl record`: 30 fps, 1280x720 (handover 1920x1080), `--pointer arrow`, one MP4 per chapter and
one joined MP4, a still per act, a timeline table. Design: `design/tour-v9.md` (on `claude/session-log`).

| File | What |
|------|------|
| `run.py` | `run.py [--check] [xp tiling mc construct handover]`: runs the chapters; `--check` runs the same rows without video, stills or pace and exits 1 on a red act |
| `tour.py` | `Tour`: acts with pace, pointer moves, typing, drags, stills, one segment per `choractl record`, chunks joined to an MP4; `Checker` judges each act (`Expect` and the invariants I1 to I14 of `tests/acts`) |
| `human.py` | `Human(Tour)`: pointer paths with overshoot, typing with one corrected typo, seeded so a run repeats; `Quick(Human)`: the pace of v4 to v9 |
| `launch.py` | headless sessions with a look, companions, the notify timing of every chapter (`notify_config`), a stand-in `wpctl`, the fake voice agent; PATH without writers, a kept notification bus; `SceneScript` runs `demo/scene-*.sh` |
| `slides.py` | the chapter slide, a PNG |
| `assemble.py` | the timeline table (with `contract`, `cells`, `capture` and a `check` column: `green` an `expect` held, `inv` no `expect` and only the invariants held, `RED: ...`, `known: ...`, `todo`) and the joined MP4 |
| `chapters/*.py` | `xp`, `tiling`, `mc` (light and dark), `construct`, `handover`: one `run(ctx)` each, the acts as rows |
| `dual.py` | the two-screen recorder of the handover chapter |
| `sheet.sh`, `env.sh` | a contact sheet of stills; the pace variables |

`tools/tour-probe.py` shows what the notification bus saw when Notepad finds no editor (a notice is judged from the bus, not from frames). The row files of a run are named for their chapter (`rows-xp.json`, `rows-mc-light.json`, ...).

## An act is a row

```python
t.act("pointer", "Start, Notepad", "the editor opens in a window", still="01-02-start",
      expect=Expect(new_window={"owner": "xp"}),   # judged with every invariant after the act
      contract="T1", cells=["doors launcher-door"], capture="Q",
      opens=[], closes=[], known=None, todo=False)
```

`expect` is an `Expect` of `tests/acts/harness.py`; an act without one is judged by the invariants
only and shows `inv`, not `green`; the read waits for every `busy` of the looks to be empty. `contract` names a persona contract (T1 to C3), `cells` the gate cells the act repeats on
screen, `capture` the frame (Q, Q+t, H50). `known="I4: why"` names the invariant that is red today (as a row of
`tests/acts/known-red.toml`): a red of that invariant shows as `known` and does not fail the run, any
other red of the act stays `RED`, and `known act N is green: drop known` says when it can go. `todo` is a stub for an act that waits for another slice (xp-give): no cues, no check.
A red act is a red row of `timeline-table.md`.

Each act is judged alone: after `done()` the tour records what is pending and reads the session.

## Run

`TOUR_REPO=/path/to/checkout` with built binaries (`cargo build --workspace`), `TOUR_OUT` the output
folder (default `/tmp/claude-0/tour-v9`), `TOUR_MOVE`, `TOUR_CPS` and `TOUR_LEAD` the pace (`env.sh`).
Needs foot, ffmpeg, Pillow, dbus-daemon, swaybg and waybar (tiling), `CHORA_FRAME_BUDGET_US=500000`
(set by `launch.py`). `tests/test_tour.py` is the selftest without a session (a failing `expect` is a
red row); `tools/copies.toml` holds the row that keeps `v*_`, `tourN`, `assembleN` and `chN` copies out.
