#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::BufReader;

    use crate::application::remarkable::format::data::PenColor::BLACK;
    use crate::application::remarkable::format::data::PenType;
    use crate::application::remarkable::format::linesdata::parse_binary_live_lines;

    #[test]
    fn test_input() {
        env_logger::init();

        let data = include_bytes!("example.bin");

        let mut reader = std::io::Cursor::new(data);

        let line = parse_binary_live_lines(&mut reader);

        assert_eq!(line.points.len(), 48);
        assert_eq!(line.brush, PenType::SharpPencil);
        assert_eq!(line.color, BLACK);
    }
}
