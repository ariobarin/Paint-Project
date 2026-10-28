// The images: decoded from what bake.py made, and (after the first frame) the stamps and backgrounds

use crate::baked::{self, Baked};
use crate::png;
use crate::surface::Surface;
use crate::text::Text;

pub struct Art {
    pub title: Surface,
    pub tool_icons: Vec<(Surface, (i32, i32))>,
    pub button_icons: Vec<(Surface, (i32, i32))>, // save, load, trash, undo, redo
    pub gradients: Vec<Surface>,
    pub stamp_small: Vec<Surface>, // fitted into the previous / next buttons
    pub stamp_tool: Vec<Surface>,  // fitted into the stamp tool's button
    pub background_thumbs: Vec<Surface>,
    pub stamps: Vec<Option<Surface>>,      // the originals
    pub backgrounds: Vec<Option<Surface>>, // each scaled to cover the window (scale_img_to_rect), see BACKGROUNDS
    pub text: Text,
}

pub fn surface(baked: &Baked) -> Surface {
    let (w, h, px, alpha) = png::decode_pixels(baked.png);
    Surface::from_pixels(w, h, px, alpha)
}

impl Art {
    pub fn new() -> Art {
        let placed = |list: &[(Baked, (i32, i32))]| list.iter().map(|(b, at)| (surface(b), *at)).collect();
        Art {
            title: surface(&baked::TITLE),
            tool_icons: placed(&baked::TOOL_ICONS),
            button_icons: placed(&baked::BUTTON_ICONS),
            gradients: baked::GRADIENTS.iter().map(surface).collect(),
            stamp_small: baked::STAMP_PREVIEWS.iter().map(|(s, _)| surface(s)).collect(),
            stamp_tool: baked::STAMP_PREVIEWS.iter().map(|(_, t)| surface(t)).collect(),
            background_thumbs: baked::BACKGROUND_THUMBS.iter().map(surface).collect(),
            stamps: (0..baked::STAMPS.len()).map(|_| None).collect(),
            backgrounds: (0..baked::BACKGROUNDS.len()).map(|_| None).collect(),
            text: Text::new(),
        }
    }

    pub fn complete(&self) -> bool {
        self.stamps.iter().all(Option::is_some) && self.backgrounds.iter().all(Option::is_some)
    }
}
