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
