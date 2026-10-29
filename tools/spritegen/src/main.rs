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

/// The body with a face grid laid over it, as the game draws it.
fn with_face(c: &Character, body: &Img, e: Expr) -> Img {
    let mut out = body.clone();
    if let Some(spot) = c.face {
        for (x, y, p) in faces::pixels(faces::grid(spot.style, e)) {
            out.set(spot.x + x, spot.y + y, Some(p));
        }
    }
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

fn build(c: &Character, sprites: &Path, preview: &Path) {
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
        Expr::ALL.iter().map(|e| (e.name().to_string(), with_face(c, &cells[0], *e))).collect()
    } else {
        egg_stages(&cells[0]).into_iter().enumerate().map(|(i, img)| (format!("stage{i}"), img)).collect()
    };
    for (name, img) in &looks {
        img::save_preview(img, &preview.join(format!("{}_{name}.png", c.name)), PREVIEW_SCALE);
    }
    let all: Vec<Img> = looks.into_iter().map(|(_, i)| i).collect();
    img::save_preview(&strip(&all), &preview.join(format!("{}_all.png", c.name)), PREVIEW_SCALE);
    img::save_preview(&strip(&all), &preview.join(format!("{}_all_1x.png", c.name)), 1);
}

fn main() {
    let sprites = root().join("assets/sprites");
    let preview = root().join("assets/preview");
    img::save_preview(&palette_sheet(), &preview.join("palette.png"), PREVIEW_SCALE);
    for c in &cast::ALL {
        build(c, &sprites, &preview);
    }
    let baby = with_face(&cast::BABY, &chars::cells(&cast::BABY)[0], Expr::Happy);
    for (light, name) in [(room::Light::Day, "day"), (room::Light::Lamp, "lamp"), (room::Light::Dark, "dark")] {
        let r = room::room(light);
        img::save(&r, &sprites.join(format!("room_{name}.png")));
        img::save_preview(&r, &preview.join(format!("room_{name}.png")), PREVIEW_SCALE);
        // The baby standing on the rug, to judge proportions in place.
        let mut scene = r.clone();
        scene.blit(&baby, 80 - 16, 101 - 30);
        img::save_preview(&scene, &preview.join(format!("scene_{name}.png")), 4);
    }
    println!("sprites in {}, previews in {}", sprites.display(), preview.display());
}
