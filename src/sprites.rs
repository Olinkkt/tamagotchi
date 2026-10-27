//! Pixel-art renderer. Everything is drawn onto a small RGB canvas and
//! converted to terminal cells with half-block characters (two pixels per cell).

use crate::pet::{Pet, Stage};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

const W: usize = 32;
const H: usize = 20;
const CX: f32 = 15.5;
const MID: i32 = 15;

type C = (u8, u8, u8);

const INK: C = (35, 25, 45);
const WHITE: C = (245, 245, 255);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fx {
    Eat,
    Hearts,
    Heal,
    Clean,
    Scold,
    Evolve,
}

impl Fx {
    pub fn duration_secs(self) -> f32 {
        match self {
            Fx::Eat => 1.8,
            Fx::Hearts => 1.6,
            Fx::Heal => 1.4,
            Fx::Clean => 1.0,
            Fx::Scold => 1.2,
            Fx::Evolve => 2.6,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mood {
    Happy,
    Neutral,
    Sad,
    Sleep,
    Sick,
    Dead,
}

fn mix(a: C, b: C, t: f32) -> C {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

fn darken(a: C, t: f32) -> C {
    mix(a, (0, 0, 0), t)
}

struct Canvas {
    px: Vec<Option<C>>,
}

impl Canvas {
    fn new() -> Self {
        Canvas { px: vec![None; W * H] }
    }

    fn set(&mut self, x: i32, y: i32, c: C) {
        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
            self.px[y as usize * W + x as usize] = Some(c);
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<C> {
        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
            self.px[y as usize * W + x as usize]
        } else {
            None
        }
    }

    fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: C) {
        for y in 0..H {
            for x in 0..W {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    self.set(x as i32, y as i32, c);
                }
            }
        }
    }

    fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: C) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set(x, y, c);
            }
        }
    }

    fn pixels(&mut self, pts: &[(i32, i32)], c: C) {
        for &(x, y) in pts {
            self.set(x, y, c);
        }
    }

    /// Put a dark edge around every filled shape.
    fn outline(&mut self, c: C) {
        let mut edge = Vec::new();
        for y in 0..H as i32 {
            for x in 0..W as i32 {
                if self.get(x, y).is_none()
                    && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| self.get(x + dx, y + dy).is_some())
                {
                    edge.push((x, y));
                }
            }
        }
        self.pixels(&edge, c);
    }

    fn blit(&mut self, src: &Canvas, dx: i32, dy: i32, flat: Option<C>) {
        for y in 0..H as i32 {
            for x in 0..W as i32 {
                if let Some(c) = src.get(x, y) {
                    self.set(x + dx, y + dy, flat.unwrap_or(c));
                }
            }
        }
    }
}

struct Face {
    eye_y: i32,
    sep: i32,
    mouth_y: i32,
}

struct Body {
    canvas: Canvas,
    face: Option<Face>,
    body: C,
    outline: C,
}

fn heart(c: &mut Canvas, x: i32, y: i32, col: C) {
    c.pixels(&[(1, 0), (3, 0)].map(|(a, b)| (x + a, y + b)), col);
    c.rect(x, y + 1, x + 4, y + 1, col);
    c.rect(x + 1, y + 2, x + 3, y + 2, col);
    c.set(x + 2, y + 3, col);
}

fn draw_body(p: &Pet, progress_egg: f32) -> Body {
    let mut c = Canvas::new();
    let tint = |col: C| if p.sick { mix(col, (120, 190, 90), 0.45) } else { col };
    let mut face = None;
    let body;

    match (p.stage, p.species.as_str()) {
        (Stage::Egg, _) => {
            body = (250, 240, 210);
            c.ellipse(CX, 13.0, 5.5, 5.0, body);
            c.ellipse(CX, 9.5, 4.2, 4.5, body);
            c.pixels(&[(13, 11), (14, 12), (18, 14), (12, 15), (17, 10), (16, 16)], (120, 200, 170));
            if progress_egg > 0.5 {
                c.pixels(&[(15, 6), (16, 7), (15, 8), (16, 9), (17, 10)], INK);
            }
            if progress_egg > 0.8 {
                c.pixels(&[(14, 10), (13, 11), (14, 12), (18, 11)], INK);
            }
        }
        (Stage::Baby, _) => {
            body = tint((255, 160, 190));
            c.ellipse(CX, 14.0, 5.0, 4.0, body);
            c.ellipse(CX, 15.0, 3.0, 2.5, mix(body, WHITE, 0.45));
            c.pixels(&[(15, 9), (16, 8)], body);
            face = Some(Face { eye_y: 12, sep: 2, mouth_y: 15 });
        }
        (Stage::Child, _) => {
            body = tint((255, 222, 95));
            let acc = darken(body, 0.15);
            c.ellipse(CX - 4.5, 7.5, 1.6, 2.2, acc);
            c.ellipse(CX + 4.5, 7.5, 1.6, 2.2, acc);
            c.ellipse(CX, 12.5, 6.0, 5.5, body);
            c.ellipse(CX, 14.5, 3.5, 3.0, mix(body, WHITE, 0.5));
            c.ellipse(CX - 3.0, 17.0, 2.2, 1.0, acc);
            c.ellipse(CX + 3.0, 17.0, 2.2, 1.0, acc);
            face = Some(Face { eye_y: 10, sep: 3, mouth_y: 13 });
        }
        (Stage::Teen, "Hashitamatchi") => {
            body = tint((175, 125, 225));
            let acc = darken(body, 0.25);
            for dx in [-3, 0, 3] {
                c.pixels(&[(MID + dx, 4), (MID + dx, 3)], acc);
            }
            c.set(MID, 2, acc);
            c.ellipse(CX, 11.5, 6.5, 6.5, body);
            c.ellipse(CX, 14.5, 4.0, 3.5, mix(body, WHITE, 0.4));
            c.ellipse(CX - 3.5, 17.0, 2.4, 1.0, acc);
            c.ellipse(CX + 3.5, 17.0, 2.4, 1.0, acc);
            face = Some(Face { eye_y: 9, sep: 3, mouth_y: 13 });
        }
        (Stage::Teen, _) => {
            body = tint((115, 175, 255));
            let acc = darken(body, 0.2);
            c.ellipse(CX - 5.0, 5.5, 1.6, 2.8, acc);
            c.ellipse(CX + 5.0, 5.5, 1.6, 2.8, acc);
            c.ellipse(CX, 11.5, 6.5, 6.5, body);
            c.ellipse(CX, 14.5, 4.0, 3.5, mix(body, WHITE, 0.5));
            c.ellipse(CX - 3.5, 17.0, 2.4, 1.0, acc);
            c.ellipse(CX + 3.5, 17.0, 2.4, 1.0, acc);
            face = Some(Face { eye_y: 9, sep: 3, mouth_y: 13 });
        }
        (Stage::Adult, "Mametchi") => {
            body = tint((135, 215, 135));
            let acc = darken(body, 0.2);
            c.ellipse(CX - 3.5, 3.5, 1.6, 2.5, acc);
            c.ellipse(CX + 3.5, 3.5, 1.6, 2.5, acc);
            c.ellipse(CX, 10.5, 6.0, 7.5, body);
            c.ellipse(CX - 6.5, 12.0, 1.3, 2.2, acc);
            c.ellipse(CX + 6.5, 12.0, 1.3, 2.2, acc);
            c.ellipse(CX, 13.0, 3.5, 4.0, mix(body, WHITE, 0.5));
            c.ellipse(CX - 3.0, 17.0, 2.4, 1.0, acc);
            c.ellipse(CX + 3.0, 17.0, 2.4, 1.0, acc);
            face = Some(Face { eye_y: 8, sep: 3, mouth_y: 12 });
        }
        (Stage::Adult, "Kuchipatchi") => {
            body = tint((255, 210, 70));
            let orange = tint((255, 140, 50));
            c.pixels(&[(15, 4), (15, 3), (16, 3), (14, 3)], orange);
            c.ellipse(CX, 11.5, 7.5, 6.5, body);
            c.ellipse(CX - 7.5, 12.0, 1.5, 2.5, darken(body, 0.15));
            c.ellipse(CX + 7.5, 12.0, 1.5, 2.5, darken(body, 0.15));
            c.ellipse(CX, 11.5, 2.6, 1.6, orange);
            c.ellipse(CX - 3.5, 17.0, 2.2, 1.0, orange);
            c.ellipse(CX + 3.5, 17.0, 2.2, 1.0, orange);
            face = Some(Face { eye_y: 8, sep: 3, mouth_y: 14 });
        }
        (Stage::Adult, "Tarakotchi") => {
            body = tint((255, 155, 75));
            let acc = darken(body, 0.25);
            c.pixels(&[(15, 3), (15, 2), (16, 2), (14, 2), (13, 3), (17, 3)], acc);
            c.ellipse(CX, 11.0, 7.0, 7.0, body);
            c.ellipse(CX, 14.0, 3.0, 1.5, (255, 140, 160));
            c.pixels(&[(10, 9), (11, 10), (20, 9), (19, 10), (9, 12), (21, 12)], acc);
            c.ellipse(CX - 3.5, 17.0, 2.4, 1.0, acc);
            c.ellipse(CX + 3.5, 17.0, 2.4, 1.0, acc);
            face = Some(Face { eye_y: 8, sep: 3, mouth_y: 14 });
        }
        (Stage::Adult, "Ichigotchi") => {
            body = tint((240, 85, 95));
            let green = tint((90, 190, 90));
            c.rect(15, 2, 15, 4, green);
            c.ellipse(CX - 3.0, 5.0, 2.6, 1.3, green);
            c.ellipse(CX + 3.0, 5.0, 2.6, 1.3, green);
            c.ellipse(CX, 11.5, 6.5, 6.5, body);
            c.pixels(&[(11, 15), (19, 15), (10, 11), (20, 11), (15, 16), (12, 12), (18, 12)], (255, 225, 110));
            c.ellipse(CX - 3.0, 17.0, 2.2, 1.0, darken(body, 0.25));
            c.ellipse(CX + 3.0, 17.0, 2.2, 1.0, darken(body, 0.25));
            face = Some(Face { eye_y: 9, sep: 3, mouth_y: 13 });
        }
        (Stage::Adult, _) => {
            body = tint((85, 205, 205));
            let acc = darken(body, 0.2);
            c.pixels(&[(15, 3), (15, 2), (14, 2), (13, 1)], acc);
            c.ellipse(CX + 6.5, 15.0, 2.2, 2.2, body);
            c.ellipse(CX + 8.5, 12.5, 1.2, 1.4, body);
            c.ellipse(CX, 11.0, 5.0, 7.0, body);
            c.ellipse(CX, 13.5, 3.0, 4.0, mix(body, WHITE, 0.5));
            face = Some(Face { eye_y: 8, sep: 2, mouth_y: 12 });
        }
        (Stage::Dead, _) => {
            body = (150, 150, 165);
            c.ellipse(CX, 10.0, 5.5, 3.0, body);
            c.rect(10, 10, 20, 17, body);
            c.pixels(&[(15, 9), (15, 10), (15, 11), (15, 12), (15, 13), (13, 11), (14, 11), (16, 11), (17, 11)], INK);
        }
    }
    let outline = darken(body, 0.5);
    c.outline(outline);
    Body { canvas: c, face, body, outline }
}

fn draw_face(c: &mut Canvas, f: &Face, mood: Mood, blink: bool, mouth_open: bool) {
    let ink = INK;
    let ey = f.eye_y;
    for (side, ex) in [(-1, MID - f.sep), (1, MID + f.sep)] {
        match (mood, blink) {
            (Mood::Sick | Mood::Dead, _) => {
                c.pixels(&[(ex - 1, ey), (ex + 1, ey), (ex, ey + 1), (ex - 1, ey + 2), (ex + 1, ey + 2)], ink)
            }
            (Mood::Sleep, _) | (_, true) => c.pixels(&[(ex - 1, ey + 1), (ex, ey + 1), (ex + 1, ey + 1)], ink),
            (Mood::Happy, _) => c.pixels(&[(ex - 1, ey + 1), (ex, ey), (ex + 1, ey + 1)], ink),
            (Mood::Sad, _) => {
                c.pixels(&[(ex, ey), (ex, ey + 1)], ink);
                if side < 0 {
                    c.pixels(&[(ex - 1, ey + 2), (ex - 1, ey + 3)], (90, 170, 255));
                }
            }
            (Mood::Neutral, _) => {
                c.pixels(&[(ex, ey), (ex, ey + 1)], ink);
                c.set(ex, ey, WHITE);
                c.set(ex, ey + 1, ink);
            }
        }
    }
    let my = f.mouth_y;
    let wide = f.sep >= 3;
    if mouth_open {
        c.rect(MID - 1, my, MID + 1, my + 1, ink);
        c.set(MID, my + 1, (230, 90, 100));
        return;
    }
    match mood {
        Mood::Happy if wide => c.pixels(&[(MID - 2, my), (MID - 1, my + 1), (MID, my + 1), (MID + 1, my + 1), (MID + 2, my)], ink),
        Mood::Happy => c.pixels(&[(MID - 1, my), (MID, my + 1), (MID + 1, my)], ink),
        Mood::Neutral | Mood::Dead => c.pixels(&[(MID - 1, my), (MID, my), (MID + 1, my)], ink),
        Mood::Sad => c.pixels(&[(MID - 1, my + 1), (MID, my), (MID + 1, my + 1)], ink),
        Mood::Sleep => c.set(MID, my, ink),
        Mood::Sick => c.pixels(&[(MID - 2, my), (MID - 1, my + 1), (MID, my), (MID + 1, my + 1), (MID + 2, my)], ink),
    }
    if mood == Mood::Happy {
        c.set(MID - f.sep - 1, ey + 3, (255, 140, 165));
        c.set(MID + f.sep + 1, ey + 3, (255, 140, 165));
    }
}

fn draw_z(c: &mut Canvas, x: i32, y: i32, col: C) {
    c.rect(x, y, x + 2, y, col);
    c.set(x + 1, y + 1, col);
    c.rect(x, y + 2, x + 2, y + 2, col);
}

fn draw_poop(c: &mut Canvas, x: i32, t: u32) {
    let (brown, dark, light) = ((150, 95, 55), (100, 60, 35), (185, 125, 80));
    c.set(x + 1, 15, brown);
    c.rect(x, 16, x + 2, 16, brown);
    c.rect(x, 17, x + 3, 17, dark);
    c.rect(x + 1, 17, x + 2, 17, brown);
    c.set(x + 1, 15, light);
    if t % 2 == 0 {
        c.pixels(&[(x, 13), (x + 2, 12)], (130, 190, 90));
    } else {
        c.pixels(&[(x + 1, 13), (x + 3, 12)], (130, 190, 90));
    }
}

fn shadow_radius(stage: Stage) -> f32 {
    match stage {
        Stage::Egg | Stage::Baby => 5.0,
        Stage::Child => 6.5,
        Stage::Teen => 7.0,
        _ => 8.0,
    }
}

/// Render the pet scene. `t` is the animation clock, `fx` an optional
/// one-shot effect with its progress in 0..1.
pub fn render(p: &Pet, t: u32, fx: Option<(Fx, f32)>) -> Vec<Line<'static>> {
    let egg_progress = p.stage_age as f32 / 30.0;
    let fx_kind = fx.map(|f| f.0);
    let fxp = fx.map(|f| f.1).unwrap_or(0.0);

    let mood = if !p.alive() {
        Mood::Dead
    } else if p.asleep {
        Mood::Sleep
    } else if p.sick {
        Mood::Sick
    } else if fx_kind == Some(Fx::Scold) || p.hunger == 0 || p.happy == 0 {
        Mood::Sad
    } else if fx_kind == Some(Fx::Hearts) || p.happy >= 3 {
        Mood::Happy
    } else {
        Mood::Neutral
    };

    let mouth_open = fx_kind == Some(Fx::Eat) && (0.3..0.9).contains(&fxp) && ((fxp * 8.0) as i32) % 2 == 0;
    let blink = !p.asleep && t % 9 == 8;

    let mut body = draw_body(p, egg_progress);
    if let Some(face) = &body.face {
        draw_face(&mut body.canvas, face, mood, blink, mouth_open);
    }

    // Movement: idle wander and bob; shake when scolded, wobble when an egg is hatching.
    let wander = [0, 1, 2, 2, 1, 0, -1, -2, -2, -1];
    let (mut dx, dy) = match p.stage {
        Stage::Dead => (0, 0),
        Stage::Egg => (if egg_progress > 0.35 { [0, 1, 0, -1][(t % 4) as usize] } else { 0 }, 0),
        _ if p.asleep => (0, ((t / 3) % 2) as i32),
        _ if p.sick || p.hunger == 0 => (0, (t % 2) as i32),
        _ => (wander[((t / 3) % 10) as usize], (t % 2) as i32),
    };
    if fx_kind == Some(Fx::Scold) {
        dx = if (fxp * 20.0) as i32 % 2 == 0 { 1 } else { -1 };
    }

    let mut scene = Canvas::new();
    scene.ellipse(CX + dx as f32, 19.0, shadow_radius(p.stage), 1.0, (55, 55, 72));
    for i in 0..p.poops.min(3) as usize {
        draw_poop(&mut scene, [1, 5, 26][i], t);
    }
    let flat = (fx_kind == Some(Fx::Evolve) && fxp < 0.6 && (fxp * 14.0) as i32 % 2 == 0).then_some(WHITE);
    scene.blit(&body.canvas, dx, dy, flat);
    let _ = (body.body, body.outline);

    if p.stage == Stage::Dead {
        let gy = 1 + ((t / 2) % 3) as i32;
        scene.ellipse(CX, gy as f32 + 3.0, 3.0, 3.0, (235, 235, 248));
        scene.rect(MID - 3, gy + 3, MID + 3, gy + 6, (235, 235, 248));
        scene.pixels(&[(MID - 3, gy + 7), (MID - 1, gy + 7), (MID + 1, gy + 7), (MID + 3, gy + 7)], (235, 235, 248));
        scene.pixels(&[(MID - 2, gy + 2), (MID - 1, gy + 3), (MID - 2, gy + 4), (MID + 2, gy + 2), (MID + 1, gy + 3), (MID + 2, gy + 4)], INK);
    }
    if p.asleep && p.alive() && p.stage != Stage::Egg {
        for i in 0..2i32 {
            let phase = ((t / 2) as i32 + i * 2) % 4;
            draw_z(&mut scene, 22 + i * 4, 9 - phase * 2, (170, 195, 255));
        }
    }
    if p.sick && !p.asleep && p.alive() && t % 2 == 0 {
        scene.pixels(&[(23, 6), (23, 7)], (110, 190, 255));
        scene.set(23, 8, (160, 215, 255));
    }
    if p.needs_attention() && t % 2 == 0 {
        scene.rect(27, 1, 28, 5, (235, 60, 60));
        scene.rect(27, 7, 28, 8, (235, 60, 60));
    }

    match fx_kind {
        Some(Fx::Hearts) => {
            for i in 0..3 {
                let y = 8 - (fxp * 11.0) as i32 + (i % 2) * 3;
                if fxp < 0.9 {
                    heart(&mut scene, 7 + i * 7, y, if i == 1 { (255, 90, 120) } else { (255, 130, 160) });
                }
            }
        }
        Some(Fx::Eat) => {
            if fxp < 0.5 {
                let x = 28 - (fxp / 0.5 * 9.0) as i32;
                scene.rect(x, 11, x + 2, 13, (230, 60, 60));
                scene.set(x + 1, 10, (90, 190, 90));
            }
        }
        Some(Fx::Heal) => {
            for (i, (x, y)) in [(7, 6), (24, 8), (9, 14), (23, 13)].into_iter().enumerate() {
                if ((fxp * 12.0) as usize + i) % 2 == 0 {
                    scene.pixels(&[(x, y), (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], (170, 255, 190));
                }
            }
        }
        Some(Fx::Clean) => {
            let x = (fxp * (W as f32 + 3.0)) as i32 - 2;
            scene.rect(x, 12, x + 1, 18, (200, 232, 255));
            scene.pixels(&[(x - 2, 14), (x - 3, 16), (x - 2, 17)], (235, 250, 255));
        }
        Some(Fx::Scold) => {
            if (fxp * 10.0) as i32 % 2 == 0 {
                let (x, y) = (7, 3);
                scene.pixels(&[(x, y), (x + 1, y), (x, y + 1), (x + 3, y), (x + 4, y), (x + 4, y + 1), (x, y + 4), (x + 1, y + 4), (x, y + 3), (x + 3, y + 4), (x + 4, y + 4), (x + 4, y + 3)], (235, 50, 50));
            }
        }
        Some(Fx::Evolve) => {
            for (i, (x, y)) in [(6, 5), (25, 4), (4, 12), (27, 11), (15, 1), (9, 16), (22, 16)].into_iter().enumerate() {
                if ((fxp * 16.0) as usize + i) % 3 != 0 {
                    scene.pixels(&[(x, y), (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], (255, 245, 170));
                }
            }
        }
        None => {}
    }

    to_lines(&scene)
}

fn to_lines(c: &Canvas) -> Vec<Line<'static>> {
    let rgb = |p: C| Color::Rgb(p.0, p.1, p.2);
    (0..H / 2)
        .map(|row| {
            let spans: Vec<Span<'static>> = (0..W)
                .map(|x| {
                    let top = c.get(x as i32, row as i32 * 2);
                    let bot = c.get(x as i32, row as i32 * 2 + 1);
                    match (top, bot) {
                        (None, None) => Span::raw(" "),
                        (Some(t), None) => Span::styled("▀", Style::new().fg(rgb(t))),
                        (None, Some(b)) => Span::styled("▄", Style::new().fg(rgb(b))),
                        (Some(t), Some(b)) => Span::styled("▀", Style::new().fg(rgb(t)).bg(rgb(b))),
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}
