"""The walk: the construct in 3D. A Python camera projects each node with a
4x4 matrix; the core divides by w. Ctrl+W/S walk, Ctrl+A/D step aside,
Ctrl+Q/E turn. The real windows show live on the first screen.
"""
import math, os, socket, threading, time
from hud import Chora, HERE, place

STEP, TURN, GLIDE = 0.35, 15.0, 160  # screen widths, degrees, milliseconds
LOOK = 0.004  # radians per logical pixel of a drag


def mul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(4)) for j in range(4)] for i in range(4)]


def half(m):
    """A screen matrix for a child of the half-size world group."""
    return [v / 2 if i % 4 < 2 else v for i, v in enumerate(m)]


def column_major(m):
    return [m[r][c] for c in range(4) for r in range(4)]


class Walk:
    def __init__(self, loader):
        self.l, self.c = loader, loader.c
        W, H = loader.w, loader.h
        self.f, self.eye = W * 0.65, H * 0.55  # about 75 degrees of view
        self.press = (0, 0)
        from donut import Donut
        self.donut = Donut(self)
        self.pose = [loader.x(1) + W / 2, -W * 2.2, 0.0, 0.0]  # x, z, yaw, pitch in radians
        self.drag = None
        self.on, self.keys = False, None

    def put(self, name):
        return self.c.put(name, open(os.path.join(HERE, name), "rb").read())

    def screens(self):
        """Each quad: id, kind, props, and its place: world x of the left
        edge, top y, z and size; flat quads lie on the floor."""
        W, H, l = self.l.w, self.l.h, self.l
        top, quads = H * 1.15, []
        rain, win = self.put("rain.frag"), self.put("window.frag")
        for i, (name, kind) in enumerate(__import__("__main__").VIEWS):
            x = l.x(i)
            if kind == "void":
                props = {"kind": "rect", "props": {"size": [W, H], "color": [1, 1, 1, 1],
                                                   "border": {"width": 2, "color": [0.75, 0.78, 0.82, 1]}}}
            elif kind:
                props = {"kind": "shader", "props": {"size": [W, H], "shader": rain}}
            else:
                props = {"kind": "rect", "props": {"size": [W, H], "color": [0.13, 0.13, 0.13, 1]}}
            quads.append((f"walk-s{i}", props, (x, top, 0.0), False))
            quads.append((f"walk-sh{i}", {"kind": "rect", "props": {
                "size": [W * 0.9, W * 0.25], "radius": W * 0.12, "color": [0, 0, 0, 0.08]}},
                (x + W * 0.05, 0, -W * 0.12), True))
            quads.append((f"walk-l{i}", {"kind": "image", "props": {
                "image": self.c.put(f"label{i}.png", __import__("__main__").label_png(name)[0]),
                "size": [H * 0.09 * l.labels[i], H * 0.09]}}, (x, top + H * 0.15, 0.0), False))
        for n in self.c.call("scene.query", {"selector": "kind:surface"}):
            if n["parent"] != "tiling":
                continue
            m = n["transform"]["matrix"]
            w, h = n["surface"]["size"]
            quads.append((f"walk-w{n['id']}", {"kind": "shader", "props": {
                "size": [w, h], "shader": win, "uniforms": {"win": {"node": n["id"]}}}},
                (l.x(0) + m[12], top - m[13], -1.0), False))
        return quads

    def matrix(self, place, flat, pose):
        x0, y0, z0 = place
        # Node point (u, v) to the world: upright, or lying on the floor.
        model = ([[1, 0, 0, x0], [0, 0, 0, 0], [0, -1, 0, z0], [0, 0, 0, 1]] if flat else
                 [[1, 0, 0, x0], [0, -1, 0, y0], [0, 0, 0, z0], [0, 0, 0, 1]])
        return self.project(model, pose)

    def project(self, model, pose):
        """The matrix of a node whose model maps it into the world."""
        cx, cz, yaw, pitch = pose
        W, H = self.l.w, self.l.h
        c, s = math.cos(yaw), math.sin(yaw)
        view = [[c, 0, -s, -c * cx + s * cz], [0, 1, 0, -self.eye],
                [s, 0, c, -s * cx - c * cz], [0, 0, 0, 1]]
        cp, sp = math.cos(pitch), math.sin(pitch)  # positive looks up
        view = mul([[1, 0, 0, 0], [0, cp, -sp, 0], [0, sp, cp, 0], [0, 0, 0, 1]], view)
        proj = [[self.f, 0, W / 2, 0], [0, -self.f, H / 2, 0], [0, 0, 0, 0], [0, 0, 1, 0]]
        return column_major(mul(proj, mul(view, model)))

    def frame(self, pose, conn=None):
        cmds = [{"update": {"id": q, "props": {"transform": {"matrix": half(self.matrix(p, flat, pose))}}}}
                for q, _, p, flat in self.quads]
        cmds.append({"update": {"id": "walk-floor", "props": {"uniforms": self.floor(pose)}}})
        cmds += self.donut.updates(pose)
        (conn or self.c).call("scene.apply", {"commands": cmds})

    def render(self):
        """The one painter of the walk: 30 frames a second from the current
        pose. Keys and drags only change the pose, so no two writers fight."""
        conn = Chora("loader")
        while self.on:
            try:
                self.frame(list(self.pose), conn)
            except RuntimeError:
                pass  # the walk ended in between
            time.sleep(1 / 30)
        conn.sock.close()

    def floor(self, pose):
        return {"view": [self.l.w, self.l.h], "cam": pose, "eye": self.eye,
                "focal": self.f, "grid": self.l.w * 0.3}

    def enter(self):
        W, H = self.l.w, self.l.h
        self.l.step(1.0, [0, 0])  # the camera at rest; the walk draws in screen space
        self.quads = self.screens()
        cmds = [{"remove": {"id": n["id"]}} for n in self.c.call("scene.query", {"selector": "id:walk"})]
        cmds += [{"create": {"id": "walk", "kind": "group", "parent": "root", "props": {}}},
                 # An opaque backdrop: a rect is no layer, so it never falls out
                 # of the layer budget, and no window shows through.
                 {"create": {"id": "walk-bg", "kind": "rect", "parent": "walk", "props": {
                     "size": [W, H], "color": [0.97, 0.975, 0.98, 1]}}},
                 # The world behind the donut, at half resolution: the glass
                 # reads it, and the budget counts size times output scale.
                 {"create": {"id": "walk-world", "kind": "group", "parent": "walk", "props": {
                     "size": [W / 2, H / 2], "transform": {"scale": [2, 2]}}}},
                 {"create": {"id": "walk-floor", "kind": "shader", "parent": "walk-world", "props": {
                     "size": [W / 2, H / 2], "shader": self.put("walkfloor.frag"), "uniforms": self.floor(self.pose)}}}]
        for q, spec, p, flat in self.quads:
            props = dict(spec["props"], transform={"matrix": half(self.matrix(p, flat, self.pose))})
            cmds.append({"create": {"id": q, "kind": spec["kind"], "parent": "walk-world", "props": props}})
        cmds += self.donut.creates(self.pose)
        # On top of the walk: a pane that turns drags into looks.
        cmds.append({"create": {"id": "walk-look", "kind": "rect", "parent": "walk", "props": {
            "size": [W, H], "color": [0, 0, 0, 0]}}})
        if self.c.call("scene.query", {"selector": "id:hud"}):
            cmds.append({"move": {"id": "hud", "parent": "root"}})
        self.c.call("scene.apply", {"commands": cmds})
        # The keys belong to a connection of their own, so that they end with the walk.
        self.keys = Chora("loader")
        self.bind = {self.keys.call("input.grab", {"binding": f"Ctrl+{k}"}): k for k in "wasdqe"}
        self.on = True
        self.donut.on = True
        threading.Thread(target=self.render, daemon=True).start()
        threading.Thread(target=self.donut.run, daemon=True).start()
        return self.keys

    def leave(self):
        self.on = False
        self.donut.on = False
        self.eye = self.l.h * 0.55
        if self.c.call("scene.query", {"selector": "id:walk"}):
            self.c.call("scene.apply", {"commands": [{"remove": {"id": "walk"}}]})
        if not self.keys:
            return
        # shutdown ends the connection and its grabs, also while the reader
        # thread still holds the file.
        self.keys.sock.shutdown(socket.SHUT_RDWR)
        self.keys = None

    def key(self, binding):
        k = self.bind.get(binding)
        if not k:
            return
        x, z, yaw, pitch = self.pose
        d = STEP * self.l.w
        fx, fz = math.sin(yaw), math.cos(yaw)  # forward
        move = {"w": (fx * d, fz * d, 0), "s": (-fx * d, -fz * d, 0),
                "a": (-fz * d, fx * d, 0), "d": (fz * d, -fx * d, 0),
                "q": (0, 0, -math.radians(TURN)), "e": (0, 0, math.radians(TURN))}[k]
        self.glide([x + move[0], z + move[1], yaw + move[2], pitch], self.eye, GLIDE)

    def glide(self, target, eye, ms):
        start, a, eye0 = time.monotonic(), list(self.pose), self.eye
        while True:
            k = min(1.0, (time.monotonic() - start) * 1000 / ms)
            e = 1 - (1 - k) ** 3
            self.eye = eye0 + (eye - eye0) * e
            self.pose = [a[i] + (target[i] - a[i]) * e for i in range(4)]
            if k == 1.0:
                break
            time.sleep(1 / 60)
        self.pose = target

    def pointer(self, p):
        """A drag over the walk looks around: sideways turns, up and down
        tilts the head."""
        if p.get("node") != "walk-look":
            return
        x, y = p["position"]
        if p["event"] == "pointer.pressed":
            self.drag = (x, y, self.pose[2], self.pose[3])
            self.press = (x, y)
        elif p["event"] == "pointer.moved" and self.drag:
            x0, y0, yaw, pitch = self.drag
            self.pose[2] = yaw + (x - x0) * LOOK
            self.pose[3] = max(-1.2, min(1.2, pitch - (y - y0) * LOOK))
        elif p["event"] == "pointer.released":
            self.drag = None
            if abs(x - self.press[0]) + abs(y - self.press[1]) < 4:
                self.click(x, y)

    def click(self, x, y):
        """A click without a drag activates the panel under it: a ray from
        the eye meets the plane of the screens."""
        W, H, l = self.l.w, self.l.h, self.l
        cx, cz, yaw, pitch = self.pose
        dx, sy = (x - W / 2) / self.f, -(y - H / 2) / self.f
        cp, sp = math.cos(pitch), math.sin(pitch)
        y1, z1 = cp * sy + sp, -sp * sy + cp
        c, s = math.cos(yaw), math.sin(yaw)
        ddx, ddz = c * dx + s * z1, -s * dx + c * z1
        if ddz <= 1e-6 or cz >= 0:
            return
        t = -cz / ddz
        hx, hy = cx + ddx * t, self.eye + y1 * t
        top = H * 1.15
        for i in range(len(l.labels)):
            u, v = hx - l.x(i), top - hy
            if 0 <= u <= W and 0 <= v <= H:
                surface = None
                for q, spec, (x0, _, _), _ in self.quads:
                    if q.startswith("walk-ws") and i == 0:
                        w, h = spec["props"]["size"]
                        if x0 - l.x(0) <= u <= x0 - l.x(0) + w and self.top_of(q) <= v <= self.top_of(q) + h:
                            surface = q[6:]
                # Glide to where the screen fills the view, then hand over to the 2D view.
                self.glide([l.x(i) + W / 2, -self.f, 0.0, 0.0], top - H / 2, 600)
                self.l.activate(i, surface)
                return

    def top_of(self, quad):
        """The top of a window quad, in pixels below the top of its screen."""
        top = self.l.h * 1.15
        return next(top - y0 for q, _, (_, y0, _), _ in self.quads if q == quad)
