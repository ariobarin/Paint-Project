// A small PNG decoder for the images bake.py writes (8-bit grey, RGB or RGBA, not interlaced). The
// browser could decode them too, but only premultiplied, which changes the colour of see-through pixels.

pub struct Png {
    pub width: i32,
    pub height: i32,
    pub channels: usize,
    pub data: Vec<u8>,
}

fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

pub fn decode(bytes: &[u8]) -> Png {
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "not a PNG");
    let (mut width, mut height, mut channels) = (0, 0, 0);
    let mut idat = Vec::new();
    let mut at = 8;
    while at + 8 <= bytes.len() {
        let length = be32(&bytes[at..]) as usize;
        let kind = &bytes[at + 4..at + 8];
        let body = &bytes[at + 8..at + 8 + length];
        match kind {
            b"IHDR" => {
                width = be32(body) as i32;
                height = be32(&body[4..]) as i32;
                assert!(body[8] == 8 && body[12] == 0, "only 8-bit, non-interlaced PNGs");
                channels = match body[9] {
                    0 => 1,
                    2 => 3,
                    4 => 2,
                    6 => 4,
                    other => panic!("PNG colour type {other}"),
                };
            }
            b"IDAT" => idat.extend_from_slice(body),
            b"IEND" => break,
            _ => {}
        }
        at += 12 + length;
    }
    let raw = miniz_oxide::inflate::decompress_to_vec_zlib(&idat).expect("PNG data");
    let stride = width as usize * channels;
    let mut data = vec![0u8; stride * height as usize];
    for y in 0..height as usize {
        let filter = raw[y * (stride + 1)];
        let line = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        for x in 0..stride {
            let a = if x >= channels { data[y * stride + x - channels] as i32 } else { 0 };
            let b = if y > 0 { data[(y - 1) * stride + x] as i32 } else { 0 };
            let c = if x >= channels && y > 0 { data[(y - 1) * stride + x - channels] as i32 } else { 0 };
            let predicted = match filter {
                0 => 0,
                1 => a,
                2 => b,
                3 => (a + b) / 2,
                _ => {
                    let p = a + b - c;
                    let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
                    if pa <= pb && pa <= pc {
                        a
                    } else if pb <= pc {
                        b
                    } else {
                        c
                    }
                }
            };
            data[y * stride + x] = line[x].wrapping_add(predicted as u8);
        }
    }
    Png { width, height, channels, data }
}

// RGBA pixels (as the game's u32s), whether the image has an alpha channel, and its size
pub fn decode_pixels(bytes: &[u8]) -> (i32, i32, Vec<u32>, bool) {
    let png = decode(bytes);
    let px = png
        .data
        .chunks(png.channels)
        .map(|p| match png.channels {
            4 => u32::from_le_bytes([p[0], p[1], p[2], p[3]]),
            3 => u32::from_le_bytes([p[0], p[1], p[2], 255]),
            2 => u32::from_le_bytes([p[0], p[0], p[0], p[1]]),
            _ => u32::from_le_bytes([p[0], p[0], p[0], 255]),
        })
        .collect();
    (png.width, png.height, px, png.channels == 4 || png.channels == 2)
}
