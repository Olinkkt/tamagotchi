//! The room and everything in it: the pet, its moods and reactions, poop, sleep, speech bubbles
//! and the one-shot effects after an action. Drawn in room pixels (160x116).

use crate::app::{App, Mode};
use crate::art::{self, Light, Sprite};
use crate::cast::{self, Character};
use crate::faces::{self, Expr};
use crate::frame::Frame;
use crate::icons;
use crate::palette::Pal;
use crate::pet::{Pet, Stage};

/// A one-shot animation after an action: what happened, picked by `App::react`.
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

/// The standard room size, which the art was drawn for. Bigger windows get a bigger room that
/// carries the wall and floor on around it, with everything kept to the bottom centre.
pub const W: i32 = 160;
pub const H: i32 = 116;
/// Where up to three poops sit, left to right, relative to the middle of the room.
const POOP_X: [i32; 3] = [-64, -46, 48];
/// Stars in the night window that twinkle (the room sprite has them all lit).
const TWINKLE: [(i32, i32); 3] = [(21, 19), (36, 18), (50, 41)];

/// The size of the room being drawn, and what hangs off it.
#[derive(Clone, Copy)]
struct Size {
    w: i32,
    h: i32,
}

impl Size {
    fn centre(self) -> i32 {
        self.w / 2
    }

    /// The pet's feet rest on this row, the middle of the rug.
    fn floor(self) -> i32 {
        self.h - (H - 102)
    }

    /// How far things drawn for the standard room move to follow the bottom centre.
    fn shift(self) -> (i32, i32) {
        (self.w / 2 - W / 2, self.h - H)
    }
}

/// Everything that decides how the room looks at one moment.
pub struct Moment<'a> {
    pub pet: &'a Pet,
    pub fx: Option<(Fx, f32)>,
    /// Milliseconds on the animation clock.
    pub ms: u64,
}

impl<'a> Moment<'a> {
    pub fn of(app: &'a App) -> Moment<'a> {
        Moment { pet: &app.pet, fx: app.current_fx(), ms: app.started.elapsed().as_millis() as u64 }
    }

    /// The 350 ms beat most of the little animations step on.
    fn beat(&self) -> u64 {
        self.ms / 350
    }

    fn fx_is(&self, fx: Fx) -> Option<f32> {
        self.fx.filter(|(f, _)| *f == fx).map(|(_, p)| p)
    }
}

/// The sprite set for the pet's current form.
pub fn character(p: &Pet) -> Character {
    match p.stage {
        Stage::Egg => cast::EGG,
        Stage::Baby => cast::BABY,
        Stage::Child => cast::CHILD,
        _ => cast::for_species(&p.species).unwrap_or(match p.stage {
            Stage::Teen => cast::TONMARUTCHI,
            _ => cast::KUCHIPATCHI,
        }),
    }
}

/// Which frame each animated part shows: a gentle back-and-forth, wings a little quicker.
fn part_frame(part: &cast::Part, beat: u64) -> usize {
    let step = if part.name == "wings" { beat } else { beat / 2 };
    [1, 0, 1, 2][(step % 4) as usize].min(part.frames - 1)
}

/// The pet's sprite: parts behind, the body with `face` laid on, parts in front. The parts
/// (ears, wings, tuft, leaves, tail) only appear when `animate_parts` is on.
fn assemble(p: &Pet, c: &Character, face: &[(i32, i32, Pal)], beat: u64) -> Sprite {
    let cells = art::cells(c);
    let mut s = Sprite::empty(c.size as i32);
    let parts = |front: bool, s: &mut Sprite| {
        if !p.cfg.animate_parts {
            return;
        }
        for (i, part) in c.parts.iter().enumerate().filter(|(_, part)| part.front == front) {
            s.over(&cells[c.part_cell(i, part_frame(part, beat))]);
        }
    };
    parts(false, &mut s);
    s.over(&cells[0]);
    for &(x, y, col) in face {
        s.put(x, y, col);
    }
    parts(true, &mut s);
    s
}

pub fn light(p: &Pet) -> Light {
    match (p.asleep && p.alive(), p.light_on) {
        (false, _) => Light::Day,
        (true, true) => Light::Lamp,
        (true, false) => Light::Dark,
    }
}

/// The face to show, following the same rules as the old terminal pet.
pub fn expression(m: &Moment) -> Expr {
    let p = m.pet;
    let eating = m.fx_is(Fx::Eat).is_some_and(|f| (0.3..0.9).contains(&f) && ((f * 8.0) as i32) % 2 == 0);
    let base = if p.asleep || (p.load.low_battery && calm(p)) {
        Expr::Sleeping
    } else if p.sick {
        Expr::Sick
    } else if m.fx_is(Fx::Scold).is_some() || p.hunger == 0 || p.happy == 0 {
        Expr::Sad
    } else if m.fx_is(Fx::Hearts).is_some() || p.happy >= 3 {
        Expr::Happy
    } else {
        Expr::Idle
    };
    if eating && base != Expr::Sleeping {
        Expr::Eating
    } else if matches!(base, Expr::Idle | Expr::Happy | Expr::Sad) && m.beat() % 9 == 8 {
        Expr::Blink
    } else {
        base
    }
}

/// Awake, alive, hatched and not ill: the only time the machine's state shows on the pet.
fn calm(p: &Pet) -> bool {
    p.alive() && p.stage != Stage::Egg && !p.asleep && !p.sick
}

/// The pet's offset from where it normally stands: it only bobs up and down, never sideways.
fn motion(m: &Moment) -> (i32, i32) {
    let p = m.pet;
    let beat = m.beat() as i32;
    let dy = match p.stage {
        Stage::Dead | Stage::Egg => 0,
        _ if p.asleep => -(beat / 3 % 2),
        _ => -(beat % 2),
    };
    (0, dy)
}

/// The pet with its face (or egg cracks) laid on.
fn pet_sprite(m: &Moment, c: &Character) -> Sprite {
    let p = m.pet;
    let mut overlay: Vec<(i32, i32, Pal)> = Vec::new();
    if p.stage == Stage::Egg {
        overlay = cracks(p);
    } else if let Some(spot) = c.face {
        let dizzy = p.load.low_ram && calm(p) && expression(m) != Expr::Sleeping;
        let face: Vec<(i32, i32, Pal)> =
            if dizzy { faces::dizzy(spot.style, m.beat() as usize) } else { faces::pixels(faces::grid(spot.style, expression(m))).collect() };
        overlay.extend(face.into_iter().map(|(dx, dy, p)| (spot.x + dx, spot.y + dy, p)));
    }
    assemble(p, c, &overlay, m.beat())
}

/// Cracks in the shell as an egg gets close to hatching.
fn cracks(p: &Pet) -> Vec<(i32, i32, Pal)> {
    let progress = p.stage_age as f32 / 30.0;
    let Some(i) = [(0.5, 0), (0.8, 1)].iter().rev().find(|(at, _)| progress > *at).map(|(_, i)| *i) else { return Vec::new() };
    let (x, y) = cast::EGG_CRACK_AT;
    faces::pixels(&faces::EGG_CRACKS[i]).map(|(dx, dy, p)| (x + dx, y + dy, p)).collect()
}

/// The pet with a given face (an egg shows its cracks instead), for the games and the gallery.
pub fn posed(p: &Pet, e: Expr) -> Sprite {
    let c = character(p);
    let face: Vec<(i32, i32, Pal)> = match c.face {
        None if p.stage == Stage::Egg => cracks(p),
        Some(spot) => faces::pixels(faces::grid(spot.style, e)).map(|(x, y, col)| (spot.x + x, spot.y + y, col)).collect(),
        None => Vec::new(),
    };
    assemble(p, &c, &face, 1)
}

/// Where the pet's sprite goes in the room.
#[derive(Clone, Copy)]
struct Place {
    x: i32,
    y: i32,
    size: i32,
    /// The top of what is actually drawn, below the sprite's empty rows.
    head: i32,
}

fn place(m: &Moment, c: &Character, s: Size) -> Place {
    let (dx, dy) = motion(m);
    let body = art::body(c);
    let y = s.floor() - body.bottom() + dy;
    Place { x: s.centre() - c.size as i32 / 2 + dx, y, size: c.size as i32, head: y + body.top() }
}

fn draw_poop(f: &mut Frame, m: &Moment, s: Size) {
    let poop = art::body(&cast::POOP);
    for (i, &off) in POOP_X.iter().take(m.pet.poops.min(3) as usize).enumerate() {
        let (x, y) = (s.centre() + off, s.floor() + 2 - poop.bottom());
        f.sprite(poop, x, y);
        // Two specks of stink, hopping about above it.
        let hop = ((m.beat() + i as u64) % 2) as i32;
        f.set(x + 4 + hop, y - 3, Pal::Lime);
        f.set(x + 10 - hop, y - 5, Pal::Lime);
    }
}

fn draw_dead(f: &mut Frame, m: &Moment, s: Size) {
    let grave = art::body(&cast::GRAVE);
    f.sprite(grave, s.centre() - 16, s.floor() + 1 - grave.bottom());
    let float = [0, -1, -2, -1][(m.beat() / 2 % 4) as usize];
    let (x, y) = (s.centre() - 12, s.floor() + 1 - grave.bottom() + grave.top() - 20 + float);
    f.sprite(art::body(&cast::GHOST), x, y);
    f.grid(&faces::GHOST_FACE, x + cast::GHOST_FACE_AT.0, y + cast::GHOST_FACE_AT.1);
}

/// A white speech bubble with the icon for what the pet wants, and a tail pointing at it.
fn draw_bubble(f: &mut Frame, x: i32, y: i32, key: char) {
    f.boxed(x, y, 18, 16, Pal::White, Pal::Ink);
    for (cx, cy) in [(x, y), (x + 17, y), (x, y + 15), (x + 17, y + 15)] {
        f.set(cx, cy, Pal::Night);
    }
    f.rect(x + 2, y + 16, 2, 1, Pal::White);
    f.set(x + 1, y + 16, Pal::Ink);
    f.set(x + 4, y + 16, Pal::Ink);
    f.set(x + 2, y + 17, Pal::Ink);
    f.set(x + 3, y + 17, Pal::Ink);
    if let Some(icon) = icons::for_key(key) {
        f.grid(icon, x + 3, y + 2);
    }
}

fn draw_effects(f: &mut Frame, m: &Moment, at: Place, s: Size) {
    let Some((fx, t)) = m.fx else { return };
    let (px, py, size, cx) = (at.x, at.head, at.size, at.x + at.size / 2);
    match fx {
        Fx::Hearts if t < 0.9 => {
            for i in 0..3 {
                f.grid(icons::HEART, cx - 18 + i * 13, py - 4 - (t * 22.0) as i32 + (i % 2) * 4);
            }
        }
        Fx::Eat if t < 0.5 => {
            let x = s.w - 10 - ((t / 0.5) * (s.w - 10 - cx - 8) as f32) as i32;
            f.grid(icons::APPLE, x, at.y + size / 2);
        }
        Fx::Heal => {
            for (i, (x, y)) in [(-20, 4), (14, 0), (-16, 22), (18, 18)].into_iter().enumerate() {
                if ((t * 12.0) as usize + i) % 2 == 0 {
                    f.grid(icons::SPARKLE, cx + x, py + y);
                }
            }
        }
        Fx::Clean => {
            let x = (t * (s.w + 24) as f32) as i32 - 16;
            f.grid(icons::CLEAN, x, s.floor() - 10);
            for i in 0..3 {
                f.grid(icons::SPARKLE, x - 8 - i * 9, s.floor() - 14 + (i % 2) * 6);
            }
        }
        Fx::Scold if (t * 10.0) as i32 % 2 == 0 => f.grid(icons::ANGER, px + 2, py - 2),
        Fx::Evolve => {
            for (i, (x, y)) in [(-30, 0), (26, -4), (-36, 20), (32, 18), (-4, -14), (-22, 34), (20, 34)].into_iter().enumerate() {
                if ((t * 16.0) as usize + i) % 3 != 0 {
                    f.grid(icons::SPARKLE, cx + x, py + y);
                }
            }
        }
        _ => {}
    }
}

fn draw_machine(f: &mut Frame, m: &Moment, c: &Character, at: Place, s: Size) {
    let p = m.pet;
    if !calm(p) {
        return;
    }
    if p.load.busy_cpu {
        if let Some(spot) = c.face {
            // Sweat drops slide down either side of the head.
            let slide = (m.beat() % 3) as i32;
            f.grid(icons::SWEAT, at.x + spot.x - 8, at.y + spot.y - 3 + slide);
            f.grid(icons::SWEAT, at.x + spot.x + 16, at.y + spot.y - 2 + slide);
        }
    }
    if p.load.low_battery {
        let (dx, dy) = s.shift();
        f.grid(icons::BATTERY, 70 + dx, 24 + dy);
    }
}

fn draw_sleep(f: &mut Frame, m: &Moment, at: Place) {
    for i in 0..2 {
        let phase = ((m.beat() / 2) as i32 + i * 2) % 4;
        f.grid(icons::Z, at.x + at.size - 4 + i * 7, at.head - phase * 4 - i * 3);
    }
}

/// The room at this moment, `w` x `h` (at least the standard 160x116 where the art allows).
pub fn room(m: &Moment, w: i32, h: i32) -> Frame {
    let p = m.pet;
    let light = light(p);
    let s = Size { w, h };
    let mut f = Frame::new(w, h, Pal::Ink);
    // The standard room sits at the bottom centre of the master art; take what fits around it.
    let master = art::room(light);
    let (cw, ch) = (w.min(master.w), h.min(master.h));
    let (sx, sy) = ((master.w - W) / 2 + W / 2 - cw / 2, master.h - ch);
    f.region(master, (sx, sy, cw, ch), (w - cw) / 2, h - ch);
    // The shadow along the ceiling, wherever the top of the room is cropped to.
    f.rect(0, 0, w, 3, if light == Light::Dark { Pal::Ink } else { Pal::Night });
    let (dx, dy) = s.shift();
    if light != Light::Day {
        for (i, &(x, y)) in TWINKLE.iter().enumerate() {
            if (m.beat() as usize + i * 3) % 7 == 0 {
                f.set(x + dx, y + dy, Pal::Night);
            }
        }
    }
    draw_poop(&mut f, m, s);
    if !p.alive() {
        draw_dead(&mut f, m, s);
        return f;
    }
    let c = character(p);
    let at = place(m, &c, s);
    let sprite = pet_sprite(m, &c);
    let flash = m.fx_is(Fx::Evolve).is_some_and(|t| t < 0.6 && (t * 14.0) as i32 % 2 == 0);
    f.sprite_with(&sprite, at.x, at.y, |c| if flash { Pal::White } else { c });
    if p.asleep {
        draw_sleep(&mut f, m, at);
    }
    draw_machine(&mut f, m, &c, at, s);
    if let Some((key, _)) = p.top_need() {
        if !p.asleep {
            let x = (at.x + at.size - 6).min(w - 19);
            draw_bubble(&mut f, x, (at.head - 16 + (m.beat() % 2) as i32).max(1), key);
        }
    }
    draw_effects(&mut f, m, at, s);
    f
}

/// The room area for the modes that show the room. Menus and lists dim it behind their text.
pub fn area(app: &App, w: i32, h: i32) -> Frame {
    let mut f = room(&Moment::of(app), w, h);
    if !matches!(app.mode, Mode::Normal) {
        f.tint(0, 0, w, h, Pal::dark);
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysinfo::Strain;

    fn pet(stage: Stage) -> Pet {
        let mut p = Pet::new(3);
        p.stage = stage;
        p.species = "Babytchi".into();
        p.hunger = 4;
        p.happy = 2;
        p
    }

    fn look(p: &Pet, ms: u64) -> Frame {
        room(&Moment { pet: p, fx: None, ms }, W, H)
    }

    #[test]
    fn every_stage_and_effect_draws_without_panicking() {
        let fxs = [None, Some(Fx::Eat), Some(Fx::Hearts), Some(Fx::Heal), Some(Fx::Clean), Some(Fx::Scold), Some(Fx::Evolve)];
        for stage in [Stage::Egg, Stage::Baby, Stage::Child, Stage::Teen, Stage::Adult, Stage::Dead] {
            let mut p = pet(stage);
            p.poops = 3;
            p.disc_call = Some(0);
            for fx in fxs {
                for t in [0.0, 0.4, 0.95] {
                    for ms in [0, 2_900, 7_000] {
                        room(&Moment { pet: &p, fx: fx.map(|f| (f, t)), ms }, W, H);
                    }
                }
            }
        }
    }

    #[test]
    fn every_form_has_its_own_sprite_and_parts_follow_the_setting() {
        let mut seen = Vec::new();
        for (stage, species) in crate::app::FORMS.iter().filter(|f| f.0 != Stage::Dead) {
            let mut p = pet(*stage);
            p.species = species.to_string();
            let c = character(&p);
            assert!(!seen.contains(&c.name), "{species} shares a sprite with another form");
            seen.push(c.name);
        }
        let mut p = pet(Stage::Adult);
        p.species = "Mametchi".into();
        let bare = look(&p, 0);
        p.cfg.animate_parts = true;
        assert_ne!(bare, look(&p, 0), "the ears show when animate_parts is on");
        assert_ne!(look(&p, 0), look(&p, 700), "and they move");
    }

    #[test]
    fn a_bigger_room_keeps_the_pet_at_the_bottom_centre_and_draws_at_any_size() {
        let p = pet(Stage::Baby);
        let m = Moment { pet: &p, fx: None, ms: 0 };
        for (w, h) in [(160, 116), (240, 116), (400, 200), (544, 244), (700, 300), (100, 60)] {
            let f = room(&m, w, h);
            assert_eq!((f.w, f.h), (w, h));
        }
        // The pet is in the same place relative to the bottom centre of any room.
        let (small, big) = (room(&m, 160, 116), room(&m, 400, 200));
        let pink = |f: &Frame, x0: i32, y0: i32| (0..32).flat_map(|y| (0..32).map(move |x| (x, y))).filter(|&(x, y)| f.get(x0 + x, y0 + y) == Some(Pal::Pink)).count();
        assert_eq!(pink(&small, 64, 70), pink(&big, 184, 154));
    }

    #[test]
    fn the_pet_never_moves_sideways() {
        for stage in [Stage::Egg, Stage::Baby, Stage::Adult] {
            let mut p = pet(stage);
            p.stage_age = 20;
            for state in 0..3 {
                p.sick = state == 1;
                p.asleep = state == 2;
                let xs: Vec<i32> = (0..40u64).map(|i| motion(&Moment { pet: &p, fx: Some((Fx::Scold, 0.1)), ms: i * 350 }).0).collect();
                assert!(xs.iter().all(|&x| x == 0), "{stage:?} state {state}: {xs:?}");
            }
        }
    }

    #[test]
    fn the_machine_shows_on_a_calm_pet_only() {
        let calm = pet(Stage::Baby);
        for strain in [
            Strain { busy_cpu: true, ..Strain::default() },
            Strain { low_ram: true, ..Strain::default() },
            Strain { low_battery: true, ..Strain::default() },
        ] {
            let mut p = pet(Stage::Baby);
            p.load = strain;
            assert_ne!(look(&calm, 0), look(&p, 0), "{strain:?} should show");
            let mut egg = pet(Stage::Egg);
            let before = look(&egg, 0);
            egg.load = strain;
            assert_eq!(before, look(&egg, 0), "an egg ignores {strain:?}");
        }
        let mut dizzy = pet(Stage::Baby);
        dizzy.load.low_ram = true;
        assert_ne!(look(&dizzy, 0), look(&dizzy, 350), "the dizzy eyes spin");
    }

    #[test]
    fn sleeping_picks_the_room_light_and_shows_zs() {
        let mut p = pet(Stage::Baby);
        assert_eq!(light(&p), Light::Day);
        p.asleep = true;
        assert_eq!(light(&p), Light::Lamp);
        p.light_on = false;
        assert_eq!(light(&p), Light::Dark);
        assert_eq!(expression(&Moment { pet: &p, fx: None, ms: 0 }), Expr::Sleeping);
    }

    #[test]
    fn moods_follow_the_needs() {
        let face = |p: &Pet, fx: Option<(Fx, f32)>| expression(&Moment { pet: p, fx, ms: 0 });
        let mut p = pet(Stage::Baby);
        assert_eq!(face(&p, None), Expr::Idle);
        p.happy = 4;
        assert_eq!(face(&p, None), Expr::Happy);
        p.hunger = 0;
        assert_eq!(face(&p, None), Expr::Sad);
        p.sick = true;
        assert_eq!(face(&p, None), Expr::Sick);
        p.sick = false;
        p.hunger = 3;
        assert_eq!(face(&p, Some((Fx::Eat, 0.5))), Expr::Eating);
        assert_eq!(expression(&Moment { pet: &p, fx: None, ms: 350 * 8 }), Expr::Blink);
    }

    #[test]
    fn poop_and_a_need_bubble_change_the_room() {
        let clean = pet(Stage::Baby);
        let mut dirty = pet(Stage::Baby);
        dirty.poops = 1;
        assert_ne!(look(&clean, 0), look(&dirty, 0));
        let mut hungry = pet(Stage::Baby);
        hungry.hunger = 0;
        let bubble_white = |f: &Frame| (0..H).flat_map(|y| (0..W).map(move |x| (x, y))).filter(|&(x, y)| f.get(x, y) == Some(Pal::White)).count();
        assert!(bubble_white(&look(&hungry, 0)) > bubble_white(&look(&clean, 0)) + 50, "a hungry pet shows a bubble");
    }
}
