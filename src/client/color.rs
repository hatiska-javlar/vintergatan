use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Color {
    LIME,
    GREEN,
    EMERALD,
    TEAL,
    CYAN,
    COBALT,
    INDIGO,
    VIOLET,
    PINK,
    MAGENTA,
    CRIMSON,
    RED,
    ORANGE,
    AMBER,
    YELLOW,
    BROWN,
    OLIVE,
    STEEL,
    MAUVE,
    SIENNA,
    NEUTRAL
}

pub struct ColorScheme {
    pub players_colors: Vec<Color>
}

impl ColorScheme {
    pub fn new() -> Self {
        ColorScheme {
            players_colors: vec![
                Color::GREEN,
                Color::CYAN,
                Color::INDIGO,
                Color::MAGENTA,
                Color::RED,
                Color::AMBER,
                Color::OLIVE,
                Color::SIENNA,
                Color::LIME,
                Color::EMERALD,
                Color::TEAL,
                Color::COBALT,
                Color::VIOLET,
                Color::PINK,
                Color::CRIMSON,
                Color::ORANGE,
                Color::YELLOW,
                Color::BROWN,
                Color::STEEL,
                Color::MAUVE
            ]
        }
    }

    pub fn get_color(color: Color) -> [f32; 4] {
        match color {
            Color::LIME => ColorScheme::RGB(164, 196, 0),
            Color::GREEN => ColorScheme::RGB(96, 169, 23),
            Color::EMERALD => ColorScheme::RGB(0, 138, 0),
            Color::TEAL => ColorScheme::RGB(0, 171, 169),
            Color::CYAN => ColorScheme::RGB(27, 161, 226),
            Color::COBALT => ColorScheme::RGB(0, 80, 239),
            Color::INDIGO => ColorScheme::RGB(106, 0, 255),
            Color::VIOLET => ColorScheme::RGB(170, 0, 255),
            Color::PINK => ColorScheme::RGB(244, 141, 208),
            Color::MAGENTA => ColorScheme::RGB(216, 0, 115),
            Color::CRIMSON => ColorScheme::RGB(162, 0, 37),
            Color::RED => ColorScheme::RGB(229, 20, 0),
            Color::ORANGE => ColorScheme::RGB(250, 104, 0),
            Color::AMBER => ColorScheme::RGB(240, 163, 10),
            Color::YELLOW => ColorScheme::RGB(227, 200, 0),
            Color::BROWN => ColorScheme::RGB(130, 90, 44),
            Color::OLIVE => ColorScheme::RGB(109, 135, 100),
            Color::STEEL => ColorScheme::RGB(100, 118, 135),
            Color::MAUVE => ColorScheme::RGB(118, 96, 138),
            Color::SIENNA => ColorScheme::RGB(160, 82, 45),
            Color::NEUTRAL => ColorScheme::RGB(255, 255, 255)
        }
    }

    fn RGB(r: u32, g: u32, b: u32) -> [f32; 4] {
        [r as f32 / 255_f32, g as f32 / 255_f32, b as f32 / 255_f32, 1_f32]
    }
}