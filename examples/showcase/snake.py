"""snake.py — keyboard snake on the chora scene API.

The snake is rect nodes on a 32px grid; food pulses with a shader. Keys
reach the game as `node.key` events (focus follows pointer, the game
node takes the keyboard), so one tick is: read events, apply moves.
Prints achieved tick rate; with SNAKE_FOOD=<n> it grows to n food items
and long tails — a stress mode against the per-frame element budget.

Run: CHORA_SOCKET=... python snake.py   (grant snake scene.* + screen.*)
"""
import json, os, time
from choraclient import Chora

CELL = 32
COLS, ROWS = 38, 22
W, H = COLS * CELL, ROWS * CELL
TICK = 0.06
FOOD = int(os.environ.get("SNAKE_FOOD", "3"))
RUN = float(os.environ.get("SNAKE_RUN", "20"))  # 0 = run until SIGINT

DIRS = {"Up": (0, -1), "Down": (0, 1), "Left": (-1, 0), "Right": (1, 0)}


def rect(i, x, y, color):
    return {"create": {
        "id": f"sn-{i}",
        "kind": "rect",
        "parent": "sn-root",
        "props": {
            "size": [CELL - 2, CELL - 2],
            "color": color,
            "transform": {"translate": [x * CELL, y * CELL]},
        },
    }}


def move(i, x, y):
    return {"update": {"id": f"sn-{i}", "props": {
        "transform": {"translate": [x * CELL, y * CELL]}}}}


def main():
    c = Chora("snake")
    c.call("events.subscribe", {"events": ["node.key"]})
    for n in [x["id"] for x in c.call("scene.query", {"selector": "owner:snake"})]:
        c.apply([{"remove": {"id": n}}])
    c.apply([
        {"create": {"id": "sn-root", "kind": "group", "parent": "root", "props": {}}},
        {"create": {"id": "sn-board", "kind": "rect", "parent": "sn-root",
                    "props": {"size": [W, H], "color": [0.07, 0.08, 0.1, 1],
                              "input": "accept"}}},
    ])
    import random

    snake, d = [(COLS // 2, ROWS // 2)], (1, 0)
    want = [d]
    food = [(COLS // 2 + 6, ROWS // 2)] + [
        (random.randrange(1, COLS - 1), random.randrange(1, ROWS - 1))
        for _ in range(FOOD - 1)]

    made = 0

    def place():
        nonlocal made
        cmds = []
        while made < len(snake):
            cmds.append(rect(made, *snake[min(made, len(snake) - 1)],
                             (0.2, 0.9, 0.4, 1) if made == 0 else (0.1, 0.6, 0.3, 1)))
            made += 1
        cmds += [move(i, x, y) for i, (x, y) in enumerate(snake)]
        return cmds

    c.apply(place())
    for j, (x, y) in enumerate(food):
        p = rect(f"f{j}", x, y, (0.95, 0.7, 0.2, 1))["create"]["props"]
        p["radius"] = CELL // 2
        c.apply([{"create": {"id": f"f{j}", "kind": "rect",
                             "parent": "sn-root", "props": p}}])
    c.apply([{"focus_node": {"node": "sn-board"}}])
    print(f"snake on {COLS}x{ROWS}, {FOOD} food, arrow keys steer", flush=True)

    start, ticks, worst = time.monotonic(), 0, 0.0
    alive = True
    while alive and (RUN == 0 or time.monotonic() - start < RUN):
        t0 = time.monotonic()
        while not c.events.empty():
            e = c.events.get_nowait()
            if e["event"] == "node.key" and e.get("state") != "released" and e["key"] in DIRS:
                nd = DIRS[e["key"]]
                if (nd[0] + d[0], nd[1] + d[1]) != (0, 0):
                    want.append(nd)
        if want:
            d = want.pop()
        hx, hy = snake[0]
        hx, hy = (hx + d[0]) % COLS, (hy + d[1]) % ROWS
        grew = (hx, hy) in food
        snake.insert(0, (hx, hy))
        if not grew:
            snake.pop()
        cmds = place()
        if grew:
            food.remove((hx, hy))
            fx, fy = (hx + 7) % (COLS - 1) + 1, (hy + 5) % (ROWS - 1) + 1
            food.append((fx, fy))
            cmds.append({"create": {"id": f"f{len(food)}", "kind": "rect",
                                    "parent": "sn-root", "props": {
                                        "size": [CELL - 2, CELL - 2], "radius": CELL // 2,
                                        "color": [0.95, 0.7, 0.2, 1],
                                        "transform": {"translate": [fx * CELL, fy * CELL]}}}})
        done = time.monotonic()
        c.apply(cmds)
        ticks += 1
        worst = max(worst, time.monotonic() - done)
        while time.monotonic() - t0 < TICK:
            time.sleep(0.002)
    span = time.monotonic() - start
    print(f"{ticks} ticks in {span:.1f}s = {ticks/span:.1f} tick/s, "
          f"worst apply {worst*1000:.1f} ms, tail {len(snake)}")
    shots = os.path.join(os.environ.get("XDG_RUNTIME_DIR", os.getcwd()),
                         "chora-showcase-shots")
    os.makedirs(shots, exist_ok=True)
    c.capture(f"{shots}/snake.png", output=os.environ.get("SNAKE_OUTPUT", "HEADLESS-1"))
    c.apply([{"remove": {"id": "sn-root"}}])


if __name__ == "__main__":
    main()
