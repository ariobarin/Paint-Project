// Renders frames from a script without a browser, to compare with the Python version frame by frame,
// for the frame check of ariobarin.com's Pygame pipeline (scripts/games/pygame/frames.py, which documents
// the script format). Needs a lossless bake (uv run wasm/bake.py --lossless). Usage:
//     cargo run --release --example frames -- <script> <out dir>

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use paint::testing::{Art, Event, Input, Paint, Surface};

fn decode(bytes: &[u8]) -> Surface {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    let channels = info.color_type.samples();
    let px = buffer[..info.buffer_size()]
        .chunks(channels)
        .map(|p| match channels {
            4 => u32::from_le_bytes([p[0], p[1], p[2], p[3]]),
            _ => u32::from_le_bytes([p[0], p[1], p[2], 255]),
        })
        .collect();
    Surface::from_pixels(info.width as i32, info.height as i32, px, channels == 4)
}

fn save(s: &Surface, path: &Path) {
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(path).unwrap()), s.w as u32, s.h as u32);
    encoder.set_color(png::ColorType::Rgb);
    let bytes: Vec<u8> = s.px.iter().flat_map(|p| p.to_le_bytes()[..3].to_vec()).collect();
    encoder.write_header().unwrap().write_image_data(&bytes).unwrap();
}

// zlib's CRC-32, to log every frame cheaply (the Python side uses zlib.crc32)
fn crc32(s: &Surface) -> u32 {
    let mut table = [0u32; 256];
    for (i, t) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *t = c;
    }
    let mut crc = !0u32;
    for p in &s.px {
        for &byte in &p.to_le_bytes()[..3] {
            crc = table[((crc ^ byte as u32) & 0xff) as usize] ^ (crc >> 8);
        }
    }
    !crc
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let script = fs::read_to_string(&args[1]).unwrap();
    let out = Path::new(&args[2]);
    fs::create_dir_all(out).unwrap();
    let baked = Path::new(env!("CARGO_MANIFEST_DIR")).join("baked");
    let seed = script.lines().find_map(|l| l.strip_prefix("seed ")).map_or(0, |s| s.trim().parse().unwrap());

    let mut art = Art::new();
    paint::testing::more(&mut art, &fs::read(baked.join("site/more.bin")).unwrap(), decode);
    let mut first = decode(&fs::read(baked.join("site/first_background.png")).expect("a lossless bake"));
    first.alpha = false;
    let mut paint = Paint::new(&first, &art, seed);
    let mut log = String::new();

    for line in script.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.first() != Some(&"f") {
            continue;
        }
        let b = w[3].as_bytes();
        let mut input = Input {
            mx: w[1].parse().unwrap(),
            my: w[2].parse().unwrap(),
            mb: [b[0] == b'1', b[1] == b'1', b[2] == b'1'],
            shift: w[4].split(',').any(|k| k == "LSHIFT" || k == "RSHIFT"),
            ctrl: w[4].split(',').any(|k| k == "LCTRL" || k == "RCTRL"),
            events: Vec::new(),
        };
        if w[5] != "-" {
            for e in w[5].split(',') {
                // Paint reads the left button and the X, Y and Z keys going down
                match e {
                    "d" => input.events.push(Event::MouseDown),
                    "u" => input.events.push(Event::MouseUp),
                    _ if e.starts_with('k') && e.len() == 2 => input.events.push(Event::Key(e.chars().nth(1).unwrap())),
                    _ => {}
                }
            }
        }
        paint.frame(&input, &art);
        log.push_str(&format!("{:08x}
", crc32(&paint.screen)));
        if let Some(name) = w.get(6) {
            save(&paint.screen, &out.join(format!("{name}.png")));
        }
    }
    fs::write(out.join("crc.txt"), log).unwrap();
}
