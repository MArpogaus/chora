"""The donut: a ring of glass panels with live system stats, behind the
start of the walk. Two rows lean in like the inner wall of a torus; the
ring turns slowly. Walk in and look around.
"""
import glob, io, math, os, threading, time
from collections import deque
from PIL import Image, ImageDraw, ImageFont
from hud import Chora, FONT, HERE

PX = (1280, 320)  # pixels of a panel picture: a flat strip of the band
CYAN, WHITE, DIM = (110, 118, 128, 255), (38, 42, 48, 255), (150, 156, 164, 120)  # accent, text, lines
SPIN = 0.04  # radians per second


def read(path, default=""):
    try:
        with open(path) as f:
            return f.read()
    except OSError:
        return default


class Stats:
    """Samples /proc and /sys once a second, with a minute of history."""

    def __init__(self):
        self.hist = {k: deque([0.0] * 60, 60) for k in ("cpu", "mem", "gpu", "rx", "tx", "rd", "wr")}
        self.prev, self.now = None, {}

    def sample(self):
        lines = read("/proc/stat").splitlines()
        cpus = [list(map(int, l.split()[1:9])) for l in lines if l.startswith("cpu")]
        net = [l.split() for l in read("/proc/net/dev").splitlines()[2:] if not l.strip().startswith("lo:")]
        disk = [l.split() for l in read("/proc/diskstats").splitlines() if l.split()[2:3] and l.split()[2].startswith("nvme0n1") and len(l.split()[2]) == 7]
        cur = {"cpus": cpus, "t": time.monotonic(),
               "rx": sum(int(n[1]) for n in net), "tx": sum(int(n[9]) for n in net),
               "rd": sum(int(d[5]) for d in disk) * 512, "wr": sum(int(d[9]) for d in disk) * 512}
        if self.prev:
            busy = []
            for a, b in zip(self.prev["cpus"], cpus):
                idle = (b[3] + b[4]) - (a[3] + a[4])
                total = sum(b) - sum(a)
                busy.append(100 * (1 - idle / total) if total else 0)
            dt = cur["t"] - self.prev["t"]
            n = self.now
            n["cpu"], n["cores"] = busy[0], busy[1:]
            for k in ("rx", "tx", "rd", "wr"):
                n[k] = (cur[k] - self.prev[k]) / dt
            mem = dict(l.split(":") for l in read("/proc/meminfo").splitlines() if ":" in l)
            kb = lambda k: int(mem[k].split()[0])
            n["mem_total"], n["mem_used"] = kb("MemTotal"), kb("MemTotal") - kb("MemAvailable")
            n["swap_total"], n["swap_used"] = kb("SwapTotal"), kb("SwapTotal") - kb("SwapFree")
            n["mem"] = 100 * n["mem_used"] / n["mem_total"]
            n["gpu"] = float(read(glob.glob("/sys/class/drm/card*/device/gpu_busy_percent")[0], "0") or 0)
            n["temps"] = {}
            for h in glob.glob("/sys/class/hwmon/hwmon*"):
                name, t = read(h + "/name").strip(), read(h + "/temp1_input").strip()
                if name and t:
                    n["temps"][name] = int(t) / 1000
            load = read("/proc/loadavg").split()
            n["load"], n["procs"] = list(map(float, load[:3])), load[3]
            n["uptime"] = float(read("/proc/uptime").split()[0])
            n["battery"] = read("/sys/class/power_supply/BAT0/capacity").strip()
            n["charging"] = read("/sys/class/power_supply/BAT0/status").strip()
            for k in self.hist:
                self.hist[k].append(n[k])
        self.prev = cur


def human(b):
    for unit in ("B", "K", "M", "G"):
        if b < 1024:
            return f"{b:5.1f}{unit}"
        b /= 1024
    return f"{b:5.1f}T"


class Painter:
    def __init__(self):
        self.big = ImageFont.truetype(FONT, 84)
        self.mid = ImageFont.truetype(FONT, 26)
        self.small = ImageFont.truetype(FONT, 20)

    def frame(self, title):
        img = Image.new("RGBA", PX)
        d = ImageDraw.Draw(img)
        w, h = PX
        # A strip of the band: no side edges, so the panels merge into one ring.
        for y in range(h):
            shade = 246 - int(10 * abs(y / h - 0.5) * 2)
            d.line((0, y, w, y), fill=(shade, shade + 1, shade + 3, 175))
        d.line((0, 1, w, 1), fill=(200, 204, 210, 255), width=2)
        d.line((0, h - 2, w, h - 2), fill=(200, 204, 210, 255), width=2)
        d.line((0, 24, 0, h - 24), fill=(222, 225, 229, 255), width=1)
        d.text((48, 30), " ".join(title.upper()), font=self.small, fill=CYAN)
        d.line((48, 62, 440, 62), fill=DIM, width=1)
        return img, d

    def spark(self, d, values, box, top=100.0, color=CYAN):
        x0, y0, x1, y1 = box
        top = max(top, max(values) or 1)
        pts = [(x0 + (x1 - x0) * i / (len(values) - 1), y1 - (y1 - y0) * v / top) for i, v in enumerate(values)]
        d.polygon(pts + [(x1, y1), (x0, y1)], fill=color[:3] + (40,))
        d.line(pts, fill=color, width=3)

    def bar(self, d, x, y, w, h, frac, color=CYAN):
        d.rectangle((x, y, x + w, y + h), outline=DIM)
        d.rectangle((x, y, x + w * max(0, min(1, frac)), y + h), fill=color)

    def panel(self, kind, s):
        """Left: the title and the big value; right: a curve or bars."""
        n, H = s.now, s.hist
        img, d = self.frame(kind)
        right = (500, 50, 1230, 280)
        big = lambda text, sub="": (d.text((48, 90), text, font=self.big, fill=WHITE),
                                    d.text((48, 190), sub, font=self.mid, fill=CYAN))
        if kind == "cpu":
            big(f"{n['cpu']:.1f}%", f"{len(n['cores'])} threads")
            self.spark(d, H["cpu"], right)
        elif kind.startswith("cores"):
            lo = 0 if kind == "cores 00-15" else 16
            part = n["cores"][lo:lo + 16]
            big(f"{sum(part) / len(part):.0f}%", f"cores {lo:02d}-{lo + 15:02d}")
            for i, v in enumerate(part):
                x = 500 + i * 46
                d.rectangle((x, 60, x + 30, 270), outline=DIM)
                d.rectangle((x, 270 - 210 * v / 100, x + 30, 270), fill=CYAN)
        elif kind == "memory":
            big(f"{n['mem_used'] / 2**20:.1f}G", f"of {n['mem_total'] / 2**20:.0f}G")
            self.spark(d, H["mem"], right)
        elif kind == "gpu":
            big(f"{n['gpu']:.0f}%", f"{n['temps'].get('amdgpu', 0):.0f}°C")
            self.spark(d, H["gpu"], right)
        elif kind == "thermal":
            t = n["temps"]
            big(f"{max(t.values()):.0f}°", "hottest sensor")
            for i, (name, v) in enumerate(sorted(t.items())[:4]):
                y = 60 + i * 54
                d.text((500, y), f"{name[:8]:8s}", font=self.mid, fill=WHITE)
                self.bar(d, 680, y + 8, 440, 22, v / 100)
                d.text((1140, y), f"{v:3.0f}°", font=self.mid, fill=CYAN)
        elif kind in ("network", "disk"):
            a, b = ("rx", "tx") if kind == "network" else ("rd", "wr")
            big(f"{human(n[a]).strip()}/s", f"{'up' if kind == 'network' else 'write'} {human(n[b]).strip()}/s")
            top = max(max(H[a]), max(H[b]), 1)
            self.spark(d, H[a], right, top)
            self.spark(d, H[b], right, top, (70, 76, 86, 255))
        elif kind == "load":
            big(f"{n['load'][0]:.2f}", "load 1m")
            for i, (label, v) in enumerate(zip(("1m", "5m", "15m"), n["load"])):
                y = 70 + i * 70
                d.text((500, y), label, font=self.mid, fill=CYAN)
                self.bar(d, 580, y + 6, 540, 26, v / len(n["cores"]))
                d.text((1140, y), f"{v:4.2f}", font=self.mid, fill=WHITE)
        elif kind == "processes":
            run, total = n["procs"].split("/")
            big(total, f"{run} running")
            self.spark(d, H["cpu"], right)
        elif kind == "uptime":
            u = int(n["uptime"])
            big(time.strftime("%H:%M:%S"), f"up {u // 86400}d {u % 86400 // 3600:02d}h {u % 3600 // 60:02d}m")
        elif kind == "swap":
            used, total = n["swap_used"] / 2**20, max(n["swap_total"] / 2**20, 0.001)
            big(f"{used:.2f}G", f"of {total:.0f}G")
            self.bar(d, 500, 150, 730, 30, used / total)
        elif kind == "power":
            big(f"{n['battery'] or '--'}%", (n["charging"] or "no battery").lower())
            self.bar(d, 500, 150, 730, 30, int(n["battery"] or 0) / 100)
        buf = io.BytesIO()
        img.save(buf, "PNG")
        return buf.getvalue()


ROWS = [["cpu", "cores 00-15", "memory", "gpu", "thermal", "network"],
        ["load", "cores 16-31", "processes", "uptime", "swap", "power"]]
COLS, MROWS = 64, 8  # mesh of the visible arc: fine, so the affine triangles
# of the renderer do not ripple the picture


class Donut:
    """All panels lie flat in one group, the strip; a mesh bends the
    visible arc of it onto the ring. One layer, no seams."""

    def __init__(self, walk):
        self.walk, self.l = walk, walk.l
        W = self.l.w
        # Behind the start of the walk, in line with the rain screen.
        self.center = (self.l.x(1) + W / 2, -W * 5.2)
        self.radius = W * 2.0
        self.scale = self.l.c.call("output.list", {})[0]["scale"]
        # The strip in logical pixels: one texel per picture pixel.
        self.pw, self.phs = PX[0] / self.scale, PX[1] / self.scale
        self.length = len(ROWS[0]) * self.pw
        # In the world, the strip wraps the ring once, at the same aspect.
        self.ph = 2 * math.pi * self.radius / len(ROWS[0]) * PX[1] / PX[0]
        self.stats, self.painter = Stats(), Painter()
        self.stats.sample()
        time.sleep(0.2)
        self.stats.sample()
        self.start = time.monotonic()
        self.on = False

    def panels(self):
        for row, kinds in enumerate(ROWS):
            for i, kind in enumerate(kinds):
                yield f"walk-p{row}{i}", kind, row, i

    def creates(self, pose):
        W, H = self.l.w, self.l.h
        glass = self.walk.c.put("glass.frag", open(os.path.join(HERE, "glass.frag"), "rb").read())
        # The mask: the band on screen, hidden, at half resolution; its node
        # texture tells the glass where to frost the world.
        cmds = [{"create": {"id": "walk-mask", "kind": "group", "parent": "walk", "props": {
                    "size": [W / 2, H / 2], "transform": {"scale": [2, 2]}, "opacity": 0, "input": "pass"}}},
                {"create": {"id": "walk-mask-band", "kind": "rect", "parent": "walk-mask", "props": {
                    "size": [1, 1], "color": [1, 1, 1, 1], "input": "pass"}}},
                {"create": {"id": "walk-glass", "kind": "shader", "parent": "walk", "props": {
                    "size": [W / 2, H / 2], "transform": {"scale": [2, 2]}, "shader": glass, "input": "pass",
                    "uniforms": {"world": {"node": "walk-world"}, "mask": {"node": "walk-mask"},
                                 "view": [W, H], "radius": 28}}}}]
        cmds += [{"create": {"id": "walk-donut", "kind": "group", "parent": "walk", "props": {
            "size": [self.length / 2, 2 * self.phs], "input": "pass"}}}]
        for p, kind, row, i in self.panels():
            png = self.walk.c.put(f"{p}.png", self.painter.panel(kind, self.stats))
            # Two copies, so a window across the end of the strip wraps.
            for copy in (0, 1):
                cmds.append({"create": {"id": f"{p}-{copy}", "kind": "image", "parent": "walk-donut", "props": {
                    "image": png, "size": [self.pw, self.phs]}}})
        return cmds + self.updates(pose)

    def point(self, theta, y, pose):
        """The screen position of a point of the ring, or None behind the eye."""
        cx, cz = self.center
        wx, wz = cx + self.radius * math.sin(theta), cz + self.radius * math.cos(theta)
        x, z, yaw, pitch = pose
        c, s = math.cos(yaw), math.sin(yaw)
        xc, zc = c * (wx - x) - s * (wz - z), s * (wx - x) + c * (wz - z)
        yc = y - self.walk.eye
        cp, sp = math.cos(pitch), math.sin(pitch)
        yc, zc = cp * yc - sp * zc, sp * yc + cp * zc
        if zc < 1:
            return None
        f, W, H = self.walk.f, self.l.w, self.l.h
        return [W / 2 + f * xc / zc, H / 2 - f * yc / zc]

    def arc(self, pose):
        """The visible arc: the longest run of the ring, in steps of 2
        degrees, that lies in front of the eye and turns its inner side to
        it. None when no part of the ring is in view."""
        cx, cz = self.center
        x, z = pose[0], pose[1]
        inside = math.dist((x, z), self.center) < self.radius
        top, bottom = self.walk.eye + self.ph, self.walk.eye - self.ph
        step = math.radians(2)
        n = round(2 * math.pi / step)

        def fine(theta):
            wx, wz = cx + self.radius * math.sin(theta), cz + self.radius * math.cos(theta)
            facing = inside or (cx - wx) * (x - wx) + (cz - wz) * (z - wz) > 0
            return facing and all(self.point(theta, y, pose) for y in (top, bottom))

        ok = [fine(i * step) for i in range(n)]
        if all(ok):  # the whole ring: show the half around the view
            return pose[2] - math.radians(100), pose[2] + math.radians(100)
        if not any(ok):
            return None
        # Start after a gap, so a run across 0 degrees stays in one piece.
        first = ok.index(False)
        best, run = (0, 0), None
        for k in range(1, n + 1):
            i = (first + k) % n
            if ok[i]:
                run = run or (first + k, first + k)
                run = (run[0], first + k)
                if run[1] - run[0] > best[1] - best[0]:
                    best = run
            else:
                run = None
        a, b = best[0] * step, best[1] * step
        # Never more than 200 degrees: the strip window is at most half of it.
        if b - a > math.radians(200):
            mid = (a + b) / 2
            a, b = mid - math.radians(100), mid + math.radians(100)
        return a, b

    def updates(self, pose):
        arc = self.arc(pose)
        if arc is None or arc[1] - arc[0] < math.radians(3):
            return [{"update": {"id": i, "props": {"visible": False}}} for i in ("walk-donut", "walk-glass")]
        a, b = arc
        spin = SPIN * (time.monotonic() - self.start)
        # Where the arc starts on the strip; the strip turns with the spin.
        x0 = ((a - spin) / (2 * math.pi) * self.length) % self.length
        width = (b - a) / (2 * math.pi) * self.length
        cmds = [{"update": {"id": f"{p}-{copy}", "props": {"transform": {"translate": [
                    i * self.pw + copy * self.length - x0, row * self.phs]}}}}
                for p, kind, row, i in self.panels() for copy in (0, 1)]
        points = []
        for r in range(MROWS + 1):
            y = self.walk.eye + self.ph * (1 - 2 * r / MROWS)
            for j in range(COLS + 1):
                pt = self.point(a + (b - a) * j / COLS, y, pose)
                points.append(pt or [0, 0])
        cmds += [{"update": {"id": i, "props": {"visible": True}}} for i in ("walk-donut", "walk-glass")]
        cmds.append({"update": {"id": "walk-donut", "props": {
            "size": [width, 2 * self.phs], "mesh": {"cols": COLS, "rows": MROWS, "points": points}}}})
        cmds.append({"update": {"id": "walk-mask-band", "props": {
            "size": [width, 2 * self.phs],
            "mesh": {"cols": COLS, "rows": MROWS, "points": [[x / 2, y / 2] for x, y in points]}}}})
        return cmds

    def haze(self, pose):
        """0 far from the donut, 1 inside: how milky the world behind gets."""
        d = math.dist((pose[0], pose[1]), self.center)
        return max(0.0, min(1.0, (self.radius * 1.3 - d) / (self.radius * 0.6)))

    def run(self):
        """While the walk lasts: new stats every second, on a connection of
        its own, so painting never holds up a frame."""
        conn = Chora("loader")
        while self.on:
            self.stats.sample()
            for p, kind, *_ in self.panels():
                conn.put(f"{p}.png", self.painter.panel(kind, self.stats))
            time.sleep(1)
        conn.sock.close()
