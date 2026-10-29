//! `tama --gallery`: a PNG contact sheet of every form in every expression, drawn without a
//! window. Animated parts appear when `animate_parts` is on, as in the game.

use std::path::Path;

use crate::app::FORMS;
use crate::art;
use crate::cast;
use crate::config::Config;
use crate::faces::Expr;
use crate::frame::Frame;
use crate::palette::Pal;
use crate::pet::{Pet, Stage};
use crate::scene;

const CELL: i32 = 52;
const NAMES: i32 = 84;
const TOP: i32 = 14;
/// Output pixels per sheet pixel.
const SCALE: usize = 3;

fn pet(stage: Stage, species: &str, cfg: &Config) -> Pet {
    let mut p = Pet::new(1);
    p.stage = stage;
    p.species = species.into();
    p.set_env(cfg.clone(), 0);
    p
}

/// The sprites for one row of the sheet.
fn row(stage: Stage, species: &str, cfg: &Config) -> Vec<art::Sprite> {
    match stage {
        Stage::Egg => [3, 20, 22, 24, 25, 26, 28]
            .map(|age| {
                let mut p = pet(stage, species, cfg);
                p.stage_age = age;
                scene::posed(&p, Expr::Idle)
            })
            .to_vec(),
        Stage::Dead => vec![art::body(&cast::GRAVE).clone(), art::body(&cast::GHOST).clone()],
        _ => Expr::ALL.iter().map(|e| scene::posed(&pet(stage, species, cfg), *e)).collect(),
    }
}

pub fn sheet(cfg: &Config) -> Frame {
    let mut f = Frame::new(NAMES + CELL * Expr::ALL.len() as i32, TOP + CELL * FORMS.len() as i32, Pal::Night);
    for (i, e) in Expr::ALL.iter().enumerate() {
        f.text(e.name(), NAMES + i as i32 * CELL + 2, 3, 1, Pal::Mist);
    }
    for (r, (stage, species)) in FORMS.iter().enumerate() {
        let y = TOP + r as i32 * CELL;
        f.rect(0, y, f.w, 1, Pal::Dusk);
        f.text(species, 4, y + CELL / 2 - 4, 1, Pal::White);
        for (i, s) in row(*stage, species, cfg).iter().enumerate() {
            let x = NAMES + i as i32 * CELL + (CELL - s.w) / 2;
            f.sprite(s, x, y + CELL - 2 - s.bottom());
        }
        if *stage == Stage::Dead {
            let ghost = NAMES + CELL + (CELL - 24) / 2;
            f.grid(&crate::faces::GHOST_FACE, ghost + cast::GHOST_FACE_AT.0, y + CELL - 2 - art::body(&cast::GHOST).bottom() + cast::GHOST_FACE_AT.1);
        }
    }
    f
}

/// Write the sheet, every pixel blown up to a `SCALE` x `SCALE` square.
pub fn write(path: &Path, cfg: &Config) -> Result<(), String> {
    let f = sheet(cfg);
    let (w, h) = (f.w as usize * SCALE, f.h as usize * SCALE);
    let small = f.rgba();
    let mut big = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            let i = ((y / SCALE) * f.w as usize + x / SCALE) * 4;
            big.extend_from_slice(&small[i..i + 4]);
        }
    }
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().and_then(|mut wr| wr.write_image_data(&big)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sheet_has_a_row_per_form_and_a_column_per_expression() {
        let f = sheet(&Config::default());
        assert_eq!((f.w, f.h), (NAMES + CELL * 7, TOP + CELL * 11));
        assert_eq!(row(Stage::Egg, "Egg", &Config::default()).len(), 7, "the egg shows every crack stage");
    }

    #[test]
    fn the_sheet_is_written_as_a_png() {
        let path = std::env::temp_dir().join(format!("tama-gallery-test-{}.png", std::process::id()));
        write(&path, &Config::default()).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(&bytes[1..4], b"PNG");
    }
}
