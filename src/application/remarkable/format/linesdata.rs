use std::io;
use std::io::Read;
use std::panic::catch_unwind;

use byteorder::{LittleEndian, ReadBytesExt};
use cairo::debug_reset_static_data;
use gdk::keys::constants::v;
use log::{debug, trace};

fn read_len(expected: u16, reader: &mut dyn io::Read) {
    let mut len = [0; 2];
    reader.read(&mut len);
    let len = (len[0] as u16) | (len[1] as u16) << 8;

    assert_eq!(len, expected, "Length of string not found. Aborting");
}

pub fn parse_binary_live_lines(file: &mut dyn io::Read) {
    debug!("Parsing data");
    let reader = file;

    debug!("Truncating the first 24 (+id) Bytes");
    reader.read_exact(&mut [0; 30]);

    // Now read the ID
    // This is a 36 Byte UUID
    let mut id = [0; 36];
    reader.read_exact(&mut id);

    trace!("ID is: {:?}", String::from_utf8_lossy(&id));

    //TODO: Find meaning of bytes
    reader.read_exact(&mut [0; 14]);

    let layer = reader.read_u16::<LittleEndian>().unwrap();

    trace!("Layer: {}", layer);

    reader.read(&mut [0; 14]);

    let lines = reader.read_u32::<LittleEndian>().unwrap();

    trace!("Lines: {}", lines);

    // What does this do?
    reader.read(&mut [0; 12]);

    reader.read(&mut [0; 8]);

    let len = reader.read_u16::<LittleEndian>().unwrap();
    let mut brush = vec![0; len as usize];
    reader.read(&mut brush);

    trace!("Using brush: {:?}", String::from_utf8(brush));

    reader.read_u8();
    reader.read_u32::<LittleEndian>();

    reader.read(&mut [0; 9]);

    let color_len = reader.read_u16::<LittleEndian>().unwrap();
    let mut color = vec![0; color_len as usize];
    reader.read(&mut color);
    reader.read_u8();

    trace!("Color: {:?}", String::from_utf8(color));

    reader.read_u32::<LittleEndian>();

    reader.read(&mut [0; 9]);

    let points = reader.read_u16::<LittleEndian>().unwrap();
    trace!("Points: {}", points);

    reader.read(&mut [0; 25]);

    let mut counter = 0;

    debug!("Entering loop");
    loop {
        counter += 1;

        read_len(9, reader);

        reader.read(&mut [0; 10]); // Discard "direction" string + 1b

        let direction = reader.read_f64::<LittleEndian>().unwrap();
        reader.read(&mut [0; 20]); // Discard more
        trace!("Direction is: {:?}", direction);

        read_len(5, reader);

        reader.read(&mut [0; 6]); // Discard "speed" string + 1b

        let speed = reader.read_f64::<LittleEndian>().unwrap();
        reader.read_u32::<LittleEndian>(); // Discard
        trace!("Speed is: {:?}", speed);

        // Width
        read_len(5, reader);
        reader.read(&mut [0; 6]);

        let width = reader.read_f64::<LittleEndian>().unwrap();
        trace!("Width is: {:?}", width);

        reader.read_u32::<LittleEndian>(); // Discard

        // X
        read_len(1, reader);
        reader.read_u16::<LittleEndian>();

        let x = reader.read_f64::<LittleEndian>().unwrap();
        trace!("X is: {:?}", x);

        reader.read_u32::<LittleEndian>();

        // Y
        read_len(1, reader);
        reader.read_u16::<LittleEndian>();

        let y = reader.read_f64::<LittleEndian>().unwrap();
        trace!("Y is: {:?}", x);

        reader.read_u32::<LittleEndian>();

        reader.read(&mut [0; 36]);

        debug!("Gathered line: ({},{}) in direction: {} with speed: {}", x, y, direction, speed);
    }

    debug!("Lines are finished. Footer incoming");
    trace!("Read {} points", counter);
}
