//! The game window. The scene is drawn into a fixed 240x160 pixel frame, uploaded as one
//! texture and shown at the largest whole-number scale that fits, so every scene pixel is
//! the same size. Sharp UI is drawn on top at full window resolution.

use macroquad::prelude::{
    clear_background, draw_text, draw_texture_ex, get_char_pressed, is_key_pressed, is_quit_requested, next_frame,
    prevent_quit, screen_height, screen_width, vec2, Color, Conf, DrawTextureParams, FilterMode, Image, KeyCode,
    Texture2D, WHITE,
};

use crate::app::{App, Key};
use crate::sprites::{mix, Canvas, C};

pub const SCENE_W: usize = 240;
pub const SCENE_H: usize = 160;
const START_SCALE: i32 = 4;
const LETTERBOX: Color = Color::new(0.06, 0.06, 0.1, 1.0);

fn conf() -> Conf {
    Conf {
        window_title: "tama".into(),
        window_width: SCENE_W as i32 * START_SCALE,
        window_height: SCENE_H as i32 * START_SCALE,
        window_resizable: true,
        ..Default::default()
    }
}

/// Where the scene goes in the window: whole-number scale, centred, letterboxed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl Viewport {
    pub fn fit(win_w: f32, win_h: f32) -> Viewport {
        let scale = (win_w / SCENE_W as f32).min(win_h / SCENE_H as f32).floor().max(1.0);
        let x = ((win_w - SCENE_W as f32 * scale) / 2.0).floor().max(0.0);
        let y = ((win_h - SCENE_H as f32 * scale) / 2.0).floor().max(0.0);
        Viewport { x, y, scale }
    }
}

/// The keys pressed since the last frame, in the game's own terms.
fn keys_pressed() -> Vec<Key> {
    let mut keys = Vec::new();
    for (code, key) in [(KeyCode::Escape, Key::Esc), (KeyCode::Tab, Key::Tab), (KeyCode::Up, Key::Up), (KeyCode::Down, Key::Down)] {
        if is_key_pressed(code) {
            keys.push(key);
        }
    }
    while let Some(c) = get_char_pressed() {
        if !c.is_control() {
            keys.push(Key::Char(c));
        }
    }
    keys
}

/// Stand-in scene until the new pixel art lands: the room's wall and floor, with a 1-pixel
/// checker border that shows whether every scene pixel comes out the same size.
fn placeholder(t: u32) -> Canvas {
    let mut c = Canvas::sized(SCENE_W, SCENE_H);
    let floor_y = SCENE_H as i32 - 40;
    for y in 0..SCENE_H as i32 {
        let col: C = if y < floor_y { mix((34, 40, 74), (74, 58, 92), y as f32 / floor_y as f32) } else { (122, 88, 66) };
        c.rect(0, y, SCENE_W as i32 - 1, y, col);
    }
    for x in 0..SCENE_W as i32 {
        for y in [0, SCENE_H as i32 - 1] {
            if (x + y) % 2 == 0 {
                c.set(x, y, (245, 245, 255));
            }
        }
    }
    for y in 0..SCENE_H as i32 {
        for x in [0, SCENE_W as i32 - 1] {
            if (x + y) % 2 == 0 {
                c.set(x, y, (245, 245, 255));
            }
        }
    }
    let bob = (t % 2) as f32;
    c.ellipse(120.0, floor_y as f32 - 14.0 + bob, 14.0, 13.0, (255, 160, 190));
    c
}

pub fn run(app: App) {
    macroquad::Window::from_config(conf(), main_loop(app));
}

async fn main_loop(mut app: App) {
    prevent_quit();
    let mut frame = Image::gen_image_color(SCENE_W as u16, SCENE_H as u16, WHITE);
    let texture = Texture2D::from_image(&frame);
    texture.set_filter(FilterMode::Nearest);
    while !app.quit {
        if is_quit_requested() {
            break;
        }
        for k in keys_pressed() {
            app.on_key(k);
        }
        app.update();

        let t = (app.started.elapsed().as_millis() / 350) as u32;
        frame.bytes = placeholder(t).rgba((0, 0, 0));
        texture.update(&frame);

        clear_background(LETTERBOX);
        let v = Viewport::fit(screen_width(), screen_height());
        let size = vec2(SCENE_W as f32 * v.scale, SCENE_H as f32 * v.scale);
        draw_texture_ex(&texture, v.x, v.y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
        let note = format!("{} - window port in progress (x{} scale, q quits)", app.pet.species, v.scale);
        draw_text(&note, v.x + 8.0 * v.scale, v.y + 12.0 * v.scale, 8.0 * v.scale, WHITE);
        next_frame().await;
    }
    app.shutdown();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_takes_the_largest_whole_scale_that_fits_and_is_centred() {
        assert_eq!(Viewport::fit(960.0, 640.0), Viewport { x: 0.0, y: 0.0, scale: 4.0 });
        assert_eq!(Viewport::fit(1000.0, 700.0), Viewport { x: 20.0, y: 30.0, scale: 4.0 });
        assert_eq!(Viewport::fit(1920.0, 1080.0), Viewport { x: 240.0, y: 60.0, scale: 6.0 });
        assert_eq!(Viewport::fit(100.0, 50.0).scale, 1.0, "never smaller than 1x");
    }
}
