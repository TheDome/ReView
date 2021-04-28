#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::BufReader;
    use log::info;

    use crate::application::remarkable::format::data::PenColor::BLACK;
    use crate::application::remarkable::format::data::PenType;
    use crate::application::remarkable::format::linesdata::parse_binary_live_lines;

    #[test]
    fn test_example() {
        env_logger::try_init();

        info!("Starting test");

        let data = include_bytes!("example.bin");

        let mut reader = std::io::Cursor::new(data);

        let line = parse_binary_live_lines(data.to_vec());

        assert!(line.is_ok(), "Line expected to be parsed correctly: Got: {:?}", line);

        let line = line.unwrap();

        assert_eq!(line.brush, PenType::TiltPencil);
        assert_eq!(line.color, BLACK);
        assert_eq!(line.points.len(), 81);
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
    }

    #[test]
    fn test_example_2() {
        env_logger::try_init();

        let data = include_bytes!("example2.bin");

        let line = parse_binary_live_lines(data.to_vec());

        assert!(line.is_ok());

        let line = line.unwrap();

        assert_eq!(line.points.len(), 169);
        assert_eq!(line.brush, PenType::SharpPencil);
        assert_eq!(line.color, BLACK);
    }
}
