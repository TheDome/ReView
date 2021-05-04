#[cfg(test)]
mod test {
    use log::info;
    
    

    use crate::remarkable::format::data::PenColor::BLACK;
    use crate::remarkable::format::data::PenType;
    use crate::remarkable::format::linesdata::parse_binary_live_lines;
    
    use num::abs;

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
