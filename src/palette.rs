//! The 24 colours every pixel in the game is drawn with. The sprite tool (tools/spritegen)
//! includes this file too, so the art and the game can never disagree about a colour.
//!
//! Ramps are hue-shifted: a colour's shadow leans cooler (towards blue and purple) and its
//! highlight leans warmer (towards yellow), instead of just getting darker or lighter.

pub type Rgb = (u8, u8, u8);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Pal {
    Ink,
    Night,
    Dusk,
    Slate,
    Mist,
    White,
    Maroon,
    Red,
    Pink,
    Blush,
    Violet,
    Rust,
    Orange,
    Yellow,
    Cream,
    Forest,
    Green,
    Lime,
    DeepTeal,
    Teal,
    Sky,
    Ice,
    Brown,
    Tan,
}

use Pal::*;

impl Pal {
    pub const ALL: [Pal; 24] = [
        Ink, Night, Dusk, Slate, Mist, White, Maroon, Red, Pink, Blush, Violet, Rust, Orange, Yellow, Cream, Forest, Green,
        Lime, DeepTeal, Teal, Sky, Ice, Brown, Tan,
    ];

    pub fn rgb(self) -> Rgb {
        match self {
            Ink => (0x1b, 0x15, 0x26),
            Night => (0x2a, 0x24, 0x40),
            Dusk => (0x46, 0x3f, 0x6b),
            Slate => (0x6e, 0x6a, 0x9a),
            Mist => (0xa8, 0xa9, 0xcc),
            White => (0xf5, 0xf1, 0xe8),
            Maroon => (0x6b, 0x24, 0x40),
            Red => (0xd9, 0x43, 0x4f),
            Pink => (0xf0, 0x8c, 0xb0),
            Blush => (0xff, 0xca, 0xd5),
            Violet => (0x93, 0x63, 0xc4),
            Rust => (0x9e, 0x4a, 0x2c),
            Orange => (0xef, 0x8a, 0x3c),
            Yellow => (0xf6, 0xc9, 0x45),
            Cream => (0xff, 0xf1, 0xb0),
            Forest => (0x2c, 0x66, 0x50),
            Green => (0x5c, 0xb8, 0x5c),
            Lime => (0xb5, 0xe4, 0x6b),
            DeepTeal => (0x1e, 0x54, 0x70),
            Teal => (0x3d, 0xb3, 0xaa),
            Sky => (0x66, 0xa3, 0xee),
            Ice => (0xc5, 0xed, 0xf0),
            Brown => (0x7d, 0x52, 0x36),
            Tan => (0xc0, 0x8e, 0x5e),
        }
    }

    /// One step into shadow, leaning cooler.
    pub fn shadow(self) -> Pal {
        match self {
            Ink | Night => Ink,
            Dusk => Night,
            Slate => Dusk,
            Mist => Slate,
            White => Mist,
            Maroon => Night,
            Red => Maroon,
            Pink => Violet,
            Blush => Pink,
            Violet => Dusk,
            Rust => Maroon,
            Orange => Rust,
            Yellow => Orange,
            Cream => Yellow,
            Forest => DeepTeal,
            Green => Forest,
            Lime => Green,
            DeepTeal => Night,
            Teal => DeepTeal,
            Sky => Dusk,
            Ice => Sky,
            Brown => Maroon,
            Tan => Brown,
        }
    }

    /// One step into the light, leaning warmer.
    pub fn highlight(self) -> Pal {
        match self {
            Ink => Night,
            Night => Dusk,
            Dusk => Slate,
            Slate => Mist,
            Mist | White | Blush | Cream | Ice => White,
            Maroon => Red,
            Red | Rust => Orange,
            Pink => Blush,
            Violet => Pink,
            Orange => Yellow,
            Yellow => Cream,
            Forest => Green,
            Green => Lime,
            Lime => Cream,
            DeepTeal => Teal,
            Teal => Ice,
            Sky => Ice,
            Brown => Tan,
            Tan => Cream,
        }
    }

    /// The same colour in a dark room: two steps into shadow.
    pub fn dark(self) -> Pal {
        self.shadow().shadow()
    }

    /// The palette entry closest to `c` (by weighted RGB distance).
    pub fn nearest(c: Rgb) -> Pal {
        let dist = |p: Pal| {
            let (r, g, b) = p.rgb();
            let d = |a: u8, b: u8| (a as i32 - b as i32).pow(2);
            2 * d(r, c.0) + 4 * d(g, c.1) + 3 * d(b, c.2)
        };
        *Pal::ALL.iter().min_by_key(|p| dist(**p)).expect("palette is not empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_colour_snaps_to_itself_and_all_are_distinct() {
        for p in Pal::ALL {
            assert_eq!(Pal::nearest(p.rgb()), p, "{p:?}");
        }
    }

    #[test]
    fn ramps_never_leave_the_palette_and_shadows_get_darker() {
        let luma = |p: Pal| {
            let (r, g, b) = p.rgb();
            299 * r as u32 + 587 * g as u32 + 114 * b as u32
        };
        for p in Pal::ALL {
            assert!(luma(p.shadow()) <= luma(p), "{p:?} -> {:?}", p.shadow());
            assert!(luma(p.highlight()) >= luma(p), "{p:?} -> {:?}", p.highlight());
        }
    }
}
