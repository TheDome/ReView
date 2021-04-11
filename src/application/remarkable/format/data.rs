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

pub fn get_pen(identifier: String) -> PenType {
    match identifier {
        _ => PenType::UNKNWON
    }
}