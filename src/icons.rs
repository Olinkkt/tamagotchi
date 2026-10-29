//! Small pixel icons: one per action button (also shown in the pet's speech bubble) and the
//! bits and pieces of the room's effects. Letters are palette colours, see `faces::colour`.

pub type Grid = &'static [&'static str];

pub const FEED: Grid = &[
    "......kk....",
    ".....kgk....",
    "...kkknkkk..",
    "..krrrnrrrk.",
    ".krwrrrrrrrk",
    ".krwrrrrrrrk",
    ".krrrrrrrrrk",
    ".krrrrrrrrmk",
    ".krrrrrrrmmk",
    "..krrrrrmmk.",
    "...krrkrmk..",
    "....kk.kk...",
];

pub const SNACK: Grid = &[
    "............",
    "............",
    "............",
    "kk..kkkk..kk",
    "kpkkppbbkkpk",
    "kppkpppbkppk",
    "kppkvpppkppk",
    "kpkkvvppkkpk",
    "kk..kkkk..kk",
    "............",
    "............",
    "............",
];

pub const PLAY: Grid = &[
    "...kkkkkk...",
    "..krrwwssk..",
    ".krrrwwsssk.",
    "krrrrwwssssk",
    "krcrrwwssssk",
    "krrrrwwssssk",
    "krrrrwwssssk",
    "krrrrwwsssdk",
    ".krrrwwsssk.",
    "..krrwwsdk..",
    "...kkkkkk...",
    "............",
];

pub const LIGHT: Grid = &[
    "....kkkk....",
    "...kwcccck..",
    "..kwcccccyk.",
    "..kccccccyk.",
    "..kccccccyk.",
    "..kyccccyyk.",
    "...kyccyyk..",
    "....kyyk....",
    "....kaak....",
    "....khhk....",
    "....kaak....",
    ".....kk.....",
];

pub const MEDICINE: Grid = &[
    "............",
    "............",
    "............",
    "..kkkkkkkk..",
    ".kwwwwkbrrk.",
    "kwwwwwkrrrrk",
    "khwwwwkrrrmk",
    ".khhhhkmmmk.",
    "..kkkkkkkk..",
    "............",
    "............",
    "............",
];

pub const CLEAN: Grid = &[
    ".....kk.....",
    "....knnk....",
    "....knnk....",
    "....knnk....",
    "....knnk....",
    "...kkkkkk...",
    "..kyyyyyyk..",
    "..kyoyyoyk..",
    ".kyyoyyoyyk.",
    ".kyoyyoyyok.",
    "kyyoyyoyyoyk",
    "kkkkkkkkkkkk",
];

pub const SCOLD: Grid = &[
    "....kkkk....",
    "...krrrrk...",
    "...krrrrk...",
    "...krrrmk...",
    "....krrk....",
    "....krmk....",
    "....krmk....",
    ".....kk.....",
    "....kkkk....",
    "....krrk....",
    "....kmmk....",
    "....kkkk....",
];

pub const SHOP: Grid = &[
    "....kkkk....",
    "...k....k...",
    "...k....k...",
    ".kkkkkkkkkk.",
    ".kyooooooook",
    ".kyoookoook.",
    ".kyookyook..",
    ".kyoookoook.",
    ".kyooooooook",
    ".kyyooooouk.",
    ".kuuuuuuuuk.",
    ".kkkkkkkkkk.",
];

pub const AWARDS: Grid = &[
    "..kkkkkkkk..",
    "kkkycyyyykkk",
    "k.kycyyyyk.k",
    "k.kycyyyok.k",
    ".kkyyyyyokk.",
    "...kyyyok...",
    "....kyok....",
    ".....kk.....",
    "....kyyk....",
    "...kkkkkk...",
    "...knnnnk...",
    "...kkkkkk...",
];

pub const GRAVEYARD: Grid = &[
    "...kkkkkk...",
    "..kahhhhak..",
    ".kahhhhhhak.",
    ".kahhzhhhak.",
    ".kahzzzhhak.",
    ".kahhzhhhak.",
    ".kahhzhhhak.",
    ".kaahhhhaak.",
    ".kaaaaaaaak.",
    "kggggggggggk",
    "kfgfggfgggfk",
    ".kkkkkkkkkk.",
];

pub const QUIT: Grid = &[
    "..kkkkkkkk..",
    "..knnnnnnk..",
    "..knttttnk..",
    "..kntnntnk..",
    "..knttttnk..",
    "..knnnnnnk..",
    "..knnnnyyk..",
    "..knttttnk..",
    "..kntnntnk..",
    "..knttttnk..",
    "..knnnnnnk..",
    "kkkkkkkkkkkk",
];

pub const NEW_EGG: Grid = &[
    "....kkkk....",
    "...kwwwwk...",
    "..kwwwwwwk..",
    "..kwwewwwk..",
    ".kwwwwwwwwk.",
    ".kwwwwweewk.",
    ".kweewwwwhk.",
    ".kwwwwwwhhk.",
    "..kwwwwhhk..",
    "...kkkkkk...",
    "............",
    "............",
];

/// The icon for an action key, as used on the buttons and in the speech bubble.
pub fn for_key(key: char) -> Option<Grid> {
    Some(match key {
        'f' => FEED,
        's' => SNACK,
        'p' => PLAY,
        'l' => LIGHT,
        'm' => MEDICINE,
        'c' => CLEAN,
        'd' => SCOLD,
        'b' => SHOP,
        'a' => AWARDS,
        'g' => GRAVEYARD,
        'q' => QUIT,
        'n' => NEW_EGG,
        _ => return None,
    })
}

pub const HEART_FULL: Grid = &[".kk.kk.", "krwkrrk", "krrrrrk", ".krrrk.", "..krk..", "...k..."];
pub const HEART_EMPTY: Grid = &[".aa.aa.", "azzazza", "azzzzza", ".azzza.", "..aza..", "...a..."];
pub const COIN: Grid = &[".kkkk.", "kyccyk", "kycyyk", "kyyyok", "kyyook", ".kkkk."];

pub const Z: Grid = &["iiiii", "...i.", "..i..", ".i...", "iiiii"];
pub const SWEAT: Grid = &[".k..", "ksk.", "ksik", "kssk", ".kk."];
pub const BATTERY: Grid = &[
    "kkkkkkkkk.",
    "krrxxxxxkk",
    "krrxxxxxkk",
    "krrxxxxxkk",
    "kkkkkkkkk.",
];
pub const HEART: Grid = &[".kk.kk.", "kbpkppk", "kpppppk", ".kpppk.", "..kpk..", "...k..."];
pub const SPARKLE: Grid = &["..c..", "..c..", "ccwcc", "..c..", "..c.."];
pub const ANGER: Grid = &["rr.rr", "r...r", ".....", "r...r", "rr.rr"];
pub const APPLE: Grid = &["...ng..", ".kkkkk.", "krwrrrk", "krrrrrk", "krrrrmk", ".krrmk.", "..kkk.."];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faces::colour;

    #[test]
    fn icons_are_rectangular_and_use_palette_letters() {
        let keys = "fsplmcdbagqn".chars().map(|k| for_key(k).unwrap());
        for g in keys.chain([HEART_FULL, HEART_EMPTY, COIN, Z, SWEAT, BATTERY, HEART, SPARKLE, ANGER, APPLE]) {
            for row in g {
                assert_eq!(row.len(), g[0].len(), "{g:?}");
                assert!(row.chars().all(|c| c == '.' || colour(c).is_some()), "{row}");
            }
        }
        for k in "fsplmcdbagqn".chars() {
            let g = for_key(k).unwrap();
            assert_eq!((g.len(), g[0].len()), (12, 12), "button icons are 12x12: {k}");
        }
    }
}
