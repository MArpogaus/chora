#!/usr/bin/env python3
"""The camera paths and the drags of the demo clips, as a script for
`choractl record --script`.

Usage: demo/paths.py CLIP          prints the cues, one JSON object per line
       demo/paths.py CLIP frames   prints the count of frames of the clip
  CLIP  walk: the camera walks through the construct of
              demo/scene-construct.sh and comes back
        swing: the pane in the middle is grabbed, swung and let go, twice
        cube: in the cube of demo/scene-cube.sh, a click on STEP BACK,
              four slow quarter turns and a click on STEP IN
        float: the cube stood back and stopped in a turn; the last frame
               is the still
        pull: the camera stands off, a window is dragged by its title bar
              off the front face, swings as a pane and is dropped

The clips run at 30 frames a second on an output of 1280 by 720.
"""

import json
import sys

HOME = {"x": 0, "z": -100, "yaw": 0, "pitch": -0.07}


def cue(frame, method, params):
    print(json.dumps({"frame": frame, "method": method, "params": params}))


def glide(frame, pose, ms):
    call = {"client": "construct", "method": "camera.glide", "params": {**pose, "ms": ms}}
    cue(frame, "methods.call", call)


def inject(frame, events):
    cue(frame, "input.inject", {"events": events})


def move(frame, x, y):
    inject(frame, [{"pointer_move": {"position": [round(x, 1), round(y, 1)]}}])


def smooth(k):
    return k * k * (3 - 2 * k)


class Drag:
    """A drag of a pane with the modifier, from a point of the output."""

    def __init__(self, frame, x, y):
        self.frame, self.x, self.y = frame, x, y
        inject(
            frame,
            [
                {"pointer_move": {"position": [x, y]}},
                {"key": {"key": "Super_L", "state": "pressed"}},
                {"button": {"state": "pressed", "hold": True}},
            ],
        )

    def to(self, frames, dx, dy):
        """Moves the pointer by an offset over a count of frames."""
        for step in range(1, frames + 1):
            k = smooth(step / frames)
            move(self.frame + step, self.x + dx * k, self.y + dy * k)
        self.frame += frames
        self.x += dx
        self.y += dy
        return self

    def hold(self, frames):
        self.frame += frames
        return self

    def release(self):
        self.frame += 1
        released = [{"button": {"state": "released"}}, {"key": {"key": "Super_L", "state": "released"}}]
        inject(self.frame, released)
        return self.frame


def walk():
    glide(1, HOME, 0)
    move(1, 640, 330)
    glide(8, {"x": -150, "z": 350, "yaw": -1.0, "pitch": -0.05}, 2600)
    glide(90, {"x": 0, "z": 560, "yaw": 0, "pitch": -0.04}, 1800)
    glide(146, {"x": 150, "z": 350, "yaw": 1.0, "pitch": -0.05}, 1800)
    glide(202, HOME, 2000)
    return 268


def swing():
    glide(1, {"x": 0, "z": 200, "yaw": 0, "pitch": -0.05}, 0)
    move(1, 640, 325)
    first = Drag(12, 640, 325)
    first.to(12, 0, 230).hold(3)
    first.to(7, 250, 0).to(11, -500, -10).to(9, 400, -10).to(8, -150, 20).hold(4)
    end = first.release()
    second = Drag(end + 34, 640, 330)
    second.to(8, 190, -70).to(9, -190, -60).hold(2)
    end = second.release()
    return end + 36


def construct(frame, method, params):
    cue(frame, "methods.call", {"client": "construct", "method": method, "params": params})


def glide_pointer(frame, start, end, frames):
    """Moves the pointer from a point to a point over a count of frames."""
    for step in range(1, frames + 1):
        k = smooth(step / frames)
        move(frame + step, start[0] + (end[0] - start[0]) * k, start[1] + (end[1] - start[1]) * k)
    return frame + frames


REST = (930, 470)
CHIP = (1200, 656)
QUARTER_MS = 1500
QUARTER = QUARTER_MS * 30 // 1000


def cube():
    move(1, *REST)
    frame = glide_pointer(14, REST, CHIP, 22)
    frame += 6
    inject(frame, [{"click": {}}])
    frame += 20
    for _ in range(4):
        construct(frame, "cube.turn", {"by": 1, "ms": QUARTER_MS})
        frame += QUARTER + 10
    frame += 2
    inject(frame, [{"click": {}}])
    frame = glide_pointer(frame + 16, CHIP, REST, 22)
    return frame + 10


def float_still():
    move(1, 1100, 560)
    construct(2, "view.step_back", {})
    construct(20, "cube.turn", {"by": 1, "ms": 4000})
    return 46


PULL_VIEW = {"x": -360, "z": -600, "yaw": 0.38, "pitch": -0.02}
TITLE = (330, 147)


def pull():
    move(1, 640, 420)
    glide(8, {**PULL_VIEW}, 1400)
    frame = glide_pointer(30, (640, 420), TITLE, 26)
    frame += 6
    inject(frame, [{"button": {"state": "pressed", "hold": True}}])
    at = list(TITLE)

    def to(frames, dx, dy):
        nonlocal frame, at
        end = (at[0] + dx, at[1] + dy)
        frame = glide_pointer(frame, at, end, frames)
        at = list(end)

    to(24, 300, 45)
    to(22, 370, 75)
    frame += 4
    to(9, 80, 80)
    to(10, -130, -40)
    to(9, 70, 25)
    frame += 6
    inject(frame, [{"button": {"state": "released"}}])
    return frame + 50


CLIPS = {"walk": walk, "swing": swing, "cube": cube, "float": float_still, "pull": pull}


def main():
    if len(sys.argv) < 2 or sys.argv[1] not in CLIPS:
        sys.exit(__doc__)
    if sys.argv[2:] == ["frames"]:
        stdout, sys.stdout = sys.stdout, open("/dev/null", "w", encoding="utf-8")
        frames = CLIPS[sys.argv[1]]()
        sys.stdout = stdout
        print(frames)
    else:
        CLIPS[sys.argv[1]]()


if __name__ == "__main__":
    main()
