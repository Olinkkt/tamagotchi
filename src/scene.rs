//! The room and everything in it: the pet, its moods and reactions, sleep, speech bubbles
//! and the one-shot effects after an action. Drawn in room pixels (160x116).

use crate::app::{App, Mode};
use crate::art::{self, Light, Sprite};
use crate::cast::{self, Character};
use crate::egg;
use crate::faces::{self, Expr};
use crate::frame::Frame;
use crate::icons;
use crate::palette::Pal;
use crate::pet::{Pet, Stage};
use crate::toilet;

/// A one-shot animation after an action: what happened, picked by `App::react`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fx {
    Eat,
    Snack,
    Hearts,
    Heal,
    Scold,
    Evolve,
}

impl Fx {
    pub fn duration_secs(self) -> f32 {
        match self {
            Fx::Eat | Fx::Snack => 1.8,
            Fx::Hearts => 1.6,
            Fx::Heal => 1.4,
            Fx::Scold => 1.2,
            Fx::Evolve => 2.6,
        }
    }
}

/// The standard room size, which the art was drawn for. Bigger windows get a bigger room that
/// carries the wall and floor on around it, with everything kept to the bottom centre.
pub const W: i32 = 160;
pub const H: i32 = 116;
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
    /// Seconds since the egg hatched, while its shell still lies on the floor.
    pub shell: Option<f32>,
    /// Milliseconds the game has counted since the pet's last whole second of age.
    pub sub_ms: u64,
}

impl<'a> Moment<'a> {
    pub fn of(app: &'a App) -> Moment<'a> {
        Moment { pet: &app.pet, fx: app.current_fx(), ms: app.started.elapsed().as_millis() as u64, shell: app.shell_age(), sub_ms: app.sub_ms() }
    }

    /// The 350 ms beat most of the little animations step on.
    fn beat(&self) -> u64 {
        self.ms / 350
    }

    /// The egg's age in milliseconds, which its whole 30-second show is worked out from.
    fn egg_ms(&self) -> u64 {
        self.pet.stage_age * 1000 + self.sub_ms
    }

    fn egg_pose(&self) -> egg::Pose {
        egg::pose(egg::seed(self.pet), self.egg_ms())
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
    let eating = m.fx_is(Fx::Eat).or(m.fx_is(Fx::Snack)).is_some_and(|f| (0.3..0.9).contains(&f) && ((f * 8.0) as i32) % 2 == 0);
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

/// How long the shell lies on the floor after hatching.
pub const SHELL_SECS: f32 = 7.0;
/// The hatching, in seconds after the egg breaks (see `egg`): the baby pops out, the cap flies
/// off in an arc and the bowl hops aside.
const HATCH_POP: f32 = 0.45;
const HATCH_BOWL: f32 = 0.6;
const HATCH_CAP: f32 = 0.8;
const POP_PX: f32 = 9.0;
const BOWL_HOP_PX: f32 = 5.0;
const CAP_ARC_PX: f32 = 26.0;

/// The pet's offset from where it normally stands. It only bobs up and down; the one sideways
/// movement is an egg, which shakes on the schedule in `egg`.
fn motion(m: &Moment) -> (i32, i32) {
    let beat = m.beat() as i32;
    match m.pet.stage {
        Stage::Dead => (0, 0),
        Stage::Egg => {
            let pose = m.egg_pose();
            (pose.dx, pose.dy)
        }
        _ if m.pet.asleep => (0, -(beat / 3 % 2)),
        // Stands still while it eats, so the food stays in its mouth.
        _ if m.fx_is(Fx::Eat).or(m.fx_is(Fx::Snack)).is_some() => (0, 0),
        _ => (0, -(beat % 2)),
    }
}

/// The pet with its face (or egg cracks) laid on.
fn pet_sprite(m: &Moment, c: &Character) -> Sprite {
    let p = m.pet;
    let mut overlay: Vec<(i32, i32, Pal)> = Vec::new();
    if p.stage == Stage::Egg {
        return egg_sprite(m, c);
    } else if let Some(spot) = c.face {
        let dizzy = p.load.low_ram && calm(p) && expression(m) != Expr::Sleeping;
        let face: Vec<(i32, i32, Pal)> =
            if dizzy { faces::dizzy(spot.style, m.beat() as usize) } else { faces::pixels(faces::grid(spot.style, expression(m))).collect() };
        overlay.extend(face.into_iter().map(|(dx, dy, p)| (spot.x + dx, spot.y + dy, p)));
    }
    assemble(p, c, &overlay, m.beat())
}

/// The egg with the cracks it has so far. Once the crack runs all the way across it splits along
/// it: the bowl stays planted and the cap lifts and rocks. Before that the whole egg tilts.
fn egg_sprite(m: &Moment, c: &Character) -> Sprite {
    let pose = m.egg_pose();
    let sprite = assemble(m.pet, c, &cracks_at(pose.cracks), m.beat());
    let middle = c.size as f32 / 2.0;
    if pose.lift == 0 {
        // Tilting pivots on the feet, so the egg never looks as if it has left the ground.
        return sprite.tilted(pose.tilt, middle, art::body(c).bottom() as f32);
    }
    let (mut cap, mut bowl) = sprite.halves(egg::split_row);
    // The cap's cut edge gets an outline of its own, like the crack gives the bowl.
    for x in 0..cap.w {
        let y = egg::split_row(x) - 1;
        if cap.get(x, y).is_some() {
            cap.put(x, y, Pal::Ink);
        }
    }
    let hinge = egg::split_row(c.size as i32 / 2) as f32;
    bowl.over(&cap.raised(pose.lift).tilted(pose.tilt, middle, hinge));
    bowl
}

/// The cracks of crack stage `n` (1 to 6); none for a whole egg.
fn cracks_at(n: usize) -> Vec<(i32, i32, Pal)> {
    if n == 0 {
        return Vec::new();
    }
    let (x, y) = cast::EGG_CRACK_AT;
    faces::pixels(&faces::EGG_CRACKS[n - 1]).map(|(dx, dy, c)| (x + dx, y + dy, c)).collect()
}

/// The middle of the pet in an area `w` x `h`: where the iris wipe closes and opens.
pub fn pet_spot(app: &App, w: i32, h: i32) -> (i32, i32) {
    if let Mode::Toilet(visit) = &app.mode {
        return toilet::spot(app, visit, w, h);
    }
    let c = character(&app.pet);
    let at = place(&Moment::of(app), &c, Size { w, h });
    (at.x + at.size / 2, (at.head + at.y + art::body(&c).bottom()) / 2)
}

/// The pet as it sits in the washroom booth.
pub fn seated(app: &App) -> Sprite {
    pet_sprite(&Moment::of(app), &character(&app.pet))
}

/// The pet with a given face (an egg shows its cracks instead), for the games and the gallery.
pub fn posed(p: &Pet, e: Expr) -> Sprite {
    let c = character(p);
    let face: Vec<(i32, i32, Pal)> = match c.face {
        None if p.stage == Stage::Egg => cracks_at(egg::crack_stage(egg::seed(p), p.stage_age * 1000)),
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

/// 0 at both ends and 1 in the middle: the height of a jump or a throw as it goes from 0 to 1.
fn arc(u: f32) -> f32 {
    4.0 * u * (1.0 - u)
}

/// How far (0 to 1) through a stretch of the hatching, starting `from` seconds in and lasting `len`.
fn progress(t: f32, from: f32, len: f32) -> f32 {
    ((t - from) / len).clamp(0.0, 1.0)
}

/// Where the shell's bowl and cap are (top-left corners), `t` seconds after the egg burst: at
/// first exactly where the two halves of the egg were, then the bowl hops left and the cap is
/// thrown right, landing in the places they stay.
fn shell_pieces(t: f32, s: Size) -> [(i32, i32); 2] {
    let cells = art::cells(&cast::SHELL);
    let floor = s.floor() + 2;
    let (bowl_y, cap_y) = ((floor - cells[0].bottom()) as f32, (floor - cells[1].bottom()) as f32);
    let (start_x, bowl_x, cap_x) = ((s.centre() - 12) as f32, (s.centre() - 46) as f32, (s.centre() + 22) as f32);
    // The egg stands on the floor and its cap is lifted one pixel (see `egg::pose`).
    let egg = art::body(&cast::EGG);
    let bowl_start = (s.floor() - cells[0].bottom()) as f32;
    let cap_start = (s.floor() - egg.bottom() + egg.top() - 1 - cells[1].top()) as f32;
    let lerp = |a: f32, b: f32, u: f32| a + (b - a) * u;
    let ease = |u: f32| 1.0 - (1.0 - u) * (1.0 - u);
    let (ub, uc) = (progress(t, 0.0, HATCH_BOWL), progress(t, 0.0, HATCH_CAP));
    let bowl = (lerp(start_x, bowl_x, ease(ub)), lerp(bowl_start, bowl_y, ease(ub)) - BOWL_HOP_PX * arc(ub));
    let cap = (lerp(start_x, cap_x, ease(uc)), lerp(cap_start, cap_y, uc) - CAP_ARC_PX * arc(uc));
    [(bowl.0.round() as i32, bowl.1.round() as i32), (cap.0.round() as i32, cap.1.round() as i32)]
}

/// The two halves of the egg: thrown apart when it bursts, then left on the floor either side of
/// the pet; they blink out at the end.
fn draw_shell(f: &mut Frame, t: f32, s: Size) {
    if t > SHELL_SECS - 1.5 && (t * 8.0) as i32 % 2 == 1 {
        return;
    }
    let cells = art::cells(&cast::SHELL);
    let [bowl, cap] = shell_pieces(t, s);
    f.sprite(&cells[0], bowl.0, bowl.1);
    f.sprite(&cells[1], cap.0, cap.1);
}

/// Bits of shell flying off the cracking egg.
fn draw_chips(f: &mut Frame, m: &Moment, at: Place) {
    let (x, y) = (at.x + at.size / 2, at.head);
    for chip in egg::chips(egg::seed(m.pet), m.egg_ms()) {
        // Shell-coloured, with a shaded underside.
        f.rect(x + chip.dx, y + chip.dy, 2, 1, if chip.spotted { Pal::Teal } else { Pal::White });
        f.rect(x + chip.dx, y + chip.dy + 1, 2, 1, Pal::Mist);
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

fn draw_effects(f: &mut Frame, m: &Moment, c: &Character, at: Place, s: Size) {
    let Some((fx, t)) = m.fx else { return };
    let (px, py, size, cx) = (at.x, at.head, at.size, at.x + at.size / 2);
    match fx {
        Fx::Hearts if t < 0.9 => {
            for i in 0..3 {
                f.grid(icons::HEART, cx - 18 + i * 13, py - 4 - (t * 22.0) as i32 + (i % 2) * 4);
            }
        }
        Fx::Eat | Fx::Snack if t < 0.9 => {
            // Tossed in from the top right, lands in the pet's mouth, then eaten in two bites (or licked, for candy).
            let (ex, ey) = match c.face {
                Some(spot) => {
                    let g = faces::grid(spot.style, Expr::Eating);
                    (px + spot.x + g[0].len() as i32 / 2 - 3, at.y + spot.y + g.len() as i32 - 2)
                }
                None => (px + size - 2, at.y + size - 8),
            };
            let (sx, sy) = (s.w - 8, 4);
            let u = (t / 0.25).min(1.0);
            let hop = if t > 0.25 { let v = ((t - 0.25) / 0.08).min(1.0); (2.0 * 4.0 * v * (1.0 - v)) as i32 } else { 0 };
            let x = sx + ((ex - sx) as f32 * u) as i32;
            let y = sy + ((ey - sy) as f32 * u) as i32 - (24.0 * 4.0 * u * (1.0 - u)) as i32 - hop;
            let [whole, bitten, last] = if fx == Fx::Snack {
                [icons::CANDY, icons::CANDY_UNWRAPPED, icons::CANDY_LICKED]
            } else {
                [icons::APPLE, icons::APPLE_BITTEN, icons::APPLE_CORE]
            };
            let icon = if t < 0.5 { whole } else if t < 0.7 { bitten } else { last };
            let crumb = if fx == Fx::Snack { "v" } else { "n" };
            f.grid(icon, x, y);
            for bite in [0.5, 0.7, 0.88] {
                if (bite..bite + 0.06).contains(&t) {
                    let k = ((t - bite) * 100.0) as i32;
                    f.grid(&[crumb], ex + 6 + k, ey - k);
                    f.grid(&[crumb], ex + 2 - k / 2, ey - 2 - k);
                }
            }
            if t >= 0.82 {
                f.grid(icons::SPARKLE, ex, ey - 2);
            }
        }
        Fx::Heal => {
            for (i, (x, y)) in [(-20, 4), (14, 0), (-16, 22), (18, 18)].into_iter().enumerate() {
                if ((t * 12.0) as usize + i) % 2 == 0 {
                    f.grid(icons::SPARKLE, cx + x, py + y);
                }
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
    if let Some(t) = m.shell {
        draw_shell(&mut f, t, s);
    }
    if !p.alive() {
        draw_dead(&mut f, m, s);
        return f;
    }
    let c = character(p);
    let mut at = place(m, &c, s);
    if let Some(t) = m.shell {
        // The baby pops up out of the shell.
        let pop = (POP_PX * arc(progress(t, 0.0, HATCH_POP))).round() as i32;
        (at.y, at.head) = (at.y - pop, at.head - pop);
    }
    let sprite = pet_sprite(m, &c);
    let egg_flash = p.stage == Stage::Egg && m.egg_pose().flash;
    let flash = egg_flash || m.fx_is(Fx::Evolve).is_some_and(|t| t < 0.6 && (t * 14.0) as i32 % 2 == 0);
    f.sprite_with(&sprite, at.x, at.y, |c| if flash { Pal::White } else { c });
    if p.stage == Stage::Egg {
        draw_chips(&mut f, m, at);
    }
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
    draw_effects(&mut f, m, &c, at, s);
    f
}

/// The room area for the modes that show the room. Menus and lists dim it behind their text.
pub fn area(app: &App, w: i32, h: i32) -> Frame {
    if let Mode::Toilet(visit) = &app.mode {
        return toilet::area(app, visit, w, h);
    }
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
        room(&Moment { pet: p, fx: None, ms, shell: None, sub_ms: 0 }, W, H)
    }

    #[test]
    fn every_stage_and_effect_draws_without_panicking() {
        let fxs = [None, Some(Fx::Eat), Some(Fx::Snack), Some(Fx::Hearts), Some(Fx::Heal), Some(Fx::Scold), Some(Fx::Evolve)];
        for stage in [Stage::Egg, Stage::Baby, Stage::Child, Stage::Teen, Stage::Adult, Stage::Dead] {
            let mut p = pet(stage);
            p.urge = Some(0);
            p.disc_call = Some(0);
            for fx in fxs {
                for t in [0.0, 0.4, 0.95] {
                    for ms in [0, 2_900, 7_000] {
                        room(&Moment { pet: &p, fx: fx.map(|f| (f, t)), ms, shell: None, sub_ms: 0 }, W, H);
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
        let m = Moment { pet: &p, fx: None, ms: 0, shell: None, sub_ms: 0 };
        for (w, h) in [(160, 116), (240, 116), (400, 200), (544, 244), (700, 300), (100, 60)] {
            let f = room(&m, w, h);
            assert_eq!((f.w, f.h), (w, h));
        }
        // The pet is in the same place relative to the bottom centre of any room.
        let (small, big) = (room(&m, 160, 116), room(&m, 400, 200));
        let pink = |f: &Frame, x0: i32, y0: i32| (0..32).flat_map(|y| (0..32).map(move |x| (x, y))).filter(|&(x, y)| f.get(x0 + x, y0 + y) == Some(Pal::Pink)).count();
        assert_eq!(pink(&small, 64, 70), pink(&big, 184, 154));
    }

    fn egg_at(age: u64) -> Pet {
        let mut egg = pet(Stage::Egg);
        egg.stage_age = age;
        egg
    }

    #[test]
    fn an_egg_moves_by_its_own_schedule() {
        let egg = egg_at(27);
        let shaking = (26..29u64).any(|age| {
            let p = egg_at(age);
            (0..1000u64).step_by(10).any(|sub_ms| motion(&Moment { pet: &p, fx: None, ms: 0, shell: None, sub_ms }).0 != 0)
        });
        assert!(shaking, "an egg in its last seconds is shaking");
        assert!(egg_at(1).stage_age == 1 && motion(&Moment { pet: &egg_at(1), fx: None, ms: 99_999, shell: None, sub_ms: 0 }) == (0, 0));
        // The animation clock plays no part: only the egg's own age does.
        let at = |ms| motion(&Moment { pet: &egg, fx: None, ms, shell: None, sub_ms: 300 });
        assert_eq!(at(0), at(12_345));
    }

    #[test]
    fn nothing_but_an_egg_moves_sideways() {
        for stage in [Stage::Baby, Stage::Adult] {
            let mut p = pet(stage);
            for state in 0..3 {
                p.sick = state == 1;
                p.asleep = state == 2;
                let xs: Vec<i32> = (0..40u64).map(|i| motion(&Moment { pet: &p, fx: Some((Fx::Scold, 0.1)), ms: i * 350, shell: None, sub_ms: 0 }).0).collect();
                assert!(xs.iter().all(|&x| x == 0), "{stage:?} state {state}: {xs:?}");
            }
        }
    }

    #[test]
    fn the_shell_lies_on_the_floor_after_hatching_and_blinks_out() {
        let p = pet(Stage::Baby);
        let room_at = |shell: Option<f32>| room(&Moment { pet: &p, fx: None, ms: 0, shell, sub_ms: 0 }, W, H);
        let none = room_at(None);
        assert!(none != room_at(Some(1.0)), "the halves are on the floor");
        assert!(room_at(Some(2.0)) == room_at(Some(3.0)), "and stay put once landed");
        assert!(none == room_at(Some(5.7)), "at 5.7s it is in a blink");
        assert!(none != room_at(Some(5.8)), "and back for a moment");
    }

    #[test]
    fn the_cracks_get_bigger_each_stage() {
        let counts: Vec<usize> = (0..=6).map(|n| cracks_at(n).len()).collect();
        assert_eq!(counts[0], 0);
        assert_eq!(counts[1], 3, "the first crack is three pixels");
        assert!(counts.windows(2).all(|w| w[0] < w[1]), "{counts:?}");
        assert!(cracks_at(6).iter().all(|c| c.2 == Pal::Ink || c.2 == Pal::Night), "ink cracks only, no glow");
    }

    #[test]
    fn the_cracks_grow_outwards_from_the_middle() {
        let xs = |n: usize| {
            let x: Vec<i32> = cracks_at(n).iter().map(|c| c.0).collect();
            (*x.iter().min().unwrap(), *x.iter().max().unwrap())
        };
        let middle = cast::EGG_CRACK_AT.0 + 9;
        for n in 1..=6 {
            let (lo, hi) = xs(n);
            assert!(lo <= middle && hi >= middle, "stage {n} covers the middle");
            if n > 1 {
                assert!(xs(n - 1).0 >= lo && xs(n - 1).1 <= hi, "stage {n} only adds");
            }
        }
        assert_eq!(xs(6).1 - xs(6).0 + 1, 18, "the last one runs the whole width");
    }

    /// The rows with something in them in column `x`.
    fn column(sp: &Sprite, x: i32) -> Vec<i32> {
        (0..sp.h).filter(|&y| sp.get(x, y).is_some()).collect()
    }

    fn pixels_of(sp: &Sprite) -> Vec<Option<Pal>> {
        (0..sp.h).flat_map(|y| (0..sp.w).map(move |x| sp.get(x, y))).collect()
    }

    fn egg_frame(p: &Pet, sub_ms: u64) -> Sprite {
        egg_sprite(&Moment { pet: p, fx: None, ms: 0, shell: None, sub_ms }, &character(p))
    }

    #[test]
    fn the_split_egg_ends_exactly_where_the_burst_starts() {
        let mut p = pet(Stage::Egg);
        p.stage_age = 29;
        let m = Moment { pet: &p, fx: None, ms: 0, shell: None, sub_ms: 800 };
        let (egg, s) = (egg_frame(&p, 800), Size { w: W, h: H });
        let at = place(&m, &character(&p), s);
        let ([top, bottom], [bowl, cap]) = ({ let (t, b) = egg.halves(egg::split_row); [t, b] }, shell_pieces(0.0, s));
        let cells = art::cells(&cast::SHELL);
        // The bowl stands where the bottom half did, the cap where the lifted top half did.
        assert_eq!(at.y + bottom.bottom(), bowl.1 + cells[0].bottom(), "bowl feet");
        assert_eq!(at.y + top.top(), cap.1 + cells[1].top(), "cap top");
        let left = |sp: &Sprite| (0..sp.w).find(|&x| !column(sp, x).is_empty()).unwrap();
        assert_eq!(at.x + left(&bottom), bowl.0 + left(&cells[0]), "bowl left");
        assert_eq!(at.x + left(&top), cap.0 + left(&cells[1]), "cap left");
        // A thin gap along the whole zigzag, wherever the two halves meet.
        for x in 0..egg.w {
            let (above, below) = (column(&top, x), column(&bottom, x));
            if let (Some(a), Some(b)) = (above.last(), below.first()) {
                assert!((2..=3).contains(&(b - a)), "column {x}: gap {} px", b - a - 1);
            }
        }
    }

    #[test]
    fn the_cap_rocks_but_the_bowl_stays_planted() {
        let mut p = pet(Stage::Egg);
        p.stage_age = 28;
        let bowls: Vec<Vec<Option<Pal>>> = (0..1000).step_by(25).map(|ms| pixels_of(&egg_frame(&p, ms).halves(egg::split_row).1)).collect();
        assert!(bowls.windows(2).all(|w| w[0] == w[1]), "the bowl never moves or tilts");
        let caps: Vec<Vec<Option<Pal>>> = (0..1000).step_by(25).map(|ms| pixels_of(&egg_frame(&p, ms).halves(egg::split_row).0)).collect();
        assert!(caps.windows(2).any(|w| w[0] != w[1]), "but the cap does");
    }

    #[test]
    fn the_egg_bursts_then_the_baby_pops_out_and_the_shell_lands() {
        let p = pet(Stage::Baby);
        let at = |t: f32| room(&Moment { pet: &p, fx: None, ms: 0, shell: Some(t), sub_ms: 0 }, W, H);
        // The baby's pink body, above the rug.
        let pink = |f: &Frame| (66..88).flat_map(|y| (60..100).map(move |x| (x, y))).filter(|&(x, y)| f.get(x, y) == Some(Pal::Pink)).count();
        assert!(pink(&at(0.2)) > 100, "the baby is out");
        let s = Size { w: W, h: H };
        let (start, rest) = (shell_pieces(0.0, s), shell_pieces(HATCH_CAP, s));
        assert_eq!(rest, shell_pieces(3.0, s), "it stays where it landed");
        assert!(start[0].0 > rest[0].0 && start[1].0 < rest[1].0, "the bowl goes left and the cap goes right");
        let mid = shell_pieces(HATCH_CAP / 2.0, s);
        assert!(mid[1].1 < start[1].1.min(rest[1].1) - 10, "the cap flies up through the air: {mid:?}");
        let xs: Vec<i32> = (0..=20).map(|i| shell_pieces(HATCH_CAP * i as f32 / 20.0, s)[1].0).collect();
        assert!(xs.windows(2).all(|w| w[0] <= w[1]), "and never comes back: {xs:?}");
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
        assert_eq!(expression(&Moment { pet: &p, fx: None, ms: 0, shell: None, sub_ms: 0 }), Expr::Sleeping);
    }

    #[test]
    fn moods_follow_the_needs() {
        let face = |p: &Pet, fx: Option<(Fx, f32)>| expression(&Moment { pet: p, fx, ms: 0, shell: None, sub_ms: 0 });
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
        assert_eq!(face(&p, Some((Fx::Snack, 0.5))), Expr::Eating);
        assert_eq!(expression(&Moment { pet: &p, fx: None, ms: 350 * 8, shell: None, sub_ms: 0 }), Expr::Blink);
    }

    #[test]
    fn a_need_bubble_changes_the_room() {
        let clean = pet(Stage::Baby);
        let mut dirty = pet(Stage::Baby);
        dirty.urge = Some(0);
        assert_ne!(look(&clean, 0), look(&dirty, 0));
        let mut hungry = pet(Stage::Baby);
        hungry.hunger = 0;
        let bubble_white = |f: &Frame| (0..H).flat_map(|y| (0..W).map(move |x| (x, y))).filter(|&(x, y)| f.get(x, y) == Some(Pal::White)).count();
        assert!(bubble_white(&look(&hungry, 0)) > bubble_white(&look(&clean, 0)) + 50, "a hungry pet shows a bubble");
    }
}
