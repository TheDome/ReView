#[cfg(test)]
mod test {
    use crate::application::remarkable::format::linesdata::parse_binary_live_lines;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_input() {
        env_logger::init();

        let data = include_bytes!("example.bin");

        let mut reader = std::io::Cursor::new(data);

        parse_binary_live_lines(&mut reader);
    }
}
