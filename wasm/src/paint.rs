// main.py and the src/ modules: the paint program, frame by frame, in the order the Python version
// draws and decides things (including where it uses last frame's settings)

use std::collections::VecDeque;

use crate::art::Art;
use crate::draw;
use crate::random::Random;
use crate::surface::{Color, Rect, Surface};

// src/config.py
const UNSELECED_COLOUR: Color = Color(140, 80, 20); // the border of buttons when not hovered
const HOVER_COLOUR: Color = Color(130, 190, 255); // the border of buttons when hovered
const SELECTED_COLOUR: Color = Color(255, 240, 90); // the border of buttons when selected
const BG_COLOUR: Color = Color(255, 120, 30); // the background of buttons
const OUTLINE_COLOUR: Color = Color(255, 255, 255);
const BLACK: Color = Color(0, 0, 0);
const WHITE: Color = Color(255, 255, 255);

const CANVAS: Rect = Rect::new(110, 100, 1080, 450);

// src/background.py
const FORWARD_RECT: Rect = Rect::new(875, 560, 195, 110);
const BACKWARD_RECT: Rect = Rect::new(875, 680, 195, 110);

// src/stamp.py
const PREVIOUS_PREVIEW: Rect = Rect::new(585, 690, 100, 100);
const NEXT_PREVIEW: Rect = Rect::new(690, 690, 100, 100);

// src/undo_redo.py
const UNDO_RECT: Rect = Rect::new(10, 10, 40, 40);
const REDO_RECT: Rect = Rect::new(60, 10, 40, 40);

// src/buttons.py, in the order of its dict
const FILL: usize = 0;
const SAVE: usize = 1;
const LOAD: usize = 2;
const TRASH: usize = 3;
const BUTTONS: [Rect; 4] = [Rect::new(10, 455, 90, 90), Rect::new(1080, 560, 110, 110), Rect::new(1080, 680, 110, 110), Rect::new(110, 10, 40, 40)];

// src/sliders.py
const RED_SLIDER: Rect = Rect::new(10, 675, 400, 35);
const GREEN_SLIDER: Rect = Rect::new(10, 715, 400, 35);
const BLUE_SLIDER: Rect = Rect::new(10, 755, 400, 35);
const COLOUR_RECT: Rect = Rect::new(420, 560, 155, 230);
const RADIUS_SLIDER: Rect = Rect::new(10, 635, 400, 35);
const STAMP_SLIDER: Rect = Rect::new(795, 560, 70, 230);

// src/tools.py, in the order of its dict
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Pencil,
    Eraser,
    Spray,
    PaintBrush,
    Line,
    Rect,
    Ellipse,
    EyeDropper,
    Fireworks,
    AirBrush,
    PaintBucket,
    FillScreen,
    Stamp,
    Select,
}

const TOOLS: [(Tool, Rect, &str); 14] = [
    (Tool::Pencil, Rect::new(10, 105, 40, 40), "Draws lines\nClick and drag to draw"),
    (Tool::Eraser, Rect::new(60, 105, 40, 40), "Erases lines\nClick and drag to erase"),
    (Tool::Spray, Rect::new(10, 155, 40, 40), "Sprays paint\nClick and drag to spray"),
    (Tool::PaintBrush, Rect::new(60, 155, 40, 40), "Paints lines\nClick and drag to paint"),
    (Tool::Line, Rect::new(10, 205, 40, 40), "Draws lines\nClick and drag to draw\nHold shift to snap to 90 degrees"),
    (Tool::Rect, Rect::new(60, 205, 40, 40), "Draws rectangles\nClick and drag to draw\nHold shift to draw squares"),
    (Tool::Ellipse, Rect::new(10, 255, 40, 40), "Draws ellipses\nClick and drag to draw\nHold shift to draw circles"),
    (Tool::EyeDropper, Rect::new(60, 255, 40, 40), "Selects a colour\nClick to select a colour"),
    (Tool::Fireworks, Rect::new(10, 305, 40, 40), "Draws fireworks\nClick to draw"),
    (Tool::AirBrush, Rect::new(60, 305, 40, 40), "Draws air brush\nClick to draw"),
    (Tool::PaintBucket, Rect::new(10, 355, 40, 40), "Fills an area with colour\nClick to fill"),
    (Tool::FillScreen, Rect::new(60, 355, 40, 40), "Fills the screen with colour\nClick to fill"),
    (Tool::Stamp, Rect::new(585, 560, 205, 125), "Stamps an image\nDrag and drop to stamp"),
    (Tool::Select, Rect::new(10, 405, 90, 40), "Selects an area\nClick and drag to select\nGrab resize handles to resize"),
];
const DESCRIPTION_RECT: Rect = Rect::new(10, 560, 400, 65);
const REDRAW_TOOLS: [Tool; 5] = [Tool::Line, Tool::Rect, Tool::Ellipse, Tool::Stamp, Tool::Select];

fn tool_rect(tool: Tool) -> Rect {
    TOOLS.iter().find(|t| t.0 == tool).unwrap().1
}

// src/selection.py
const SELECTED: Color = Color(0, 255, 0);
const UNSELECTED: Color = Color(255, 0, 0);
const RHS: i32 = 15; // resize handle size
const RHS2: i32 = RHS / 2;

// One frame's input: what pygame's event queue, mouse and keyboard would report
#[derive(Default, Clone)]
pub struct Input {
    pub events: Vec<Event>,
    pub mx: i32,
    pub my: i32,
    pub mb: [bool; 3],
    pub shift: bool,
    pub ctrl: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    MouseDown,
    MouseUp,
    Key(char),
}

#[derive(Clone, Copy)]
struct ButtonData {
    mb: [bool; 3],
    mx: i32,
    my: i32,
    omx: i32,
    omy: i32,
    dmx: i32,
    dmy: i32,
    mouse_up: bool,
    mouse_down: bool,
    shift: bool,
    ctrl: bool,
    key_x: bool,
    key_y: bool,
    key_z: bool,
}

#[derive(Clone, Copy)]
struct ToolSettings {
    tool: Tool,
    col: Color,
    radius: i32,
    fill: bool,
    stamp: usize,
    stamp_size: i32,
}

// The undo history keeps every version of the canvas, like the Python list; all but the newest are
// stored run-length encoded, which is the same picture in a fraction of the memory
enum Snapshot {
    Plain(Surface),
    Packed(Vec<(u32, u32)>),
}

impl Snapshot {
    fn pack(surface: Surface) -> Snapshot {
        let mut runs: Vec<(u32, u32)> = Vec::new();
        for &p in &surface.px {
            match runs.last_mut() {
                Some((value, count)) if *value == p => *count += 1,
                _ => runs.push((p, 1)),
            }
        }
        Snapshot::Packed(runs)
    }

    fn surface(self) -> Surface {
        match self {
            Snapshot::Plain(s) => s,
            Snapshot::Packed(runs) => {
                let mut px = Vec::with_capacity((CANVAS.w * CANVAS.h) as usize);
                for (value, count) in runs {
                    px.extend(std::iter::repeat_n(value, count as usize));
                }
                Surface::from_pixels(CANVAS.w, CANVAS.h, px, false)
            }
        }
    }
}

struct History {
    list: Vec<Snapshot>,
}

impl History {
    fn new(first: Surface) -> History {
        History { list: vec![Snapshot::Plain(first)] }
    }

    fn len(&self) -> usize {
        self.list.len()
    }

    fn push(&mut self, s: Surface) {
        if let Some(top) = self.list.pop() {
            let top = match top {
                Snapshot::Plain(s) => Snapshot::pack(s),
                packed => packed,
            };
            self.list.push(top);
        }
        self.list.push(Snapshot::Plain(s));
    }

    fn pop(&mut self) -> Option<Surface> {
        let top = self.list.pop()?.surface();
        if let Some(next) = self.list.pop() {
            self.list.push(Snapshot::Plain(next.surface()));
        }
        Some(top)
    }

    fn clear(&mut self) {
        self.list.clear();
    }

    fn last(&self) -> &Surface {
        match self.list.last() {
            Some(Snapshot::Plain(s)) => s,
            _ => unreachable!("the newest snapshot is never packed"),
        }
    }
}

pub struct Paint {
    pub screen: Surface,
    rng: Random,
    background_list: Vec<usize>, // indices of the backgrounds, rotated like the Python list
    stamp_list: Vec<usize>,
    omx: i32,
    omy: i32,
    dmx: i32,
    dmy: i32,
    tool: Tool,
    col: Color,
    radius: i32,
    stamp_size: i32,
    fill: bool,
    tool_settings: ToolSettings,
    undo_list: History,
    redo_list: History,

    // src/selection.py's globals
    select_state: i32,
    og_start: (i32, i32),
    og_rect: Rect,
    move_start: (i32, i32),
    resize_start: (i32, i32),
    resize_corner: i32,
    clipboard: Option<Surface>,

    // the browser's save and load (src/webcompat.py)
    pub saved: Option<Surface>, // the canvas, to download
    pub load_requested: bool,
    pub picked_image: Option<Surface>,
}

impl Paint {
    pub fn new(first_background: &Surface, art: &Art, seed: u64) -> Paint {
        let mut screen = Surface::new(1200, 800, false);
        // initialize_background: the first background as it is, then the title
        screen.blit(first_background, 0, 0);
        draw_title(&mut screen, art);
        draw::rect(&mut screen, WHITE.pixel(), CANVAS, 0);
        let first = screen.copy_rect(CANVAS);
        let tool_settings = ToolSettings { tool: Tool::Pencil, col: Color(99, 179, 212), radius: 10, fill: false, stamp: 0, stamp_size: 10 };
        Paint {
            screen,
            rng: Random::new(seed),
            background_list: (0..art.background_thumbs.len()).collect(),
            stamp_list: (0..art.stamp_small.len()).collect(),
            omx: 0,
            omy: 0,
            dmx: 0,
            dmy: 0,
            tool: Tool::Pencil,
            col: Color(99, 179, 212),
            radius: 10,
            stamp_size: 10,
            fill: false,
            tool_settings,
            undo_list: History::new(first),
            redo_list: History { list: Vec::new() },
            select_state: 0,
            og_start: (0, 0),
            og_rect: Rect::new(0, 0, 0, 0),
            move_start: (0, 0),
            resize_start: (0, 0),
            resize_corner: 0,
            clipboard: None,
            saved: None,
            load_requested: false,
            picked_image: None,
        }
    }

    // One pass of main.py's while loop
    pub fn frame(&mut self, input: &Input, art: &Art) {
        let (mut done_move, mut mouse_up, mut mouse_down, mut key_x, mut key_y, mut key_z) = (false, false, false, false, false, false);
        for evt in &input.events {
            match evt {
                Event::MouseDown => {
                    mouse_down = true;
                    (self.dmx, self.dmy) = (input.mx, input.my);
                }
                Event::MouseUp => mouse_up = true,
                Event::Key('x') => key_x = true,
                Event::Key('y') => key_y = true,
                Event::Key('z') => key_z = true,
                Event::Key(_) => {}
            }
        }
        let b = ButtonData {
            mb: input.mb,
            mx: input.mx,
            my: input.my,
            omx: self.omx,
            omy: self.omy,
            dmx: self.dmx,
            dmy: self.dmy,
            mouse_up,
            mouse_down,
            shift: input.shift,
            ctrl: input.ctrl,
            key_x,
            key_y,
            key_z,
        };

        // draw the background, stamps and buttons (no music player: the browser build has no music)
        self.background_manager(&b, art);
        self.stamp_manager(&b, art);

        // check for tool changes
        self.tool = self.tool_change(&b);

        // draw the colour palette and radius selector
        self.col = self.draw_colour_palette(&b, art);
        self.radius = self.draw_radius_selector(&b, art);

        // draw the buttons and check if they are clicked
        let updated;
        (self.fill, updated) = self.button_manager(&b, art);

        // undo and redo
        self.undo_manager(&b, art);

        // draw the tools and check if they are clicked
        self.draw_tools(&b, art);

        if updated {
            self.undo_list.push(self.screen.copy_rect(CANVAS));
            self.redo_list.clear();
        }

        if REDRAW_TOOLS.contains(&self.tool) {
            self.screen.blit(self.undo_list.last(), CANVAS.x, CANVAS.y);
        }

        // draw the stamp sizer and update the tool settings
        let dont_update;
        (self.stamp_size, dont_update) = self.draw_stamp_sizer(&b, art);
        self.tool_settings = ToolSettings {
            tool: self.tool,
            col: self.col,
            radius: self.radius,
            fill: self.fill,
            stamp: self.stamp_list[0],
            stamp_size: self.stamp_size,
        };

        // check if the user is using the tool
        if CANVAS.collidepoint(self.dmx, self.dmy) || self.tool_settings.tool == Tool::Select {
            self.screen.set_clip(Some(CANVAS));
            (self.tool_settings.col, done_move) = self.tool_manager(&b, art);
            self.screen.set_clip(None);
        }

        // add the new canvas to the undo list if the user has finished using the tool
        if (mouse_up && CANVAS.collidepoint(self.dmx, self.dmy) && self.tool != Tool::Select || done_move) && !dont_update {
            self.undo_list.push(self.screen.copy_rect(CANVAS));
            self.redo_list.clear();
        }

        // draw border around canvas
        draw::rect(&mut self.screen, BLACK.pixel(), CANVAS, 5);
        (self.omx, self.omy) = (input.mx, input.my);
    }

    // --- src/background.py ---

    fn background_manager(&mut self, b: &ButtonData, art: &Art) {
        if FORWARD_RECT.collidepoint(b.mx, b.my) && b.mouse_up {
            let first = self.background_list.remove(0);
            self.background_list.push(first);
            self.change_background(art);
        }
        if BACKWARD_RECT.collidepoint(b.mx, b.my) && b.mouse_up {
            let last = self.background_list.pop().unwrap();
            self.background_list.insert(0, last);
            self.change_background(art);
        }
        self.draw_background_buttons(b, art);
    }

    fn change_background(&mut self, art: &Art) {
        // scale_img_to_rect
        if let Some(background) = &art.backgrounds[self.background_list[0]] {
            let (_, _, (x, y)) = crate::baked::BACKGROUNDS[self.background_list[0]];
            self.screen.blit(background, x, y);
        }
        self.screen.blit(self.undo_list.last(), CANVAS.x, CANVAS.y);
        draw_title(&mut self.screen, art);
    }

    fn draw_background_buttons(&mut self, b: &ButtonData, art: &Art) {
        let s = &mut self.screen;
        draw::rect(s, BG_COLOUR.pixel(), FORWARD_RECT, 0);
        draw::rect(s, BG_COLOUR.pixel(), BACKWARD_RECT, 0);
        fit_img_to_rect(s, &art.background_thumbs[self.background_list[1]], FORWARD_RECT);
        fit_img_to_rect(s, &art.background_thumbs[*self.background_list.last().unwrap()], BACKWARD_RECT);
        draw::rect(s, UNSELECED_COLOUR.pixel(), FORWARD_RECT, 2);
        draw::rect(s, UNSELECED_COLOUR.pixel(), BACKWARD_RECT, 2);
        if FORWARD_RECT.collidepoint(b.mx, b.my) {
            draw::rect(s, HOVER_COLOUR.pixel(), FORWARD_RECT, 2);
        }
        if BACKWARD_RECT.collidepoint(b.mx, b.my) {
            draw::rect(s, HOVER_COLOUR.pixel(), BACKWARD_RECT, 2);
        }
    }

    // --- src/stamp.py ---

    fn stamp_manager(&mut self, b: &ButtonData, art: &Art) {
        if NEXT_PREVIEW.collidepoint(b.mx, b.my) && b.mouse_up {
            let first = self.stamp_list.remove(0);
            self.stamp_list.push(first);
        }
        if PREVIOUS_PREVIEW.collidepoint(b.mx, b.my) && b.mouse_up {
            let last = self.stamp_list.pop().unwrap();
            self.stamp_list.insert(0, last);
        }
        let s = &mut self.screen;
        draw::rect(s, BLACK.pixel(), PREVIOUS_PREVIEW, 0);
        draw::rect(s, BLACK.pixel(), NEXT_PREVIEW, 0);
        fit_img_to_rect(s, &art.stamp_small[*self.stamp_list.last().unwrap()], PREVIOUS_PREVIEW);
        fit_img_to_rect(s, &art.stamp_small[self.stamp_list[1]], NEXT_PREVIEW);
        draw::rect(s, UNSELECED_COLOUR.pixel(), NEXT_PREVIEW, 2);
        draw::rect(s, UNSELECED_COLOUR.pixel(), PREVIOUS_PREVIEW, 2);
        if NEXT_PREVIEW.collidepoint(b.mx, b.my) {
            draw::rect(s, HOVER_COLOUR.pixel(), NEXT_PREVIEW, 2);
        }
        if PREVIOUS_PREVIEW.collidepoint(b.mx, b.my) {
            draw::rect(s, HOVER_COLOUR.pixel(), PREVIOUS_PREVIEW, 2);
        }
    }

    // --- src/tools.py ---

    fn tool_change(&mut self, b: &ButtonData) -> Tool {
        if b.mouse_up {
            if let Some((tool, ..)) = TOOLS.iter().find(|(_, rect, _)| rect.collidepoint(b.mx, b.my)) {
                self.select_state = 0;
                self.move_start = (0, 0);
                self.og_start = (0, 0);
                self.clipboard = None;
                return *tool;
            }
        }
        self.tool_settings.tool
    }

    fn draw_tools(&mut self, b: &ButtonData, art: &Art) {
        let mut description = "Hover over a tool to see its description";
        for (i, (tool, rect, text)) in TOOLS.iter().enumerate() {
            draw::rect(&mut self.screen, BG_COLOUR.pixel(), *rect, 0);
            if *tool != Tool::Stamp {
                let (icon, (x, y)) = &art.tool_icons[if i > 12 { i - 1 } else { i }];
                self.screen.blit(icon, *x, *y);
            } else {
                // draw_stamp_preview
                draw::rect(&mut self.screen, BG_COLOUR.pixel(), *rect, 0);
                fit_img_to_rect(&mut self.screen, &art.stamp_tool[self.tool_settings.stamp], *rect);
            }
            if rect.collidepoint(b.mx, b.my) {
                draw::rect(&mut self.screen, HOVER_COLOUR.pixel(), *rect, 2);
                description = text;
            } else {
                draw::rect(&mut self.screen, UNSELECED_COLOUR.pixel(), *rect, 2);
            }
        }
        // write_description
        draw::rect(&mut self.screen, BG_COLOUR.pixel(), DESCRIPTION_RECT, 0);
        art.text.write_centered_text(&mut self.screen, description, BLACK);
        draw::rect(&mut self.screen, BLACK.pixel(), DESCRIPTION_RECT, 2);
        // draw_selected_tool
        draw::rect(&mut self.screen, SELECTED_COLOUR.pixel(), tool_rect(self.tool_settings.tool), 2);
    }

    fn tool_manager(&mut self, b: &ButtonData, art: &Art) -> (Color, bool) {
        let ts = self.tool_settings;
        let mut col = ts.col;
        let mut update = false;
        if b.mb[0] || b.mouse_up || ts.tool == Tool::Select {
            match ts.tool {
                Tool::Pencil => draw::line(&mut self.screen, ts.col.pixel(), (b.omx, b.omy), (b.mx, b.my), 1),
                Tool::Eraser => {
                    draw::circle(&mut self.screen, WHITE.pixel(), (b.mx, b.my), ts.radius);
                    draw_rounded_line(&mut self.screen, WHITE, (b.omx, b.omy), (b.mx, b.my), ts.radius);
                }
                Tool::Spray => self.spray_tool(b, ts),
                Tool::PaintBrush => {
                    draw::circle(&mut self.screen, ts.col.pixel(), (b.mx, b.my), ts.radius);
                    draw_rounded_line(&mut self.screen, ts.col, (b.omx, b.omy), (b.mx, b.my), ts.radius);
                }
                Tool::Line => {
                    let (mut mx, mut my) = (b.mx, b.my);
                    if b.shift {
                        // snap to 90 degree angles
                        if (mx - b.dmx).abs() > (my - b.dmy).abs() {
                            my = b.dmy;
                        } else {
                            mx = b.dmx;
                        }
                    }
                    draw_rounded_line(&mut self.screen, ts.col, (b.dmx, b.dmy), (mx, my), ts.radius);
                }
                Tool::Rect | Tool::Ellipse => {
                    let mut left = b.mx.min(b.dmx);
                    let mut top = b.my.min(b.dmy);
                    let mut width = (b.mx - b.dmx).abs();
                    let mut height = (b.my - b.dmy).abs();
                    if b.shift {
                        width = width.min(height);
                        height = width;
                        if b.mx < b.dmx {
                            left = b.dmx - width;
                        }
                        if b.my < b.dmy {
                            top = b.dmy - height;
                        }
                    }
                    let r = Rect::new(left, top, width, height);
                    let thickness = if ts.fill { 0 } else { ts.radius };
                    if ts.tool == Tool::Rect {
                        draw::rect(&mut self.screen, ts.col.pixel(), r, thickness);
                    } else {
                        draw::ellipse(&mut self.screen, ts.col.pixel(), r, thickness);
                    }
                }
                Tool::EyeDropper => col = self.screen.get_at(b.mx, b.my).unwrap_or(col),
                Tool::Fireworks => self.fireworks_tool(b, ts),
                Tool::AirBrush => self.air_brush_tool(b, ts),
                Tool::PaintBucket => self.paint_bucket_tool(b, ts),
                Tool::FillScreen => {
                    if b.mouse_down {
                        self.screen.fill_rect(CANVAS, ts.col.pixel());
                    }
                }
                Tool::Stamp => {
                    if let Some(stamp) = &art.stamps[ts.stamp] {
                        let scaled = scale_stamp(stamp, ts.stamp_size);
                        self.screen.blit(&scaled, b.mx - scaled.w / 2, b.my - scaled.h / 2);
                    }
                }
                Tool::Select => update = self.selection_tool(b),
            }
        }
        (col, update)
    }

    fn spray_tool(&mut self, b: &ButtonData, ts: ToolSettings) {
        let radius = ts.radius;
        // number of loops per frame to fill the circle in 240 frames
        let loops = 4 * radius * radius / 240;
        for _ in 0..loops {
            let x = self.rng.randint(-radius, radius);
            let y = self.rng.randint(-radius, radius);
            if x * x + y * y <= radius * radius {
                let circle_radius = self.rng.randint(1, 3);
                draw::circle(&mut self.screen, ts.col.pixel(), (b.mx + x, b.my + y), circle_radius);
            }
        }
    }

    fn varied(&mut self, col: Color) -> Color {
        // change the colour slightly (+/- 128), kept between 0 and 255
        let r = (col.0 as i32 + self.rng.randint(-128, 128)).clamp(0, 255);
        let g = (col.1 as i32 + self.rng.randint(-128, 128)).clamp(0, 255);
        let b = (col.2 as i32 + self.rng.randint(-128, 128)).clamp(0, 255);
        Color(r as u8, g as u8, b as u8)
    }

    fn fireworks_tool(&mut self, b: &ButtonData, ts: ToolSettings) {
        let radius = ts.radius;
        let loops = (radius + 24) / 25; // ceil(radius / 25)
        for _ in 0..loops {
            let colour = self.varied(ts.col);
            let x1 = self.rng.randint(-radius, radius);
            let y1 = self.rng.randint(-radius, radius);
            let x2 = self.rng.randint(-radius, radius);
            let y2 = self.rng.randint(-radius, radius);
            if x1 * x1 + y1 * y1 <= radius * radius && x2 * x2 + y2 * y2 <= radius * radius {
                draw::line(&mut self.screen, colour.pixel(), (b.mx + x1, b.my + y1), (b.mx + x2, b.my + y2), 1);
            }
        }
    }

    fn air_brush_tool(&mut self, b: &ButtonData, ts: ToolSettings) {
        let radius = ts.radius;
        let loops = 4 * radius * radius / 240;
        for _ in 0..loops {
            let x = self.rng.randint(-radius, radius);
            let y = self.rng.randint(-radius, radius);
            if x * x + y * y <= radius * radius {
                let colour = self.varied(ts.col);
                let circle_radius = self.rng.randint(1, 3);
                draw::circle(&mut self.screen, colour.pixel(), (b.mx + x, b.my + y), circle_radius);
            }
        }
    }

    // A breadth-first fill of the area of the clicked colour; outside the canvas the pixels are only
    // read (the clip area keeps set_at off them), as in the Python version
    fn paint_bucket_tool(&mut self, b: &ButtonData, ts: ToolSettings) {
        let Some(fill_col) = self.screen.get_at(b.mx, b.my) else { return };
        if !(CANVAS.collidepoint(b.mx, b.my) && b.mouse_down) {
            return;
        }
        let (w, h) = (self.screen.w, self.screen.h);
        let mut visited = vec![false; (w * h) as usize];
        let mut pixels = VecDeque::from([(b.mx, b.my)]);
        while let Some((x, y)) = pixels.pop_front() {
            if self.screen.get_at(x, y) != Some(fill_col) {
                continue;
            }
            self.screen.set_at(x, y, ts.col.pixel());
            for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue; // the Python version would stop with an IndexError here
                }
                let seen = &mut visited[(ny * w + nx) as usize];
                if !*seen {
                    *seen = true;
                    pixels.push_back((nx, ny));
                }
            }
        }
    }

    // --- src/buttons.py ---

    fn button_manager(&mut self, b: &ButtonData, art: &Art) -> (bool, bool) {
        let mut fill = self.tool_settings.fill;
        self.draw_buttons(b, art, fill);
        let mut updated = false;
        if b.mouse_up {
            match BUTTONS.iter().position(|r| r.collidepoint(b.mx, b.my)) {
                Some(FILL) => fill = !fill,
                Some(SAVE) => self.saved = Some(self.screen.copy_rect(CANVAS)),
                Some(LOAD) => self.load_requested = true, // the picked image arrives in a later frame
                Some(TRASH) => {
                    self.trash();
                    updated = true;
                }
                _ => {}
            }
        }
        if let Some(img) = self.picked_image.take() {
            self.draw_loaded_image(&img);
            updated = true;
        }
        if b.key_x {
            self.trash();
            updated = true;
        }
        (fill, updated)
    }

    fn draw_buttons(&mut self, b: &ButtonData, art: &Art, fill: bool) {
        for (i, rect) in BUTTONS.iter().enumerate() {
            draw::rect(&mut self.screen, BG_COLOUR.pixel(), *rect, 0);
            if i == FILL {
                continue;
            }
            let (icon, (x, y)) = &art.button_icons[i - 1];
            self.screen.blit(icon, *x, *y);
            let outline = if rect.collidepoint(b.mx, b.my) { HOVER_COLOUR } else { UNSELECED_COLOUR };
            draw::rect(&mut self.screen, outline.pixel(), *rect, 2);
        }
        // draw_fill_button
        let rect = BUTTONS[FILL];
        draw::rect(&mut self.screen, if fill { SELECTED_COLOUR } else { BG_COLOUR }.pixel(), rect, 0);
        let outline = if rect.collidepoint(b.mx, b.my) { HOVER_COLOUR } else { UNSELECED_COLOUR };
        draw::rect(&mut self.screen, outline.pixel(), rect, 2);
        art.text.write_centered_text(&mut self.screen, "Fill", BLACK);
    }

    fn draw_loaded_image(&mut self, img: &Surface) {
        draw::rect(&mut self.screen, WHITE.pixel(), CANVAS, 0);
        if img.w < CANVAS.w && img.h < CANVAS.h {
            self.screen.blit(img, CANVAS.x, CANVAS.y);
        } else {
            // scale the image to fit the canvas while maintaining aspect ratio
            let scale = (CANVAS.w as f64 / img.w as f64).min(CANVAS.h as f64 / img.h as f64);
            let scaled = img.scale((img.w as f64 * scale) as i32, (img.h as f64 * scale) as i32);
            self.screen.blit(&scaled, CANVAS.x, CANVAS.y);
        }
    }

    fn trash(&mut self) {
        draw::rect(&mut self.screen, WHITE.pixel(), CANVAS, 0);
    }

    // --- src/undo_redo.py ---

    fn undo_manager(&mut self, b: &ButtonData, art: &Art) {
        let s = &mut self.screen;
        draw::rect(s, BG_COLOUR.pixel(), UNDO_RECT, 0);
        draw::rect(s, BG_COLOUR.pixel(), REDO_RECT, 0);
        let (undo_img, (ux, uy)) = &art.button_icons[3];
        let (redo_img, (rx, ry)) = &art.button_icons[4];
        s.blit(undo_img, *ux, *uy);
        s.blit(redo_img, *rx, *ry);
        draw::rect(s, UNSELECED_COLOUR.pixel(), UNDO_RECT, 2);
        draw::rect(s, UNSELECED_COLOUR.pixel(), REDO_RECT, 2);
        if UNDO_RECT.collidepoint(b.mx, b.my) {
            draw::rect(s, HOVER_COLOUR.pixel(), UNDO_RECT, 2);
        }
        if REDO_RECT.collidepoint(b.mx, b.my) {
            draw::rect(s, HOVER_COLOUR.pixel(), REDO_RECT, 2);
        }

        let undo = b.ctrl && b.key_z || UNDO_RECT.collidepoint(b.mx, b.my) && b.mouse_up;
        if undo && self.undo_list.len() > 1 {
            let top = self.undo_list.pop().unwrap();
            self.redo_list.push(top);
            self.screen.blit(self.undo_list.last(), CANVAS.x, CANVAS.y);
        }
        let redo = b.ctrl && b.shift && b.key_z || b.ctrl && b.key_y || REDO_RECT.collidepoint(b.mx, b.my) && b.mouse_up;
        if redo && self.redo_list.len() > 0 {
            let top = self.redo_list.pop().unwrap();
            self.undo_list.push(top);
            self.screen.blit(self.undo_list.last(), CANVAS.x, CANVAS.y);
        }
    }

    // --- src/sliders.py ---

    fn draw_colour_palette(&mut self, b: &ButtonData, art: &Art) -> Color {
        let s = &mut self.screen;
        // the gradients, black to each colour
        for (gradient, slider) in art.gradients.iter().zip([RED_SLIDER, GREEN_SLIDER, BLUE_SLIDER]) {
            s.blit(gradient, slider.x, slider.y);
        }
        for slider in [RED_SLIDER, GREEN_SLIDER, BLUE_SLIDER] {
            draw::rect(s, UNSELECED_COLOUR.pixel(), slider, 2);
        }
        let Color(mut r, mut g, mut bl) = self.tool_settings.col;
        if b.mb[0] {
            let value = |slider: Rect| ((slider.x + slider.w - b.mx) * 255).div_euclid(slider.w) as u8;
            if RED_SLIDER.collidepoint(b.mx, b.my) {
                r = value(RED_SLIDER);
            }
            if GREEN_SLIDER.collidepoint(b.mx, b.my) {
                g = value(GREEN_SLIDER);
            }
            if BLUE_SLIDER.collidepoint(b.mx, b.my) {
                bl = value(BLUE_SLIDER);
            }
        }
        let col = Color(r, g, bl);
        draw::rect(s, col.pixel(), COLOUR_RECT, 0);
        draw::rect(s, UNSELECED_COLOUR.pixel(), COLOUR_RECT, 2);
        let txt_col = if (r as i32 + g as i32 + bl as i32) < 384 { WHITE } else { BLACK };
        art.text.write_centered_text(s, &format!("#{r:02X}{g:02X}{bl:02X}"), txt_col);
        for (slider, v) in [(RED_SLIDER, r), (GREEN_SLIDER, g), (BLUE_SLIDER, bl)] {
            let pos = (slider.x + slider.w - (v as i32 * slider.w).div_euclid(255)).clamp(slider.x, slider.x + slider.w);
            draw::rect(s, OUTLINE_COLOUR.pixel(), Rect::new(pos, slider.y, 2, slider.h), 0);
        }
        col
    }

    fn draw_radius_selector(&mut self, b: &ButtonData, art: &Art) -> i32 {
        let s = &mut self.screen;
        let mut radius = self.tool_settings.radius;
        let fill_col = (radius * 255).div_euclid(100) as u8;
        draw::rect(s, Color(fill_col, fill_col, fill_col).pixel(), RADIUS_SLIDER, 0);
        draw::rect(s, UNSELECED_COLOUR.pixel(), RADIUS_SLIDER, 2);
        if b.mb[0] && RADIUS_SLIDER.collidepoint(b.mx, b.my) {
            radius = 1.max(((b.mx - RADIUS_SLIDER.x) * 100).div_euclid(RADIUS_SLIDER.w));
        }
        let pos = (RADIUS_SLIDER.x + (radius * RADIUS_SLIDER.w).div_euclid(100)).clamp(RADIUS_SLIDER.x, RADIUS_SLIDER.x + RADIUS_SLIDER.w);
        let slider_col = if fill_col < 128 { Color(196, 196, 196) } else { Color(64, 64, 64) };
        draw::rect(s, slider_col.pixel(), Rect::new(pos, RADIUS_SLIDER.y, 2, RADIUS_SLIDER.h), 0);
        let txt_col = if fill_col < 128 { WHITE } else { BLACK };
        art.text.write_centered_text(s, "Size", txt_col);
        radius
    }

    fn draw_stamp_sizer(&mut self, b: &ButtonData, art: &Art) -> (i32, bool) {
        let mut dont_update = false;
        let mut stamp_size = self.tool_settings.stamp_size;
        let fill_col = (stamp_size * 255).div_euclid(100) as u8;
        draw::rect(&mut self.screen, Color(fill_col, fill_col, fill_col).pixel(), STAMP_SLIDER, 0);
        draw::rect(&mut self.screen, UNSELECED_COLOUR.pixel(), STAMP_SLIDER, 2);
        if (b.mb[0] || b.mouse_up) && STAMP_SLIDER.collidepoint(b.dmx, b.dmy) {
            // 100 at the top, 0 at the bottom
            stamp_size = ((STAMP_SLIDER.y + STAMP_SLIDER.h - b.my) * 100).div_euclid(STAMP_SLIDER.h).clamp(1, 100);
            if self.tool_settings.tool == Tool::Stamp {
                dont_update = true;
                // a preview of the stamp at this size in the middle of the canvas
                if let Some(stamp) = &art.stamps[self.stamp_list[0]] {
                    let scaled = scale_stamp(stamp, stamp_size);
                    self.screen.blit(&scaled, CANVAS.centerx() - scaled.w / 2, CANVAS.centery() - scaled.h / 2);
                }
            }
        }
        let pos = (STAMP_SLIDER.y + STAMP_SLIDER.h - (stamp_size * STAMP_SLIDER.h).div_euclid(100)).clamp(STAMP_SLIDER.y, STAMP_SLIDER.y + STAMP_SLIDER.h);
        let slider_col = if fill_col < 128 { Color(196, 196, 196) } else { Color(64, 64, 64) };
        draw::rect(&mut self.screen, slider_col.pixel(), Rect::new(STAMP_SLIDER.x, pos, STAMP_SLIDER.w, 2), 0);
        let txt_col = if fill_col < 128 { WHITE } else { BLACK };
        art.text.write_centered_text(&mut self.screen, "Stamp\nSize", txt_col);
        (stamp_size, dont_update)
    }

    // --- src/selection.py ---

    fn selection_tool(&mut self, b: &ButtonData) -> bool {
        let (mx, my, dmx, dmy) = (b.mx, b.my, b.dmx, b.dmy);
        if b.ctrl && b.key_z {
            self.select_state = 0;
            return false;
        }
        self.select_state = self.selection_state_manager(b);
        let white = WHITE.pixel();
        match self.select_state {
            1 => {
                // draw the selection rectangle preview
                let r = Rect::new(mx.min(dmx), my.min(dmy), (mx - dmx).abs(), (my - dmy).abs());
                draw_dotted_rect(&mut self.screen, UNSELECTED, r, 3);
            }
            2 => {
                // if the selected area is too small, don't create a selection
                if (mx - dmx).abs() < RHS || (my - dmy).abs() < RHS {
                    self.select_state = 0;
                    return false;
                }
                let mut top = my.min(dmy);
                let mut left = mx.min(dmx);
                let mut width = (mx - dmx).abs();
                let mut height = (my - dmy).abs();
                // make sure the selection rectangle is within the canvas
                if top < CANVAS.y {
                    height = top - CANVAS.y + height;
                    top = CANVAS.y;
                }
                if left < CANVAS.x {
                    width = left - CANVAS.x + width;
                    left = CANVAS.x;
                }
                if top + height > CANVAS.y + CANVAS.h {
                    height = CANVAS.y + CANVAS.h - top;
                }
                if left + width > CANVAS.x + CANVAS.w {
                    width = CANVAS.x + CANVAS.w - left;
                }
                // (a selection wholly outside the canvas ends up empty; the Python version stops there)
                let clip = self.screen.copy_rect(Rect::new(left, top, width.max(0), height.max(0)));
                self.og_rect = Rect::new(0, 0, clip.w, clip.h);
                self.clipboard = Some(clip);
                draw_dotted_rect(&mut self.screen, UNSELECTED, Rect::new(left, top, width, height), 3);
                self.move_start = (left, top);
                self.og_start = (left, top);
            }
            3 => {
                let (left, top) = self.move_start;
                let clipboard = self.clipboard.take().unwrap();
                draw::rect(&mut self.screen, white, Rect::new(self.og_start.0, self.og_start.1, self.og_rect.w, self.og_rect.h), 0);
                self.screen.blit(&clipboard, left, top);
                let r = Rect::new(left, top, clipboard.w, clipboard.h);
                draw_dotted_rect(&mut self.screen, SELECTED, r, 3);
                draw_resize_boxes(&mut self.screen, r, SELECTED);
                self.clipboard = Some(clipboard);
            }
            4 => {
                let dx = dmx - self.move_start.0;
                let dy = dmy - self.move_start.1;
                let (top, left) = (my - dy, mx - dx);
                let clipboard = self.clipboard.take().unwrap();
                draw::rect(&mut self.screen, white, Rect::new(self.og_start.0, self.og_start.1, self.og_rect.w, self.og_rect.h), 0);
                self.screen.blit(&clipboard, left, top);
                draw_dotted_rect(&mut self.screen, SELECTED, Rect::new(left, top, clipboard.w, clipboard.h), 3);
                self.clipboard = Some(clipboard);
            }
            5 => {
                // paste the clipboard image at the new location
                let dx = dmx - self.move_start.0;
                let dy = dmy - self.move_start.1;
                let (top, left) = (my - dy, mx - dx);
                draw::rect(&mut self.screen, white, Rect::new(self.og_start.0, self.og_start.1, self.og_rect.w, self.og_rect.h), 0);
                if let Some(clipboard) = &self.clipboard {
                    self.screen.blit(clipboard, left, top);
                }
                return true;
            }
            6 | 7 => {
                let clipboard = self.clipboard.take().unwrap();
                let (left, top, width, height) = self.get_resize_rect(mx, my, dmx, dmy, &clipboard);
                draw::rect(&mut self.screen, white, Rect::new(self.og_start.0, self.og_start.1, self.og_rect.w, self.og_rect.h), 0);
                let scaled = clipboard.scale(width, height);
                self.screen.blit(&scaled, left, top);
                let r = Rect::new(left, top, width, height);
                draw_dotted_rect(&mut self.screen, SELECTED, r, 3);
                draw_resize_boxes(&mut self.screen, r, SELECTED);
                if self.select_state == 7 {
                    self.clipboard = Some(scaled);
                    self.move_start = (left, top);
                } else {
                    self.clipboard = Some(clipboard);
                }
            }
            _ => {}
        }
        false
    }

    fn selection_state_manager(&mut self, b: &ButtonData) -> i32 {
        let (mx, my, dmx, dmy) = (b.mx, b.my, b.dmx, b.dmy);
        match self.select_state {
            0 => {
                if b.mb[0] {
                    self.select_state = 1;
                }
            }
            1 => {
                if b.mouse_up || !b.mb[0] {
                    self.select_state = 2;
                }
            }
            2 => self.select_state = 3,
            3 => {
                if b.mb[0] {
                    let dx = dmx - self.move_start.0;
                    let dy = dmy - self.move_start.1;
                    let (top, left) = (my - dy, mx - dx);
                    let (cw, ch) = self.clipboard.as_ref().map_or((0, 0), |c| (c.w, c.h));
                    let top_left = Rect::new(left - RHS2, top - RHS2, RHS, RHS);
                    let top_right = Rect::new(left + cw - RHS2, top - RHS2, RHS, RHS);
                    let bottom_left = Rect::new(left - RHS2, top + ch - RHS2, RHS, RHS);
                    let bottom_right = Rect::new(left + cw - RHS2, top + ch - RHS2, RHS, RHS);
                    if top_left.collidepoint(mx, my) {
                        self.resize_corner = 0;
                        self.resize_start = (left + cw, top + ch);
                        self.select_state = 6;
                    } else if top_right.collidepoint(mx, my) {
                        self.resize_corner = 1;
                        self.resize_start = (left, top + ch);
                        self.select_state = 6;
                    } else if bottom_left.collidepoint(mx, my) {
                        self.resize_corner = 2;
                        self.resize_start = (left + cw, top);
                        self.select_state = 6;
                    } else if bottom_right.collidepoint(mx, my) {
                        self.resize_corner = 3;
                        self.resize_start = (left, top);
                        self.select_state = 6;
                    } else if Rect::new(left, top, cw, ch).collidepoint(mx, my) {
                        self.select_state = 4;
                    } else {
                        self.select_state = 5;
                    }
                } else if b.mouse_up {
                    self.select_state = 3;
                }
            }
            4 => {
                let dx = dmx - self.move_start.0;
                let dy = dmy - self.move_start.1;
                let (top, left) = (my - dy, mx - dx);
                let (cw, ch) = self.clipboard.as_ref().map_or((0, 0), |c| (c.w, c.h));
                if !Rect::new(left, top, cw, ch).collidepoint(mx, my) {
                    self.select_state = 5;
                }
                if b.mouse_up {
                    self.move_start = (left, top);
                    self.select_state = 3;
                }
            }
            5 => {
                self.select_state = 0;
                self.clipboard = None;
            }
            6 => {
                if b.mouse_up {
                    self.select_state = 7;
                }
            }
            7 => self.select_state = 3,
            _ => {}
        }
        self.select_state
    }

    fn get_resize_rect(&self, mx: i32, my: i32, dmx: i32, dmy: i32, clipboard: &Surface) -> (i32, i32, i32, i32) {
        let (cw, ch) = (clipboard.w, clipboard.h);
        let rs = self.resize_start;
        let (mut left, mut top, mut width, mut height) = match self.resize_corner {
            0 => {
                let (w, h) = (cw + dmx - mx, ch + dmy - my);
                (rs.0 - w, rs.1 - h, w, h)
            }
            1 => {
                let (w, h) = (cw - dmx + mx, ch + dmy - my);
                (rs.0, rs.1 - h, w, h)
            }
            2 => {
                let (w, h) = (cw + dmx - mx, ch - dmy + my);
                (rs.0 - w, rs.1, w, h)
            }
            _ => (rs.0, rs.1, cw - dmx + mx, ch - dmy + my),
        };
        // make sure the width and height are at least RHS pixels
        width = width.max(RHS);
        height = height.max(RHS);
        // match_resize_corner
        match self.resize_corner {
            0 => {
                top = top.min(rs.1 - RHS);
                left = left.min(rs.0 - RHS);
            }
            1 => {
                top = top.min(rs.1 - RHS);
                left = left.max(rs.0);
            }
            2 => {
                top = top.max(rs.1);
                left = left.min(rs.0 - RHS);
            }
            _ => {
                top = top.max(rs.1);
                left = left.max(rs.0);
            }
        }
        (left, top, width, height)
    }
}

// draw_title in src/background.py
fn draw_title(screen: &mut Surface, art: &Art) {
    let (x, y) = crate::baked::TITLE_AT;
    screen.blit(&art.title, x, y);
}

// fit_img_to_rect in src/utils.py, with the image already scaled by bake.py
fn fit_img_to_rect(screen: &mut Surface, scaled: &Surface, rect: Rect) {
    screen.blit(scaled, rect.centerx() - scaled.w / 2, rect.centery() - scaled.h / 2);
}

// The stamp scaled to fit the canvas, times the stamp size percentage (stamp_tool, draw_stamp_sizer)
fn scale_stamp(stamp: &Surface, stamp_size: i32) -> Surface {
    let mut scale = (CANVAS.w as f64 / stamp.w as f64).min(CANVAS.h as f64 / stamp.h as f64);
    scale *= stamp_size as f64 / 100.0;
    stamp.scale((stamp.w as f64 * scale) as i32, (stamp.h as f64 * scale) as i32)
}

// draw_rounded_line in src/utils.py: a circle at each whole step along the line
fn draw_rounded_line(screen: &mut Surface, col: Color, start: (i32, i32), end: (i32, i32), radius: i32) {
    let (dx, dy) = (end.0 - start.0, end.1 - start.1);
    // hypot of two integers: the correctly rounded square root, as Python's math.hypot gives
    let dist = ((dx as f64) * (dx as f64) + (dy as f64) * (dy as f64)).sqrt();
    if dist == 0.0 {
        return;
    }
    let (dx, dy) = (dx as f64 / dist, dy as f64 / dist);
    for i in 0..dist as i32 {
        let x = (start.0 as f64 + dx * i as f64) as i32;
        let y = (start.1 as f64 + dy * i as f64) as i32;
        draw::circle(screen, col.pixel(), (x, y), radius);
    }
}

// draw_dotted_rect in src/utils.py
fn draw_dotted_rect(screen: &mut Surface, color: Color, r: Rect, dash_length: i32) {
    let (x, y, w, h) = (r.x, r.y, r.w, r.h);
    let mut i = 0;
    while i < w {
        draw::line(screen, color.pixel(), (x + i, y), (x + i + dash_length, y), 3);
        draw::line(screen, color.pixel(), (x + i, y + h), (x + i + dash_length, y + h), 3);
        i += dash_length * 2;
    }
    let mut i = 0;
    while i < h {
        draw::line(screen, color.pixel(), (x, y + i), (x, y + i + dash_length), 3);
        draw::line(screen, color.pixel(), (x + w, y + i), (x + w, y + i + dash_length), 3);
        i += dash_length * 2;
    }
}

// draw_resize_boxes in src/selection.py
fn draw_resize_boxes(screen: &mut Surface, r: Rect, colour: Color) {
    for (cx, cy) in [(r.x, r.y), (r.x + r.w, r.y), (r.x, r.y + r.h), (r.x + r.w, r.y + r.h)] {
        draw::rect(screen, colour.pixel(), Rect::new(cx - RHS2, cy - RHS2, RHS, RHS), 0);
    }
}
