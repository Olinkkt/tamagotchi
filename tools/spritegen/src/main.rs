//! Builds the game's pixel art: `cargo art` (an alias for `cargo run -p spritegen --release`).
//!
//! Sprite sheets go to `assets/sprites/` (embedded by the game), and an 8x preview of every
//! sprite goes to `assets/preview/` for review.

#[path = "../../../src/palette.rs"]
#[allow(dead_code)]
mod palette;
#[path = "../../../src/faces.rs"]
#[allow(dead_code)]
mod faces;
#[path = "../../../src/cast.rs"]
#[allow(dead_code)]
mod cast;
mod chars;
mod img;
mod process;
mod room;
mod sketch;

use std::path::{Path, PathBuf};

use cast::Character;
use faces::Expr;
use img::Img;
use palette::Pal;

const PREVIEW_SCALE: usize = 8;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Each palette entry as a shadow | base | highlight strip, so the ramps can be judged.
fn palette_sheet() -> Img {
    let (cols, cell_w, cell_h) = (6, 13, 6);
    let mut sheet = Img::new(cols * cell_w + 1, Pal::ALL.len().div_ceil(cols) * cell_h + 1);
    for (i, p) in Pal::ALL.iter().enumerate() {
        let (x, y) = ((i % cols * cell_w + 1) as i32, (i / cols * cell_h + 1) as i32);
        sheet.rect(x, y, 3, 4, p.shadow());
        sheet.rect(x + 3, y, 6, 4, *p);
        sheet.rect(x + 9, y, 3, 4, p.highlight());
        sheet.rect(x, y + 4, 12, 1, p.dark());
    }
    sheet
}

/// Images side by side with a 1px gap.
fn strip(imgs: &[Img]) -> Img {
    let h = imgs.iter().map(|i| i.h).max().unwrap_or(0);
    let mut out = Img::new(imgs.iter().map(|i| i.w + 1).sum::<usize>().saturating_sub(1), h);
    let mut x = 0;
    for i in imgs {
        out.blit(i, x, 0);
        x += i.w as i32 + 1;
    }
    out
}

/// The character as the game draws it: parts behind, body, face, parts in front.
/// `frame` picks the frame of every animated part; `None` leaves the parts off.
fn posed(c: &Character, cells: &[Img], e: Expr, frame: Option<usize>) -> Img {
    let parts = |front: bool, out: &mut Img| {
        if let Some(f) = frame {
            for (i, part) in c.parts.iter().enumerate().filter(|(_, p)| p.front == front) {
                out.blit(&cells[c.part_cell(i, f.min(part.frames - 1))], 0, 0);
            }
        }
    };
    let mut out = Img::new(c.size, c.size);
    parts(false, &mut out);
    out.blit(&cells[0], 0, 0);
    if let Some(spot) = c.face {
        for (x, y, p) in faces::pixels(faces::grid(spot.style, e)) {
            out.set(spot.x + x, spot.y + y, Some(p));
        }
    }
    parts(true, &mut out);
    out
}

fn egg_stages(body: &Img) -> Vec<Img> {
    let mut out = vec![body.clone()];
    for cracks in &faces::EGG_CRACKS {
        let mut egg = body.clone();
        for (x, y, p) in faces::pixels(cracks) {
            egg.set(cast::EGG_CRACK_AT.0 + x, cast::EGG_CRACK_AT.1 + y, Some(p));
        }
        out.push(egg);
    }
    out
}

/// Writes the sheet and previews; returns the row of looks for the cast contact sheet.
fn build(c: &Character, sprites: &Path, preview: &Path) -> Img {
    let cells = chars::cells(c);
    assert_eq!(cells.len(), c.cells(), "{} has the wrong number of cells", c.name);
    let sheet = strip(&cells);
    // The game slices cells at a fixed pitch, so the sheet has no gaps.
    let mut packed = Img::new(c.size * cells.len(), c.size);
    for (i, cell) in cells.iter().enumerate() {
        packed.blit(cell, (i * c.size) as i32, 0);
    }
    img::save(&packed, &sprites.join(format!("{}.png", c.name)));
    img::save_preview(&sheet, &preview.join(format!("{}_sheet.png", c.name)), PREVIEW_SCALE);

    let looks: Vec<(String, Img)> = if c.face.is_some() {
        let mut looks: Vec<(String, Img)> = Expr::ALL.iter().map(|e| (e.name().to_string(), posed(c, &cells, *e, Some(1)))).collect();
        if !c.parts.is_empty() {
            looks.push(("bare".into(), posed(c, &cells, Expr::Idle, None)));
            looks.extend((0..3).map(|f| (format!("frame{f}"), posed(c, &cells, Expr::Idle, Some(f)))));
        }
        looks
    } else if c.name == "ghost" {
        let mut ghost = cells[0].clone();
        for (x, y, p) in faces::pixels(&faces::GHOST_FACE) {
            ghost.set(cast::GHOST_FACE_AT.0 + x, cast::GHOST_FACE_AT.1 + y, Some(p));
        }
        vec![("face".to_string(), ghost)]
    } else if c.name != "egg" {
        vec![("plain".to_string(), cells[0].clone())]
    } else {
        egg_stages(&cells[0]).into_iter().enumerate().map(|(i, img)| (format!("stage{i}"), img)).collect()
    };
    for (name, img) in &looks {
        img::save_preview(img, &preview.join(format!("{}_{name}.png", c.name)), PREVIEW_SCALE);
    }
    let all: Vec<Img> = looks.into_iter().map(|(_, i)| i).collect();
    let row = strip(&all);
    img::save_preview(&row, &preview.join(format!("{}_all.png", c.name)), PREVIEW_SCALE);
    img::save_preview(&row, &preview.join(format!("{}_all_1x.png", c.name)), 1);
    row
}

fn main() {
    let sprites = root().join("assets/sprites");
    let preview = root().join("assets/preview");
    img::save_preview(&palette_sheet(), &preview.join("palette.png"), PREVIEW_SCALE);
    let rows: Vec<Img> = cast::ALL.iter().map(|c| build(c, &sprites, &preview)).collect();
    let mut sheet = Img::new(rows.iter().map(|r| r.w).max().unwrap_or(0), rows.iter().map(|r| r.h + 2).sum());
    let mut y = 0;
    for r in &rows {
        sheet.blit(r, 0, y);
        y += r.h as i32 + 2;
    }
    img::save_preview(&sheet, &preview.join("cast.png"), 4);
    let baby = posed(&cast::BABY, &chars::cells(&cast::BABY), Expr::Happy, None);
    for (light, name) in [(room::Light::Day, "day"), (room::Light::Lamp, "lamp"), (room::Light::Dark, "dark")] {
        let r = room::room(light);
        img::save(&r, &sprites.join(format!("room_{name}.png")));
        img::save_preview(&r, &preview.join(format!("room_{name}.png")), 2);
        // The standard 160x116 room with the baby standing on the rug, to judge proportions.
        let (sx, sy) = ((cast::ROOM_MASTER.0 - cast::ROOM_STAGE.0) as i32 / 2, (cast::ROOM_MASTER.1 - cast::ROOM_STAGE.1) as i32);
        let mut scene = Img::new(cast::ROOM_STAGE.0, cast::ROOM_STAGE.1);
        scene.blit(&r, -sx, -sy);
        scene.blit(&baby, 80 - 16, 101 - 30);
        img::save_preview(&scene, &preview.join(format!("scene_{name}.png")), 4);
    }
    println!("sprites in {}, previews in {}", sprites.display(), preview.display());
}
