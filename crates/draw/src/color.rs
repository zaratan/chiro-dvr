#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

pub const PALETTE: [Rgb; 12] = [
    Rgb::new(255, 214, 10),
    Rgb::new(76, 201, 240),
    Rgb::new(255, 140, 26),
    Rgb::new(91, 227, 91),
    Rgb::new(255, 92, 203),
    Rgb::new(110, 168, 255),
    Rgb::new(200, 240, 60),
    Rgb::new(185, 140, 255),
    Rgb::new(255, 92, 92),
    Rgb::new(235, 235, 235),
    Rgb::new(255, 200, 140),
    Rgb::new(20, 170, 170),
];
pub const INK: Rgb = Rgb::new(0, 0, 0);
pub const PANEL: Rgb = Rgb::new(32, 32, 32);
pub const PANEL_TEXT: Rgb = Rgb::new(230, 230, 230);
pub const SEPARATOR: Rgb = Rgb::new(80, 80, 80);
pub const BOX: Rgb = Rgb::new(255, 0, 0);
pub const TRAIL: Rgb = Rgb::new(255, 200, 0);
