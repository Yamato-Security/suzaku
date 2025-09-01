use termcolor::Color;

pub enum SuzakuColor {
    Red,
    Orange,
    Green,
    Yellow,
    Cyan,
    White,
}

impl SuzakuColor {
    pub fn rdg(&self, no_color: bool) -> Option<Color> {
        if no_color {
            return None;
        }
        match self {
            SuzakuColor::Red => Some(Color::Rgb(255, 0, 0)),
            SuzakuColor::Orange => Some(Color::Rgb(255, 175, 0)),
            SuzakuColor::Yellow => Some(Color::Rgb(255, 255, 0)),
            SuzakuColor::Green => Some(Color::Rgb(0, 255, 0)),
            SuzakuColor::Cyan => Some(Color::Rgb(0, 255, 255)),
            SuzakuColor::White => None,
        }
    }
}

pub fn rgb(color: &Option<Color>) -> comfy_table::Color {
    match color {
        Some(Color::Rgb(255, 0, 0)) => comfy_table::Color::Rgb { r: 255, g: 0, b: 0 },
        Some(Color::Rgb(255, 175, 0)) => comfy_table::Color::Rgb {
            r: 255,
            g: 175,
            b: 0,
        },
        Some(Color::Rgb(255, 255, 0)) => comfy_table::Color::Rgb {
            r: 255,
            g: 255,
            b: 0,
        },
        Some(Color::Rgb(0, 255, 0)) => comfy_table::Color::Rgb { r: 0, g: 255, b: 0 },
        _ => comfy_table::Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
    }
}
