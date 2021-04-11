use std::fs::read_to_string;
use std::io::Write;
use std::sync::mpsc::channel;

use cairo::{Context, Surface, SurfaceType};
use gdk;
use gdk::WindowAttr;
use gio::dbus_address_escape_value;
use glib::{Continue, MainContext};
use gtk::prelude::*;
use gtk::{Inhibit, WidgetExt, Window};
use log::{debug, info, trace, warn};

use crate::application::application::WINDOWS_STRING;
use crate::application::remarkable::format::linesdata::parse_binary_live_lines;
use crate::application::remarkable::web_socket::data_socket;

pub struct LiveViewWindow {
    receiver: glib::Receiver<Vec<u8>>,
    draw_area: gtk::DrawingArea,
    window: gtk::Window,
}

impl LiveViewWindow {
    pub fn new(host: &str, auth0_id: &str, session_token: &str) -> Self {
        let (receiver, socket) = data_socket(host.to_string(), auth0_id, session_token.to_string());

        let builder = gtk::Builder::from_string(WINDOWS_STRING);

        let window: gtk::Window = builder
            .get_object("liveview_window")
            .expect("Failed to find liveview window");
        let draw_area: gtk::DrawingArea = builder
            .get_object("liveview_draw_area")
            .expect("Failed to find drawing area");

        window.connect_destroy(move |_| {
            debug!("Destroying thread");
            socket.send(());
        });

        window.show_all();

        LiveViewWindow {
            receiver,
            window,
            draw_area,
        }
    }

    /**
     * shows the window
     */
    pub fn listen(self) {
        debug!("Listening for events");
        let draw = self.draw_area.clone();

        let receiver = self.receiver;

        receiver.attach(None, |ctx| {
            debug!("Received data");

            let mut file = std::fs::File::create("foo.bin").unwrap();

            #[cfg(debug_assertions)]
            file.write(ctx.as_slice());

            let mut cursor = std::io::Cursor::new(ctx);

            parse_binary_live_lines(&mut cursor);

            Continue(true)
        });

        let surface = gdk::Window::create_similar_surface(
            &gdk::Window::new(None, &gdk::WindowAttr::default()),
            cairo::Content::Color,
            50,
            50,
        )
        .unwrap();

        draw.connect_configure_event(|dr, ev| true);

        draw.connect_draw(move |area, cx| {
            cx.set_source_surface(&surface, 0.0, 0.0);

            Inhibit(false)
        });
    }
}
