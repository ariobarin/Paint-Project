"""Write input scripts for the frame comparison (paint_frames.py and examples/frames.rs): a scripted
tour of every tool and button, and random sessions. Save and Load are left out (they open file
dialogs in the Python version). Usage: gen_scripts.py [sessions], writing to wasm/parity/scripts/."""
import random
import sys
from pathlib import Path

OUT = Path(__file__).parent / "scripts"
OUT.mkdir(exist_ok=True)

TOOLS = {
    "pencil": (30, 125), "eraser": (80, 125), "spray": (30, 175), "paint_brush": (80, 175), "line": (30, 225),
    "rect": (80, 225), "ellipse": (30, 275), "eye_dropper": (80, 275), "fireworks": (30, 325), "air_brush": (80, 325),
    "paint_bucket": (30, 375), "fill_screen": (80, 375), "stamp": (687, 622), "select": (55, 425),
}
FILL, TRASH, UNDO, REDO = (55, 500), (130, 30), (30, 30), (80, 30)
NEXT_STAMP, PREVIOUS_STAMP = (740, 740), (635, 740)
FORWARD, BACKWARD = (972, 615), (972, 735)


class Script:
    def __init__(self, seed, shot_every=0):
        self.lines = [f"seed {seed}"]
        self.n = 0
        self.shot_every = shot_every

    def f(self, x, y, b="000", m="-", e="-", shot=None):
        self.n += 1
        if shot is None and self.shot_every and self.n % self.shot_every == 0:
            shot = f"frame{self.n:05d}"
        self.lines.append(f"f {x} {y} {b} {m} {e}" + (f" {shot}" if shot else ""))

    def click(self, x, y, m="-", hold=1):
        self.f(x, y, m=m)
        self.f(x, y, "100", m, "d")
        for _ in range(hold):
            self.f(x, y, "100", m)
        self.f(x, y, "000", m, "u")

    def drag(self, points, m="-"):
        x, y = points[0]
        self.f(x, y, m=m)
        self.f(x, y, "100", m, "d")
        for x, y in points[1:]:
            self.f(x, y, "100", m)
        self.f(x, y, "000", m, "u")

    def key(self, k, x=600, y=300, m="-"):
        self.f(x, y, "000", m, "k" + k)

    def write(self, name):
        (OUT / name).write_text("\n".join(self.lines) + "\n")


def path(r, a, b, steps):
    (x0, y0), (x1, y1) = a, b
    return [(round(x0 + (x1 - x0) * t / steps + r.randint(-3, 3)), round(y0 + (y1 - y0) * t / steps + r.randint(-3, 3))) for t in range(steps + 1)]


def tour():
    s = Script(11)
    r = random.Random(11)
    s.f(0, 0, shot="start")
    for name, at in TOOLS.items():  # hover descriptions
        s.f(*at, shot=f"hover_{name}")
    s.drag([(200, 690), (250, 690), (300, 690)])  # colour
    s.drag([(100, 652), (200, 652), (300, 652)])  # size
    s.f(600, 300, shot="colour_and_size")
    for name in ["pencil", "eraser", "spray", "paint_brush", "line", "rect", "ellipse", "fireworks", "air_brush"]:
        s.click(*TOOLS[name])
        s.drag(path(r, (200 + r.randint(0, 300), 150 + r.randint(0, 100)), (700 + r.randint(0, 400), 300 + r.randint(0, 200)), 12))
        s.f(600, 300, shot=f"used_{name}")
    s.click(*FILL)  # filled shapes, and shift
    for name in ["rect", "ellipse"]:
        s.click(*TOOLS[name])
        s.drag(path(r, (300, 200), (520, 330), 6), m="s")
        s.drag(path(r, (700, 400), (900, 250), 6))
        s.f(600, 300, shot=f"filled_{name}")
    s.click(*FILL)
    s.click(*TOOLS["line"])
    s.drag(path(r, (300, 450), (800, 480), 8), m="s")
    s.drag([(100, 652), (380, 652)])  # big size
    s.click(*TOOLS["ellipse"])
    s.drag(path(r, (150, 120), (1150, 530), 8))
    s.f(600, 300, shot="thick_ellipse")
    s.drag([(100, 652), (60, 652)])
    s.click(*TOOLS["eye_dropper"])
    s.click(500, 300)
    s.f(600, 300, shot="eye_dropper")
    s.click(*TOOLS["paint_bucket"])
    s.click(1100, 520)
    s.f(600, 300, shot="bucket")
    s.click(*NEXT_STAMP)
    s.click(*NEXT_STAMP)
    s.click(*PREVIOUS_STAMP)
    s.click(*TOOLS["stamp"])
    s.drag([(830, 700), (830, 650), (830, 600)])  # stamp sizer with the stamp tool
    s.f(600, 300, shot="stamp_sizer")
    s.drag([(400, 300), (420, 310)])
    s.f(600, 300, shot="stamped")
    s.click(*FORWARD)
    s.f(600, 300, shot="background_forward")
    s.click(*BACKWARD)
    s.click(*BACKWARD)
    s.f(600, 300, shot="background_backward")
    s.click(*TOOLS["select"])
    s.drag(path(r, (300, 200), (600, 400), 6))
    s.f(600, 300, shot="selected")
    s.drag(path(r, (450, 300), (800, 350), 6))  # move
    s.f(600, 300, shot="moved")
    s.drag(path(r, (950, 450), (1100, 520), 4))  # resize from the bottom right handle (after the move)
    s.f(600, 300, shot="resized")
    s.click(150, 520)  # outside: paste
    s.f(600, 300, shot="pasted")
    for _ in range(3):
        s.click(*UNDO)
    s.f(600, 300, shot="undone")
    s.click(*REDO)
    s.key("z", m="c")
    s.key("y", m="c")
    s.key("z", m="sc")
    s.f(600, 300, shot="keys")
    s.click(*TOOLS["fill_screen"])
    s.click(600, 300)
    s.f(600, 300, shot="fill_screen")
    s.key("x")
    s.click(*TRASH)
    s.f(600, 300, shot="trashed")
    s.write("tour.txt")


def fuzz(seed, actions):
    r = random.Random(seed)
    s = Script(seed, shot_every=0)
    canvas = lambda: (r.randint(90, 1210), r.randint(80, 570))
    ui = list(TOOLS.values()) + [FILL, TRASH, UNDO, REDO, NEXT_STAMP, PREVIOUS_STAMP, FORWARD, BACKWARD]
    for _ in range(actions):
        kind = r.random()
        if kind < 0.3:
            s.click(*r.choice(ui))
        elif kind < 0.7:
            s.drag(path(r, canvas(), canvas(), r.randint(1, 14)), m=r.choice(["-", "-", "s"]))
        elif kind < 0.8:  # sliders
            y = r.choice([652, 692, 732, 772])
            s.drag(path(r, (r.randint(10, 409), y), (r.randint(10, 409), y), r.randint(1, 5)))
        elif kind < 0.85:
            s.drag(path(r, (830, r.randint(560, 789)), (830, r.randint(560, 789)), 3))
        elif kind < 0.92:
            s.key(r.choice("xyz"), m=r.choice(["-", "c", "sc"]))
        else:
            for _ in range(r.randint(1, 4)):
                s.f(r.randint(0, 1199), r.randint(0, 799))
    s.write(f"fuzz{seed}.txt")


tour()
for seed in range(1, 1 + int(sys.argv[1] if len(sys.argv) > 1 else 4)):
    fuzz(seed, 120)
print("\n".join(f"{p.name}: {len(p.read_text().splitlines())} lines" for p in sorted(OUT.iterdir())))
