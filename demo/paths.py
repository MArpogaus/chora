#!/usr/bin/env python3
"""The camera paths and the drags of the demo clips, as a script for
`choractl record --script`.

Usage: demo/paths.py CLIP          prints the cues, one JSON object per line
       demo/paths.py CLIP frames   prints the count of frames of the clip
  CLIP  walk: the camera walks through the construct of
              demo/scene-construct.sh and comes back
        swing: the pane in the middle is grabbed, swung and let go, twice

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


CLIPS = {"walk": walk, "swing": swing}


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
