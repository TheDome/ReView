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

impl From<&String> for PenType {
    fn from(identifier: &String) -> Self {
        match identifier.as_str() {
            "Pencilv2" => PenType::TiltPencil,
            "SharpPencilv2" => PenType::SharpPencil,
            _ => PenType::UNKNWON,
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
