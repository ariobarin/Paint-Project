// write_centered_text (src/utils.py). pygame renders text with SDL_ttf; here the fixed strings come
// pre-rendered from bake.py, and the colour code (#RRGGBB, the only text that changes) is laid out the
// way SDL_ttf 2.20 does it, from the font metrics and glyph coverage bake.py took from pygame.

use crate::baked;
use crate::png;
use crate::surface::{Color, Rect, Surface};

struct Mask {
    w: i32,
    h: i32,
    alpha: Vec<u8>,
    x: i32,
    y: i32,
}

pub struct Text {
    fixed: Vec<(&'static str, Vec<Mask>)>,
    glyphs: Vec<Vec<Vec<u8>>>, // [index in HEX_SIZES][glyph] coverage
}

impl Text {
    pub fn new() -> Text {
        let fixed = baked::TEXTS
            .iter()
            .map(|(text, lines)| {
                let masks = lines
                    .iter()
                    .map(|(baked, (x, y))| {
                        let png = png::decode(baked.png);
                        Mask { w: png.width, h: png.height, alpha: png.data, x: *x, y: *y }
                    })
                    .collect();
                (*text, masks)
            })
            .collect();
        let mut at = 0;
        let glyphs = baked::HEX_SIZES
            .iter()
            .map(|&size| {
                baked::HEX_METRICS[size - 1]
                    .3
                    .iter()
                    .map(|&(_, _, _, w, rows)| {
                        let n = (w * rows) as usize;
                        at += n;
                        baked::HEX_GLYPH_COVERAGE[at - n..at].to_vec()
                    })
                    .collect()
            })
            .collect();
        Text { fixed, glyphs }
    }

    // The game only writes its fixed strings into their own boxes, and colour codes into the colour box
    pub fn write_centered_text(&self, screen: &mut Surface, text: &str, colour: Color) {
        if let Some((_, masks)) = self.fixed.iter().find(|(t, _)| *t == text) {
            for m in masks {
                screen.blit_mask(&m.alpha, m.w, m.h, m.x, m.y, colour);
            }
        } else {
            self.write_colour_code(screen, text, colour);
        }
    }

    fn write_colour_code(&self, screen: &mut Surface, text: &str, colour: Color) {
        let rect = Rect::new(420, 560, 155, 230).inflate(-5, -5);
        let idx: Vec<usize> = text.chars().map(|c| baked::HEX_GLYPHS.find(c).unwrap()).collect();

        // binary search for the largest font size that fits
        let (mut low, mut high) = (1, baked::HEX_MAX_SIZE);
        while low < high {
            let mid = (low + high + 1) / 2;
            let l = layout(&idx, mid);
            if l.h <= rect.h && l.w <= rect.w {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        let Some(slot) = baked::HEX_SIZES.iter().position(|&s| s == low) else { return };
        let l = layout(&idx, low);

        // TTF_RenderUTF8_Blended: each glyph's coverage OR-ed into the line, which is then blitted
        let (ascent, ..) = baked::HEX_METRICS[low - 1];
        let mut alpha = vec![0u8; (l.w * l.h) as usize];
        for (pen, &g) in l.pens.iter().zip(&idx) {
            let (_, left, top, w, rows) = baked::HEX_METRICS[low - 1].3[g];
            let gx = l.xstart + (pen >> 6) as i32 + left;
            let gy = l.ystart + ascent - top;
            let coverage = &self.glyphs[slot][g];
            for yy in 0..rows {
                for xx in 0..w {
                    let (x, y) = (gx + xx, gy + yy);
                    if x >= 0 && y >= 0 && x < l.w && y < l.h {
                        alpha[(y * l.w + x) as usize] |= coverage[(yy * w + xx) as usize];
                    }
                }
            }
        }
        let x = rect.x + (rect.w - l.w).div_euclid(2);
        let y = rect.y + (rect.h - l.h).div_euclid(2);
        screen.blit_mask(&alpha, l.w, l.h, x, y, colour);
    }
}

struct Layout {
    pens: Vec<i64>, // 26.6 fixed point
    w: i32,
    h: i32,
    xstart: i32,
    ystart: i32,
}

// SDL_ttf's TTF_Size_Internal with HarfBuzz: pen positions in 26.6 from the advances and the kerning
// (scaled by HarfBuzz's truncating em_mult), then the bounding box of the glyphs at those pens
fn layout(idx: &[usize], size: usize) -> Layout {
    let (ascent, height, mult, glyphs) = &baked::HEX_METRICS[size - 1];
    let mut x: i64 = 0;
    let mut pens = Vec::with_capacity(idx.len());
    for (i, &g) in idx.iter().enumerate() {
        pens.push(x);
        x += glyphs[g].0 as i64 * 64;
        if let Some(&next) = idx.get(i + 1) {
            x += (baked::HEX_KERN[g][next] as i64 * mult) >> 16;
        }
    }
    let (mut minx, mut maxx, mut miny, mut maxy) = (0, 0, 0, *height);
    for (&pen, &g) in pens.iter().zip(idx) {
        let (_, left, top, w, rows) = glyphs[g];
        let px = (pen >> 6) as i32 + left;
        let py = ascent - top;
        minx = minx.min(px);
        maxx = maxx.max(px + w);
        miny = miny.min(py);
        maxy = maxy.max(py + rows);
    }
    maxx = maxx.max((x >> 6) as i32);
    Layout { pens, w: maxx - minx, h: maxy - miny, xstart: -minx, ystart: -miny }
}
