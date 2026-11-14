# Tour handover (recorder session, 2026-11-14)

Tooling: branch `claude/tour` (48fc05b5), `demo/tour/`, no media. Media: `media/2026-11-12/tour`
(v1), `tour-v2` (06044cf), `construct-v2` (captures). tour-v3 was never recorded.


## What the tour covers
Eight chapters at 1280x720, 30 fps, stepped clock: 1 xp, 2 tiling, 3 mc (light, dark),
4 construct, 5 construct with xp and tiling in panes, 6 xp cube, 7 window drags between looks on
two screens (xp, tiling, construct, mc; key and menu gives), 8 voice and chat (fake voice agent).
v2 has chapter slides, moves 0.55 s, 9 characters a second, 151 timeline rows.
## How to run
See `demo/tour/README.md`. `TOUR_REPO=<checkout>` with built binaries, `python3 chN.py`, then
`assemble.py`. `/usr/share/chora/flavors` must point at the checkout's `flavors/`. Paths in the
scripts are those of the recording container.
v3 state: ch2 (tiling v2: Super+Shift+space, Super+/ key list), ch3 (mc v2: Super+Home,
Super+a chat, column) and ch4 (construct start card, Super+t terminals) are written for the v2
looks but were never run. ch5 to ch8 are the v2 scripts. ch6 (cube) needs new coordinates:
the cube is smaller, rests 14 degrees off square and turns with the hand.
## Open UI fixes (MAJOR, reports/review-construct-v2.md, round 1)
- M1 cube stood back is hidden behind an open pane; face label and pane label pile up
- M2 corner chip `Step back Alt+Esc` is back in use mode and covers the face
- M3 pane label floats over the face in use mode
- M4 rest-pose cube is off centre, flips side between captures; label touches the cube
- M5 glass capsule over dark content: grey fill and a stray rim line
- M6 side panes in walk mode are grey slabs (#ACACAC)
- M7 first program's text touches the screen edge (foot has no padding)
Status after that review: not known to me; a fresh worker was to fix them (I was told to wait).
## What makes the current script artificial
- Pacing is uniform: every act has the same lead, tail and 0.55 s move; a person pauses at
  what is new and hurries through what is known.
- Pointer paths are smoothstep lines; no overshoot, no hesitation, never a wrong click.
- Typing is a constant rate; no typos, no pauses to think. Commands are `ls`, `cat friday.md`.
- Sessions start ready: no boot, no first-run confusion, no discovery by trial.
- Scenes no user would do: stepping through every layout in a row; muting and unmuting;
  F1 in xp (does nothing); Ctrl+Alt+Right to "show" a turn; method calls (camera glide,
  `pane.manager`, `chat.show`) in place of walking and clicking; a fake voice agent whose
  events are written to a FIFO at set frames; five window drags in one row.
- Chapter 7 shows two screens at half size each; injected keys cannot be held across steps.
