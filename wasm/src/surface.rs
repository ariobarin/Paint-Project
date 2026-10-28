// SPDX-License-Identifier: LGPL-2.1-or-later
// Ported from pygame 2.6.1 (Copyright (C) 2000-2001 Pete Shinners and the pygame contributors), which
// is under the GNU Library General Public License, version 2 or later; this file is under the same terms.
//
// The parts of pygame's Surface and Rect the game uses, with pygame's own pixel arithmetic

// Pixels are RGBA bytes read as little-endian u32s (0xAABBGGRR), the layout canvas ImageData uses
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn pixel(self) -> u32 {
        0xff00_0000 | (self.2 as u32) << 16 | (self.1 as u32) << 8 | self.0 as u32
    }

    pub fn from_pixel(p: u32) -> Color {
        Color(p as u8, (p >> 8) as u8, (p >> 16) as u8)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn collidepoint(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }

    // like pygame, the position moves by half the change, rounded toward zero (C division)
    pub fn inflate(&self, dx: i32, dy: i32) -> Rect {
        Rect::new(self.x - dx / 2, self.y - dy / 2, self.w + dx, self.h + dy)
    }

    pub fn centerx(&self) -> i32 {
        self.x + self.w / 2
    }

    pub fn centery(&self) -> i32 {
        self.y + self.h / 2
    }

    // SDL_IntersectRect: None when empty
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        if self.w <= 0 || self.h <= 0 || other.w <= 0 || other.h <= 0 {
            return None;
        }
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w).min(other.x + other.w);
        let y2 = (self.y + self.h).min(other.y + other.h);
        (x2 > x1 && y2 > y1).then(|| Rect::new(x1, y1, x2 - x1, y2 - y1))
    }
}

#[derive(Clone)]
pub struct Surface {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u32>,
    pub alpha: bool, // per-pixel alpha (SRCALPHA)
    pub clip: Rect,
}

impl Surface {
    pub fn new(w: i32, h: i32, alpha: bool) -> Surface {
        let fill = if alpha { 0 } else { 0xff00_0000 };
        Surface { w, h, px: vec![fill; (w.max(0) * h.max(0)) as usize], alpha, clip: Rect::new(0, 0, w, h) }
    }

    pub fn from_pixels(w: i32, h: i32, px: Vec<u32>, alpha: bool) -> Surface {
        Surface { w, h, px, alpha, clip: Rect::new(0, 0, w, h) }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.w, self.h)
    }

    pub fn set_clip(&mut self, clip: Option<Rect>) {
        self.clip = clip.and_then(|c| c.intersect(&self.rect())).unwrap_or(self.rect());
    }

    pub fn get_at(&self, x: i32, y: i32) -> Option<Color> {
        (x >= 0 && y >= 0 && x < self.w && y < self.h).then(|| Color::from_pixel(self.px[(y * self.w + x) as usize]))
    }

    // Surface.set_at and pygame's draw set_at: nothing outside the clip area
    pub fn set_at(&mut self, x: i32, y: i32, pixel: u32) -> bool {
        let c = self.clip;
        if x < c.x || x >= c.x + c.w || y < c.y || y >= c.y + c.h {
            return false;
        }
        self.px[(y * self.w + x) as usize] = pixel;
        true
    }

    // SDL_FillRect / Surface.fill(colour, rect): within the clip area
    pub fn fill_rect(&mut self, rect: Rect, pixel: u32) {
        let Some(r) = rect.intersect(&self.clip) else { return };
        for y in r.y..r.y + r.h {
            let row = (y * self.w) as usize;
            self.px[row + r.x as usize..row + (r.x + r.w) as usize].fill(pixel);
        }
    }

    // Surface.blit(src, (x, y)) onto this (opaque) surface: a copy, or pygame's alpha blend for SRCALPHA
    // sources: d + ((s - d) * a + s) / 256
    pub fn blit(&mut self, src: &Surface, x: i32, y: i32) {
        let Some(r) = Rect::new(x, y, src.w, src.h).intersect(&self.clip) else { return };
        for dy in r.y..r.y + r.h {
            let s_row = ((dy - y) * src.w + (r.x - x)) as usize;
            let d_row = (dy * self.w + r.x) as usize;
            let s = &src.px[s_row..s_row + r.w as usize];
            let d = &mut self.px[d_row..d_row + r.w as usize];
            if src.alpha {
                for (d, &s) in d.iter_mut().zip(s) {
                    *d = blend(s, s >> 24, *d);
                }
            } else {
                for (d, &s) in d.iter_mut().zip(s) {
                    *d = s | 0xff00_0000;
                }
            }
        }
    }

    // Blit a coverage mask in one colour (rendered text: the colour, with the coverage as alpha)
    pub fn blit_mask(&mut self, mask: &[u8], w: i32, h: i32, x: i32, y: i32, colour: Color) {
        let Some(r) = Rect::new(x, y, w, h).intersect(&self.clip) else { return };
        let s = colour.pixel();
        for dy in r.y..r.y + r.h {
            for dx in r.x..r.x + r.w {
                let a = mask[((dy - y) * w + (dx - x)) as usize] as u32;
                let d = &mut self.px[(dy * self.w + dx) as usize];
                *d = blend(s, a, *d);
            }
        }
    }

    // surface.subsurface(rect).copy()
    pub fn copy_rect(&self, rect: Rect) -> Surface {
        let mut out = Surface::new(rect.w, rect.h, self.alpha);
        for y in 0..rect.h {
            let from = ((rect.y + y) * self.w + rect.x) as usize;
            let to = (y * rect.w) as usize;
            out.px[to..to + rect.w as usize].copy_from_slice(&self.px[from..from + rect.w as usize]);
        }
        out
    }

    // transform.scale: pygame's stretch(), nearest neighbour by error accumulation
    pub fn scale(&self, w: i32, h: i32) -> Surface {
        let mut out = Surface::new(w, h, self.alpha);
        if w == 0 || h == 0 || self.w == 0 || self.h == 0 {
            return out;
        }
        let (src_w2, src_h2, dst_w2, dst_h2) = (self.w * 2, self.h * 2, w * 2, h * 2);
        let mut h_err = src_h2 - dst_h2;
        let mut src_row = 0usize;
        for row in 0..h as usize {
            let mut w_err = src_w2 - dst_w2;
            let mut src = src_row;
            for col in 0..w as usize {
                out.px[row * w as usize + col] = self.px[src];
                while w_err >= 0 {
                    src += 1;
                    w_err -= dst_w2;
                }
                w_err += src_w2;
            }
            while h_err >= 0 {
                src_row += self.w as usize;
                h_err -= dst_h2;
            }
            h_err += src_h2;
        }
        out
    }
}

// pygame's ALPHA_BLEND_COMP on each channel; the result is opaque
pub fn blend(s: u32, a: u32, d: u32) -> u32 {
    let a = a as i32;
    let channel = |shift: u32| {
        let s = ((s >> shift) & 0xff) as i32;
        let d = ((d >> shift) & 0xff) as i32;
        (((((s - d) * a + s) >> 8) + d) as u32 & 0xff) << shift
    };
    0xff00_0000 | channel(16) | channel(8) | channel(0)
}
