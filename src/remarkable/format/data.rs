
use log::warn;

pub const DEVICE_WIDTH: f64 = 1404.0 ;
pub const DEVICE_HEIGHT: f64 = 1872.0;

#[derive(Debug, Eq, PartialEq)]
pub enum PenType {
    BallPoint,
    Marker,
    Fineliner,
    SharpPencil,
    TiltPencil,
    Brush,
    Highlighter,
    Eraser,
    EraseArea,
    EraseAll,
    Calligraphy,
    Pen,
    SelectionBrush,
    UNKNWON,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PenColor {
    BLACK,
    GRAY,
    WHITE,
}

impl From<&String> for PenColor {
    fn from(color: &String) -> Self {
        match color.as_str() {
            "Black" => PenColor::BLACK,
            "White" => PenColor::WHITE,
            _ => PenColor::GRAY,
        }
    }
}

impl PenColor {
    pub fn as_rgb(&self) -> (f64, f64, f64) {
        match self {
            PenColor::BLACK => (0.0, 0.0, 0.0),
            PenColor::WHITE => (1.0, 1.0, 1.0),
            PenColor::GRAY => (0.5, 0.5, 0.5)
        }
    }
}

impl From<&String> for PenType {
    fn from(identifier: &String) -> Self {
        match identifier.as_str() {
            "Pencilv2" => PenType::TiltPencil,
            "SharpPencilv2" => PenType::SharpPencil,
            e => {
                warn!("Could not identify type {}", e);
                PenType::UNKNWON
            },
        }
    }
}

#[derive(Debug)]
pub struct Point {
    pub speed: f64,
    pub width: f64,
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
}

#[derive(Debug)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl From<Point> for Vector2D {
    fn from(p: Point) -> Self {
        Vector2D { x: p.x, y: p.y }
    }
}

#[derive(Debug)]
pub struct Line {
    pub points: Vec<Point>,
    pub brush: PenType,
    pub color: PenColor,
}
