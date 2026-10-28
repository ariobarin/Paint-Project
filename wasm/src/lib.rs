// The paint project, compiled from Rust to WebAssembly. paint.rs is main.py and src/ ported line by
// line; web/index.html is the page that runs it.
//
// The page talks to the program through the functions below:
//  1. it decodes the first background (first_background.webp), copies the pixels into
//     image_buffer(width, height) and calls start(seed)
//  2. it fetches more.bin (the stamps and the other backgrounds), copies it into more_buffer(length),
//     decodes each background (background_offset(i), background_length(i)) into image_buffer and
//     calls background_ready(i)
//  3. on every animation frame it calls frame(now) and, when that returns 1, shows screen_pixels()
//  4. mouse and keyboard go to pointer(), mouse_button() and key(); save and load come back through
//     the imports below and picked_image()

mod art;
mod baked;
mod draw;
mod paint;
mod png;
mod random;
mod surface;
mod text;

use std::cell::RefCell;

use art::Art;
use paint::{Event, Input, Paint};
use surface::Surface;

// main.py's clock: 60 frames a second
const FRAME_MS: f64 = 1000.0 / 60.0;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    // hands the page a picture (RGBA) to download as painting.png
    fn save_image(pixels: *const u8, width: i32, height: i32);
    // asks the page for a picture; it arrives through image_buffer and picked_image
    fn pick_image();
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn save_image(_pixels: *const u8, _width: i32, _height: i32) {}
#[cfg(not(target_arch = "wasm32"))]
unsafe fn pick_image() {}

struct Program {
    art: Art,
    paint: Paint,
    input: Input,
    clock: f64,
    last: Option<f64>,
    shown: u64, // a hash of the last picture shown
    first: bool,
}

#[derive(Default)]
struct State {
    image: Option<Surface>,
    more: Vec<u8>,
    stamps_decoded: usize,
    program: Option<Program>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|state| f(&mut state.borrow_mut()))
}

// Room for a decoded picture (RGBA), for start, background_ready or picked_image
#[unsafe(no_mangle)]
pub extern "C" fn image_buffer(width: i32, height: i32) -> *mut u8 {
    with(|state| {
        let image = state.image.insert(Surface::new(width, height, true));
        image.px.as_mut_ptr() as *mut u8
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn start(seed_high: u32, seed_low: u32) {
    with(|state| {
        let mut first = state.image.take().expect("the first background");
        first.alpha = false;
        let art = Art::new();
        let paint = Paint::new(&first, &art, (seed_high as u64) << 32 | seed_low as u64);
        state.program = Some(Program { art, paint, input: Input::default(), clock: 0.0, last: None, shown: 0, first: true });
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn more_buffer(length: usize) -> *mut u8 {
    with(|state| {
        state.more = vec![0; length];
        state.more.as_mut_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn background_count() -> usize {
    baked::BACKGROUNDS.len()
}

#[unsafe(no_mangle)]
pub extern "C" fn background_offset(i: usize) -> *const u8 {
    with(|state| state.more[baked::BACKGROUNDS[i].0..].as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn background_length(i: usize) -> usize {
    baked::BACKGROUNDS[i].1
}

#[unsafe(no_mangle)]
pub extern "C" fn background_ready(i: usize) {
    with(|state| {
        let mut image = state.image.take().expect("a background");
        image.alpha = false;
        if let Some(program) = state.program.as_mut() {
            program.art.backgrounds[i] = Some(image);
        }
    })
}

// A picture the page picked for Load (src/webcompat.py's picked_image)
#[unsafe(no_mangle)]
pub extern "C" fn picked_image() {
    with(|state| {
        let image = state.image.take();
        if let Some(program) = state.program.as_mut() {
            program.paint.picked_image = image;
        }
    })
}

// The mouse position in the program's pixels, and which buttons are held (1 left, 2 middle, 4 right)
#[unsafe(no_mangle)]
pub extern "C" fn pointer(x: i32, y: i32, buttons: u32, shift: u32, ctrl: u32) {
    with(|state| {
        if let Some(program) = state.program.as_mut() {
            let input = &mut program.input;
            input.mx = x.clamp(0, 1199);
            input.my = y.clamp(0, 799);
            input.mb = [buttons & 1 != 0, buttons & 2 != 0, buttons & 4 != 0];
            input.shift = shift != 0;
            input.ctrl = ctrl != 0;
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mouse_button(down: u32) {
    with(|state| {
        if let Some(program) = state.program.as_mut() {
            program.input.events.push(if down != 0 { Event::MouseDown } else { Event::MouseUp });
        }
    })
}

// A key press: the character of the key, lower case (the program reacts to x, y and z)
#[unsafe(no_mangle)]
pub extern "C" fn key(code: u32) {
    with(|state| {
        if let Some(program) = state.program.as_mut() {
            if let Some(c) = char::from_u32(code) {
                program.input.events.push(Event::Key(c));
            }
        }
    })
}

// Runs the frames owed since the last call. Returns 1 when the picture changed. Until the stamps and
// backgrounds are in, the program shows its first frame and waits.
#[unsafe(no_mangle)]
pub extern "C" fn frame(now: f64) -> u32 {
    with(|state| {
        decode_a_stamp(state);
        let Some(program) = state.program.as_mut() else { return 0 };
        let elapsed = now - program.last.unwrap_or(now);
        program.last = Some(now);
        let mut frames = 0;
        if program.first {
            program.first = false;
            frames = 1;
        } else if program.art.complete() {
            program.clock = if elapsed > 250.0 { 0.0 } else { program.clock + elapsed };
            // round to the nearest frame so a 60 Hz screen gets exactly one per refresh despite jitter
            frames = ((program.clock + FRAME_MS / 4.0) / FRAME_MS).floor().max(0.0) as u32;
            program.clock -= frames as f64 * FRAME_MS;
            if frames > 4 {
                frames = 4;
                program.clock = 0.0;
            }
        }
        for _ in 0..frames {
            let input = program.input.clone();
            program.input.events.clear();
            program.paint.frame(&input, &program.art);
            if let Some(canvas) = program.paint.saved.take() {
                unsafe { save_image(canvas.px.as_ptr() as *const u8, canvas.w, canvas.h) };
            }
            if std::mem::take(&mut program.paint.load_requested) {
                unsafe { pick_image() };
            }
        }
        if frames == 0 {
            return 0;
        }
        // only show the picture when it changed: an idle paint program redraws the same frame
        let hash = hash(&program.paint.screen.px);
        if hash == program.shown {
            return 0;
        }
        program.shown = hash;
        1
    })
}

// Decodes one more stamp, if any are waiting; the page calls this while it holds the first frame
#[unsafe(no_mangle)]
pub extern "C" fn work() {
    with(decode_a_stamp);
}

// one stamp per call, so the page never stalls on all of them at once
fn decode_a_stamp(state: &mut State) {
    if state.more.is_empty() || state.stamps_decoded >= baked::STAMPS.len() {
        return;
    }
    let (offset, length) = baked::STAMPS[state.stamps_decoded];
    if let Some(program) = state.program.as_mut() {
        let (w, h, px, alpha) = png::decode_pixels(&state.more[offset..offset + length]);
        program.art.stamps[state.stamps_decoded] = Some(Surface::from_pixels(w, h, px, alpha));
        state.stamps_decoded += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn screen_pixels() -> *const u8 {
    with(|state| state.program.as_ref().map_or(std::ptr::null(), |p| p.paint.screen.px.as_ptr() as *const u8))
}

fn hash(px: &[u32]) -> u64 {
    // FNV-1a over pairs of pixels
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for pair in px.chunks(2) {
        let v = pair[0] as u64 | (pair.get(1).copied().unwrap_or(0) as u64) << 32;
        h = (h ^ v).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

// For the frame by frame comparison with the Python version (examples/frames.rs)
#[doc(hidden)]
pub mod testing {
    pub use crate::art::Art;
    pub use crate::paint::{Event, Input, Paint};
    pub use crate::surface::Surface;

    pub fn more(art: &mut Art, pack: &[u8], background: impl Fn(&[u8]) -> Surface) {
        for (i, &(offset, length)) in crate::baked::STAMPS.iter().enumerate() {
            let (w, h, px, alpha) = crate::png::decode_pixels(&pack[offset..offset + length]);
            art.stamps[i] = Some(Surface::from_pixels(w, h, px, alpha));
        }
        for (i, &(offset, length, _)) in crate::baked::BACKGROUNDS.iter().enumerate() {
            art.backgrounds[i] = Some(background(&pack[offset..offset + length]));
        }
    }
}
