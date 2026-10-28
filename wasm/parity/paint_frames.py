"""Run the Python version (pygame 2.6.1, headless) through a script of per-frame inputs; log a CRC-32 of
every frame and save the named ones, to compare with the WebAssembly port (examples/frames.rs reads the
same scripts). Usage, from the repository root:

    uv run --with pygame==2.6.1 --with numpy python wasm/parity/paint_frames.py . <script> <out dir>

Script lines: "seed N", or one frame: "f X Y BUTTONS MODIFIERS EVENTS [SHOT]" (see examples/frames.rs).
The music is switched off, as in the browser version (no music player, no volume slider), and folders
are listed in sorted order, as the browser version lists them.
"""
import asyncio, os, random, sys, zlib
os.environ["SDL_VIDEODRIVER"] = "dummy"
os.environ["SDL_AUDIODRIVER"] = "dummy"
game, script_path, out = (os.path.abspath(a) for a in sys.argv[1:4])
os.makedirs(out, exist_ok=True)
lines = [l.split() for l in open(script_path) if l.strip()]
seed = next((int(l[1]) for l in lines if l[0] == "seed"), 0)
frames = [l for l in lines if l[0] == "f"]
os.chdir(game)
sys.path.insert(0, game)
import pygame
listdir = os.listdir
os.listdir = lambda path=".": sorted(listdir(path))
import src.music_player
import src.sliders
src.music_player.initialize_music = lambda: []
src.music_player.music_player = lambda screen, button_data, tool_settings, music_tracks: music_tracks
src.sliders.draw_volume_slider = lambda screen, button_data, tool_settings: tool_settings["volume"]
random.seed(seed)
state = {"i": -1}
log = []
def current():
    f = frames[min(state["i"], len(frames) - 1)]
    return {"pos": (int(f[1]), int(f[2])), "buttons": tuple(c == "1" for c in f[3]), "mods": f[4], "events": f[5], "shot": f[6] if len(f) > 6 else None}
def get_events():
    state["i"] += 1
    if state["i"] >= len(frames):
        return [pygame.event.Event(pygame.QUIT)]
    f = current()
    evs = []
    if f["events"] != "-":
        for e in f["events"].split(","):
            if e == "d":
                evs.append(pygame.event.Event(pygame.MOUSEBUTTONDOWN, button=1, pos=f["pos"]))
            elif e == "u":
                evs.append(pygame.event.Event(pygame.MOUSEBUTTONUP, button=1, pos=f["pos"]))
            else:
                evs.append(pygame.event.Event(pygame.KEYDOWN, key=getattr(pygame, "K_" + e[1]), mod=0, unicode=e[1]))
    return evs
class Keys:
    def __getitem__(self, k):
        m = current()["mods"]
        return ("s" in m and k in (pygame.K_LSHIFT, pygame.K_RSHIFT)) or ("c" in m and k in (pygame.K_LCTRL, pygame.K_RCTRL))
pygame.event.get = get_events
pygame.mouse.get_pos = lambda: current()["pos"]
pygame.mouse.get_pressed = lambda num_buttons=3: current()["buttons"]
pygame.key.get_pressed = lambda: Keys()
class Clock:
    def tick(self, *a): return 16
pygame.time.Clock = Clock
def flip():
    if 0 <= state["i"] < len(frames):
        surface = pygame.display.get_surface()
        log.append(f"{zlib.crc32(pygame.image.tobytes(surface, 'RGB')):08x}")
        if current()["shot"]:
            pygame.image.save(surface, os.path.join(out, current()["shot"] + ".png"))
pygame.display.flip = flip
import main
if asyncio.iscoroutinefunction(main.main):
    asyncio.run(main.main())
else:
    main.main()
open(os.path.join(out, "crc.txt"), "w").write("\n".join(log) + "\n")
