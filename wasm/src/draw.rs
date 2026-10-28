// pygame.draw (pygame 2.6.1, src_c/draw.c): the shapes the game draws, ported line for line so they
// cover exactly the same pixels

use crate::surface::{Rect, Surface};

fn drawhorzline(surf: &mut Surface, color: u32, x1: i32, y1: i32, x2: i32) {
    let row = (y1 * surf.w) as usize;
    surf.px[row + x1 as usize..=row + x2 as usize].fill(color);
}

fn drawhorzlineclip(surf: &mut Surface, color: u32, mut x1: i32, y1: i32, mut x2: i32) {
    let c = surf.clip;
    if y1 < c.y || y1 >= c.y + c.h {
        return;
    }
    if x2 < x1 {
        std::mem::swap(&mut x1, &mut x2);
    }
    x1 = x1.max(c.x);
    x2 = x2.min(c.x + c.w - 1);
    if x2 < c.x || x1 >= c.x + c.w {
        return;
    }
    if x1 == x2 {
        surf.set_at(x1, y1, color);
        return;
    }
    drawhorzline(surf, color, x1, y1, x2);
}

fn inside_clip(surf: &Surface, x: i32, y: i32) -> bool {
    let c = surf.clip;
    !(x < c.x || x >= c.x + c.w || y < c.y || y >= c.y + c.h)
}

fn sign(x: i32, y: i32) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else {
        y
    }
}

// C's (int) cast of p * n +- 0.5, rounding away from zero
fn round_away(v: f64) -> i32 {
    (if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i32
}

fn clip_line(surf: &Surface, x1: &mut i32, y1: &mut i32, x2: &mut i32, y2: &mut i32) -> bool {
    let c = surf.clip;
    let p1 = *x1 - *x2;
    let p2 = -p1;
    let p3 = *y1 - *y2;
    let p4 = -p3;
    let q1 = *x1 - c.x;
    let q2 = c.w + c.x - *x1;
    let q3 = *y1 - c.y;
    let q4 = c.h + c.y - *y1;
    let old_x1 = *x1;
    let old_y1 = *y1;
    let mut nmax = 0.0f64;
    let mut pmin = 1.0f64;

    if (p1 == 0 && q1 < 0) || (p2 == 0 && q2 < 0) || (p3 == 0 && q3 < 0) || (p4 == 0 && q4 < 0) {
        return false;
    }
    if p1 != 0 {
        let r1 = q1 as f64 / p1 as f64;
        let r2 = q2 as f64 / p2 as f64;
        if p1 < 0 {
            if r1 > nmax {
                nmax = r1;
            }
            if r2 < pmin {
                pmin = r2;
            }
        } else {
            if r2 > nmax {
                nmax = r2;
            }
            if r1 < pmin {
                pmin = r1;
            }
        }
    }
    if p3 != 0 {
        let r1 = q3 as f64 / p3 as f64;
        let r2 = q4 as f64 / p4 as f64;
        if p3 < 0 {
            if r1 > nmax {
                nmax = r1;
            }
            if r2 < pmin {
                pmin = r2;
            }
        } else {
            if r2 > nmax {
                nmax = r2;
            }
            if r1 < pmin {
                pmin = r1;
            }
        }
    }
    if nmax > pmin {
        return false;
    }
    *x1 = old_x1 + round_away(p2 as f64 * nmax);
    *y1 = old_y1 + round_away(p4 as f64 * nmax);
    *x2 = old_x1 + round_away(p2 as f64 * pmin);
    *y2 = old_y1 + round_away(p4 as f64 * pmin);
    true
}

fn draw_line(surf: &mut Surface, mut x1: i32, mut y1: i32, x2: i32, y2: i32, color: u32) {
    if x1 == x2 && y1 == y2 {
        // Single point
        surf.set_at(x1, y1, color);
        return;
    }
    if y1 == y2 {
        // Horizontal line
        let dx = if x1 < x2 { 1 } else { -1 };
        for sx in 0..=(x1 - x2).abs() {
            surf.set_at(x1 + dx * sx, y1, color);
        }
        return;
    }
    if x1 == x2 {
        // Vertical line
        let dy = if y1 < y2 { 1 } else { -1 };
        for sy in 0..=(y1 - y2).abs() {
            surf.set_at(x1, y1 + dy * sy, color);
        }
        return;
    }
    let dx = (x2 - x1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let dy = (y2 - y1).abs();
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = (if dx > dy { dx } else { -dy }) / 2;
    while x1 != x2 || y1 != y2 {
        surf.set_at(x1, y1, color);
        let e2 = err;
        if e2 > -dx {
            err -= dy;
            x1 += sx;
        }
        if e2 < dy {
            err += dx;
            y1 += sy;
        }
    }
    surf.set_at(x2, y2, color);
}

#[allow(unused_assignments)] // kept as in draw.c, which also steps the position after its last use
fn draw_line_width(surf: &mut Surface, color: u32, mut x1: i32, mut y1: i32, mut x2: i32, mut y2: i32, width: i32) {
    let end_x = x2;
    let end_y = y2;
    // Decide which direction to grow (width/thickness)
    let xinc = (x1 - x2).abs() <= (y1 - y2).abs();
    let dx = (x2 - x1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let dy = (y2 - y1).abs();
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = (if dx > dy { dx } else { -dy }) / 2;

    if !clip_line(surf, &mut x1, &mut y1, &mut x2, &mut y2) {
        return;
    }
    if width == 1 {
        draw_line(surf, x1, y1, x2, y2, color);
        return;
    }
    let (mut left_top, mut right_bottom) =
        if xinc { (x1 - (width - 1) / 2, x1 + width / 2) } else { (y1 - (width - 1) / 2, y1 + width / 2) };
    while sign(x1 - x2, sx) != sx || sign(y1 - y2, sy) != sy {
        if xinc {
            drawhorzlineclip(surf, color, left_top, y1, right_bottom);
        } else {
            for y in left_top..=right_bottom {
                surf.set_at(x1, y, color);
            }
        }
        let e2 = err;
        if e2 > -dx {
            err -= dy;
            x1 += sx;
            if xinc {
                left_top += sx;
                right_bottom += sx;
            }
        }
        if e2 < dy {
            err += dx;
            y1 += sy;
            if !xinc {
                left_top += sy;
                right_bottom += sy;
            }
        }
    }
    if xinc {
        while y1 != end_y && (inside_clip(surf, left_top, y1) || inside_clip(surf, right_bottom, y1)) {
            drawhorzlineclip(surf, color, left_top, y1, right_bottom);
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x1 += sx;
                left_top += sx;
                right_bottom += sx;
            }
            if e2 < dy {
                err += dx;
                y1 += sy;
            }
        }
        drawhorzlineclip(surf, color, left_top, y1, right_bottom);
    } else {
        while x1 != end_x && (inside_clip(surf, x1, left_top) || inside_clip(surf, x1, right_bottom)) {
            for y in left_top..=right_bottom {
                surf.set_at(x1, y, color);
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x1 += sx;
            }
            if e2 < dy {
                err += dx;
                y1 += sy;
                left_top += sy;
                right_bottom += sy;
            }
        }
        for y in left_top..=right_bottom {
            surf.set_at(x1, y, color);
        }
    }
}

// draw.line
pub fn line(surf: &mut Surface, color: u32, start: (i32, i32), end: (i32, i32), width: i32) {
    if width < 1 {
        return;
    }
    draw_line_width(surf, color, start.0, start.1, end.0, end.1, width);
}

fn draw_circle_filled(surf: &mut Surface, x0: i32, y0: i32, radius: i32, color: u32) {
    let mut f = 1 - radius;
    let mut ddf_x = 0;
    let mut ddf_y = -2 * radius;
    let mut x = 0;
    let mut y = radius;

    while x < y {
        if f >= 0 {
            y -= 1;
            ddf_y += 2;
            f += ddf_y;
        }
        x += 1;
        ddf_x += 2;
        f += ddf_x + 1;

        // only draw a line if y-step is about to be decreased
        if f >= 0 {
            drawhorzlineclip(surf, color, x0 - x, y0 + y - 1, x0 + x - 1);
            drawhorzlineclip(surf, color, x0 - x, y0 - y, x0 + x - 1);
        }
        drawhorzlineclip(surf, color, x0 - y, y0 + x - 1, x0 + y - 1);
        drawhorzlineclip(surf, color, x0 - y, y0 - x, x0 + y - 1);
    }
}

// draw.circle, filled (the only kind the game draws)
pub fn circle(surf: &mut Surface, color: u32, center: (i32, i32), radius: i32) {
    let (posx, posy) = center;
    if radius < 1 {
        return;
    }
    let c = surf.clip;
    if posx > c.x + c.w + radius || posx < c.x - radius || posy > c.y + c.h + radius || posy < c.y - radius {
        return;
    }
    draw_circle_filled(surf, posx, posy, radius, color);
}

fn draw_rect(surf: &mut Surface, x1: i32, y1: i32, x2: i32, y2: i32, width: i32, color: u32) {
    for i in 0..width {
        drawhorzlineclip(surf, color, x1, y1 + i, x2);
        drawhorzlineclip(surf, color, x1, y2 - i, x2);
    }
    for i in 0..(y2 - y1) - 2 * width + 1 {
        drawhorzlineclip(surf, color, x1, y1 + width + i, x1 + width - 1);
        drawhorzlineclip(surf, color, x2 - width + 1, y1 + width + i, x2);
    }
}

// draw.rect (no rounded corners): filled when width is 0, or when the outline would fill it anyway
pub fn rect(surf: &mut Surface, color: u32, r: Rect, width: i32) {
    if width < 0 {
        return;
    }
    let Some(clipped) = r.intersect(&surf.clip) else { return };
    if width > 0 && width * 2 < clipped.w && width * 2 < clipped.h {
        draw_rect(surf, r.x, r.y, r.x + r.w - 1, r.y + r.h - 1, width, color);
    } else {
        surf.fill_rect(clipped, color);
    }
}

fn draw_ellipse_filled(surf: &mut Surface, mut x0: i32, mut y0: i32, mut width: i32, mut height: i32, color: u32) {
    if width == 1 {
        draw_line(surf, x0, y0, x0, y0 + height - 1, color);
        return;
    }
    if height == 1 {
        drawhorzlineclip(surf, color, x0, y0, x0 + width - 1);
        return;
    }
    x0 += width / 2;
    y0 += height / 2;
    let x_offset = (width + 1) % 2;
    let y_offset = (height + 1) % 2;
    width /= 2;
    height /= 2;
    let (w, h) = (width as i64, height as i64);
    let mut x: i64 = 0;
    let mut y: i64 = h;
    let mut d1 = (height * height - width * width * height) as f64 + 0.25 * width as f64 * width as f64;
    let mut dx = 2 * h * h * x;
    let mut dy = 2 * w * w * y;
    while dx < dy {
        drawhorzlineclip(surf, color, x0 - x as i32, y0 - y as i32, x0 + x as i32 - x_offset);
        drawhorzlineclip(surf, color, x0 - x as i32, y0 + y as i32 - y_offset, x0 + x as i32 - x_offset);
        if d1 < 0.0 {
            x += 1;
            dx += 2 * h * h;
            d1 = d1 + dx as f64 + (h * h) as f64;
        } else {
            x += 1;
            y -= 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d1 = d1 + dx as f64 - dy as f64 + (h * h) as f64;
        }
    }
    let mut d2 = (h as f64 * h as f64) * ((x as f64 + 0.5) * (x as f64 + 0.5)) + (w as f64 * w as f64) * ((y - 1) * (y - 1)) as f64
        - w as f64 * w as f64 * h as f64 * h as f64;
    while y >= 0 {
        drawhorzlineclip(surf, color, x0 - x as i32, y0 - y as i32, x0 + x as i32 - x_offset);
        drawhorzlineclip(surf, color, x0 - x as i32, y0 + y as i32 - y_offset, x0 + x as i32 - x_offset);
        if d2 > 0.0 {
            y -= 1;
            dy -= 2 * w * w;
            d2 = d2 + (w * w) as f64 - dy as f64;
        } else {
            y -= 1;
            x += 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d2 = d2 + dx as f64 - dy as f64 + (w * w) as f64;
        }
    }
}

fn draw_ellipse_thickness(surf: &mut Surface, mut x0: i32, mut y0: i32, mut width: i32, mut height: i32, thickness: i32, color: u32) {
    x0 += width / 2;
    y0 += height / 2;
    let x_offset = (width + 1) % 2;
    let y_offset = (height + 1) % 2;
    width /= 2;
    height /= 2;
    let (w, h, t) = (width as i64, height as i64, thickness as i64);
    let (wi, hi) = (w - t, h - t); // the inner ellipse's half sizes
    let mut line = true;
    let mut x: i64 = 0;
    let mut y: i64 = h;
    let mut x_inner: i64 = 0;
    let mut y_inner: i64 = h - t;
    let mut d1 = (height * height - width * width * height) as f64 + 0.25 * width as f64 * width as f64;
    let hti = height - thickness;
    let wti = width - thickness;
    let mut d1_inner = (hti * hti - wti * wti * hti) as f64 + 0.25 * wti as f64 * wti as f64;
    let mut d2_inner = 0.0f64;
    let mut dx = 2 * h * h * x;
    let mut dy = 2 * w * w * y;
    let mut dx_inner = 2 * hi * hi * x_inner;
    let mut dy_inner = 2 * wi * wi * y_inner;

    let strips = |surf: &mut Surface, line: bool, x: i64, y: i64, x_inner: i64| {
        let (x, y, x_inner) = (x as i32, y as i32, x_inner as i32);
        if line {
            drawhorzlineclip(surf, color, x0 - x, y0 - y, x0 + x - x_offset);
            drawhorzlineclip(surf, color, x0 - x, y0 + y - y_offset, x0 + x - x_offset);
        } else {
            drawhorzlineclip(surf, color, x0 - x, y0 - y, x0 - x_inner);
            drawhorzlineclip(surf, color, x0 - x, y0 + y - y_offset, x0 - x_inner);
            drawhorzlineclip(surf, color, x0 + x - x_offset, y0 - y, x0 + x_inner - x_offset);
            drawhorzlineclip(surf, color, x0 + x - x_offset, y0 + y - y_offset, x0 + x_inner - x_offset);
        }
    };

    while dx < dy {
        strips(surf, line, x, y, x_inner);
        if d1 < 0.0 {
            x += 1;
            dx += 2 * h * h;
            d1 = d1 + dx as f64 + (h * h) as f64;
        } else {
            x += 1;
            y -= 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d1 = d1 + dx as f64 - dy as f64 + (h * h) as f64;
            if line && y < h - t {
                line = false;
            }
            if !line && dx_inner < dy_inner {
                while d1_inner < 0.0 {
                    x_inner += 1;
                    dx_inner += 2 * hi * hi;
                    d1_inner = d1_inner + dx_inner as f64 + (hi * hi) as f64;
                }
                x_inner += 1;
                y_inner -= 1;
                dx_inner += 2 * hi * hi;
                dy_inner -= 2 * wi * wi;
                d1_inner = d1_inner + dx_inner as f64 - dy_inner as f64 + (hi * hi) as f64;
            }
        }
    }
    let mut d2 = (h as f64 * h as f64) * ((x as f64 + 0.5) * (x as f64 + 0.5)) + (w as f64 * w as f64) * ((y - 1) * (y - 1)) as f64
        - w as f64 * w as f64 * h as f64 * h as f64;
    while y >= 0 {
        strips(surf, line, x, y, x_inner);
        if d2 > 0.0 {
            y -= 1;
            dy -= 2 * w * w;
            d2 = d2 + (w * w) as f64 - dy as f64;
        } else {
            y -= 1;
            x += 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d2 = d2 + dx as f64 - dy as f64 + (w * w) as f64;
        }
        if line && y < h - t {
            line = false;
        }
        if !line {
            if dx_inner < dy_inner {
                while d1_inner < 0.0 {
                    x_inner += 1;
                    dx_inner += 2 * hi * hi;
                    d1_inner = d1_inner + dx_inner as f64 + (hi * hi) as f64;
                }
                x_inner += 1;
                y_inner -= 1;
                dx_inner += 2 * hi * hi;
                dy_inner -= 2 * wi * wi;
                d1_inner = d1_inner + dx_inner as f64 - dy_inner as f64 + (hi * hi) as f64;
            } else if y_inner >= 0 {
                if d2_inner == 0.0 {
                    d2_inner = (hi as f64 * hi as f64) * ((x_inner as f64 + 0.5) * (x_inner as f64 + 0.5))
                        + (wi as f64 * wi as f64) * ((y_inner - 1) * (y_inner - 1)) as f64
                        - wi as f64 * wi as f64 * hi as f64 * hi as f64;
                }
                if d2_inner > 0.0 {
                    y_inner -= 1;
                    dy_inner -= 2 * wi * wi;
                    d2_inner = d2_inner + (wi * wi) as f64 - dy_inner as f64;
                } else {
                    y_inner -= 1;
                    x_inner += 1;
                    dx_inner += 2 * hi * hi;
                    dy_inner -= 2 * wi * wi;
                    d2_inner = d2_inner + dx_inner as f64 - dy_inner as f64 + (wi * wi) as f64;
                }
            }
        }
    }
}

// draw.ellipse
pub fn ellipse(surf: &mut Surface, color: u32, r: Rect, width: i32) {
    if width < 0 {
        return;
    }
    if width == 0 || width >= (r.w / 2 + r.w % 2).min(r.h / 2 + r.h % 2) {
        draw_ellipse_filled(surf, r.x, r.y, r.w, r.h, color);
    } else {
        draw_ellipse_thickness(surf, r.x, r.y, r.w, r.h, width - 1, color);
    }
}
