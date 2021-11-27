use std::io::{Error, ErrorKind};

use log::{debug, trace, warn};
use qt_json_rs::elements::{JsonBaseValue, JsonValue};
use qt_json_rs::QJSONDocument;

use crate::remarkable::format::data::PenColor::BLACK;
use crate::remarkable::format::data::PenType::TiltPencil;
use crate::remarkable::format::data::{Line, PenColor, PenType, Point};

struct LiveViewUpdate {
    page: u8,
    line: Line,
    layer: u8,
    id: String,
}

pub fn parse_binary_live_lines(data: Vec<u8>) -> Result<Line, std::io::Error> {
    let json = QJSONDocument::from_binary(data)?;

    debug!("Successfully parsed data");

    let base = match json.base {
        JsonBaseValue::Array(a) => {
            warn!("Did not expect an Array as JSON");
            trace!("{:?}", a);
            Err(Error::new(
                ErrorKind::InvalidData,
                "Did not expect an array",
            ))
        }
        JsonBaseValue::Object(o) => Ok(o),
    }?;

    let base_info = base.values;

    let lines = match base_info.get("lines") {
        Some(JsonValue::Object(a)) => Ok(a),
        Some(a) => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Expected an Object. Got: {:?}", a),
        )),
        None => Err(Error::new(
            ErrorKind::InvalidData,
            "Did not expect no lines",
        )),
    }?;

    let mut points = Vec::new();

    if let Some(JsonValue::Array(array_entries)) = lines.values.get("points") {
        for line in array_entries {
            let addable = match line {
                JsonValue::Object(o) => {
                    let vals = &(o).values;

                    let _direction = parse_to_number(vals.get("direction"))?;
                    let pressure = parse_to_number(vals.get("p"))?;
                    let speed = parse_to_number(vals.get("speed"))?;
                    let width = parse_to_number(vals.get("width"))?;
                    let x = parse_to_number(vals.get("x"))?;
                    let y = parse_to_number(vals.get("y"))?;

                    Ok(Point {
                        width: *width,
                        speed: *speed,
                        pressure: *pressure,
                        y: *y,
                        x: *x,
                    })
                }
                _ => Err(Error::new(
                    ErrorKind::InvalidData,
                    "Expected points to be an object",
                )),
            }?;
            points.push(addable);
        }
    } else {
        warn!("Could not parse points. Skipping");
    }

    let mut brush: Option<PenType> = None;

    if let Some(JsonValue::String(brush_type)) = lines.values.get("brush") {
        brush = Some(brush_type.into());
    }

    let mut color: Option<PenColor> = None;

    if let Some(JsonValue::String(color_type)) = lines.values.get("color") {
        color = Some(color_type.into());
    }

    Ok(Line {
        points,
        brush: brush.unwrap_or(TiltPencil),
        color: color.unwrap_or(BLACK),
    })
}

fn parse_to_number(val: Option<&JsonValue>) -> Result<&f64, Error> {
    match val.unwrap_or(&JsonValue::Number(0.0)) {
        JsonValue::Number(n) => Ok(n),
        v => Err(Error::new(
            ErrorKind::InvalidData,
            format!("Expected an f64. Got: {:?}", v),
        )),
    }
}

#[cfg(test)]
mod tests {
    use log::info;
    use num::abs;

    use super::*;

    #[test]
    fn test_example() {
        env_logger::try_init();

        info!("Starting test");

        let data = include_bytes!("example.bin");

        let _reader = std::io::Cursor::new(data);

        let line = parse_binary_live_lines(data.to_vec());

        assert!(
            line.is_ok(),
            "Line expected to be parsed correctly: Got: {:?}",
            line
        );

        let line = line.unwrap();

        assert_eq!(line.brush, PenType::TiltPencil);
        assert_eq!(line.color, BLACK);
        assert_eq!(line.points.len(), 81);

        let p0 = &line.points[0];

        let x = 1105.065673828125;
        let y = 55.36293411254883;

        assert!(abs(p0.x - x) < 0.05, "X should be {}, is {}", x, p0.x);
        assert!(abs(p0.y - y) < 0.05, "X should be {}, is {}", y, p0.y);
    }

    #[test]
    fn test_example_1() {
        env_logger::try_init();

        let data = include_bytes!("example1.bin");

        let line = parse_binary_live_lines(data.to_vec());

        assert!(line.is_ok());

        let line = line.unwrap();

        assert_eq!(line.points.len(), 169);
        assert_eq!(line.brush, PenType::SharpPencil);
        assert_eq!(line.color, BLACK);

        let p0 = &line.points[0];

        let x = 605.072021484375;
        let y = 482.8433837890625;

        assert!(abs(p0.x - x) < 0.05, "X should be {}, is {}", x, p0.x);
        assert!(abs(p0.y - y) < 0.05, "X should be {}, is {}", y, p0.y);
    }
}
