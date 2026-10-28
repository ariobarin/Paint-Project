# The paint project in the browser

The paint project ported line by line to Rust and compiled to WebAssembly, with no Python or pygame
in the browser. `src/paint.rs` is `main.py` and `src/`, and `src/draw.rs` is the part of pygame's
`draw.c` the program uses. Every frame comes out pixel for pixel the same as the Python version with
pygame 2.6.1, which it was made with: the shapes, the blending, the scaling and the text. The one
difference is the music, which is left out (it's commercial songs), so there's no music player or
volume slider. This is the version [ariobarin.com/paint](https://ariobarin.com/paint) embeds.

Build it (needs [uv](https://docs.astral.sh/uv/) and Rust with its `wasm32-unknown-unknown` target):

```bash
uv run wasm/bake.py
cd wasm
cargo build --release --target wasm32-unknown-unknown
```

`bake.py` uses pygame itself to prepare everything that never changes: the icons and previews
smoothscaled into their buttons, the backgrounds scaled to the window, the fixed text, and the font
metrics and glyphs for the colour code. The browser page is `web/index.html`; it loads `paint.wasm`
(about 1 MB, with the buttons and text built in) and `first_background.webp`, then `more.bin` (the
stamps and the other backgrounds) after the first frame.

## Checking it against the Python version

`parity/` runs the Python version headless through a script of inputs and logs every frame, and
`examples/frames.rs` does the same for the port. Bake with `--lossless` first, so the backgrounds
are exact.

```bash
uv run wasm/bake.py --lossless
uv run python wasm/parity/gen_scripts.py 4
uv run --with pygame==2.6.1 --with numpy python wasm/parity/paint_frames.py . wasm/parity/scripts/tour.txt python-frames
cargo run --release --manifest-path wasm/Cargo.toml --example frames -- wasm/parity/scripts/tour.txt rust-frames
diff --strip-trailing-cr python-frames/crc.txt rust-frames/crc.txt
```

The tour and three random sessions (2,785 frames, every tool, button and slider) are identical.

## Licence

`src/draw.rs` and `src/surface.rs` are ported from pygame's C source (`src_c/draw.c`, `src_c/transform.c`
and its blending), which is under the GNU Library General Public License, version 2 or later, so those
two files are under the LGPL 2.1 or later too.
