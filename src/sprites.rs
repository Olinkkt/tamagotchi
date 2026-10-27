use crate::pet::{Pet, Stage};

fn face(p: &Pet) -> &'static str {
    if !p.alive() {
        "x_x"
    } else if p.asleep {
        "-_-"
    } else if p.sick {
        "@_@"
    } else if p.hunger == 0 || p.happy == 0 {
        "T_T"
    } else if p.happy >= 3 {
        "^_^"
    } else {
        "o_o"
    }
}

fn ears(species: &str) -> &'static str {
    match species {
        "Mametchi" => " /\\_/\\ ",
        "Kuchipatchi" => " ,~~~, ",
        "Tarakotchi" => " \\|/|/ ",
        "Ichigotchi" => "  ~*~  ",
        _ => "  ~~~  ",
    }
}

/// Returns the sprite as lines; `frame` toggles a one-column bob.
pub fn sprite(p: &Pet, frame: bool) -> Vec<String> {
    let f = face(p);
    let lines: Vec<String> = match p.stage {
        Stage::Egg => vec![
            "  ___  ".into(),
            " /   \\ ".into(),
            "|  .  |".into(),
            " \\___/ ".into(),
        ],
        Stage::Baby => vec![
            "  .---.  ".into(),
            format!(" ( {f} ) "),
            "  '---'  ".into(),
        ],
        Stage::Child => vec![
            "   ___   ".into(),
            "  /   \\  ".into(),
            format!(" ( {f} ) "),
            "  \\___/  ".into(),
            "   | |   ".into(),
        ],
        Stage::Teen => vec![
            "  _/\\_  ".into(),
            " /    \\ ".into(),
            format!("| {f}  |"),
            " \\_  _/ ".into(),
            "  |__|  ".into(),
        ],
        Stage::Adult => vec![
            ears(&p.species).into(),
            format!("( {f} )"),
            " \\___/ ".into(),
            " /   \\ ".into(),
        ],
        Stage::Dead => vec![
            "  _____  ".into(),
            " /     \\ ".into(),
            " | R.I.P | ".into(),
            format!(" |  {f}  | "),
            " |_______| ".into(),
        ],
    };
    if frame && p.alive() && !p.asleep {
        lines.into_iter().map(|l| format!(" {l}")).collect()
    } else {
        lines
    }
}
