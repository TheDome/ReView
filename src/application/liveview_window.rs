use std::f64::consts::PI;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use cairo::PdfSurface;
use cairo::{Context, Format, Surface, SurfaceType};
use gdk;
use gdk::WindowAttr;
use gio::dbus_address_escape_value;
use glib::{Continue, MainContext};
use gtk::prelude::*;
use gtk::{Inhibit, WidgetExt, Window};
use log::{debug, info, trace, warn};

use crate::application::application::WINDOWS_STRING;
use crate::remarkable::format::data::{DEVICE_HEIGHT, DEVICE_WIDTH};
use crate::remarkable::format::linesdata::parse_binary_live_lines;
use crate::remarkable::web_socket::data_socket;

pub struct LiveViewWindow {
    receiver: glib::Receiver<Vec<u8>>,
    draw_area: gtk::DrawingArea,
    window: gtk::Window,
    surface: cairo::PdfSurface,
}

impl LiveViewWindow {
    pub fn new(host: &str, auth0_id: &str, session_token: &str) -> Self {
        let (receiver, socket) = data_socket(host.to_string(), auth0_id, session_token.to_string());

        /*   let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT_IDLE);

        std::thread::spawn(move  ||{

            let data = include_bytes!("../remarkable/format/example.bin");

            std::thread::sleep(Duration::from_secs(5));

            sender.send(data.to_vec());

            std::thread::sleep(Duration::from_secs(10));

            let data = include_bytes!("../remarkable/format/example1.bin");

            sender.send(data.to_vec());

        });*/

        let builder = gtk::Builder::from_string(WINDOWS_STRING);

        let window: gtk::Window = builder
            .get_object("liveview_window")
            .expect("Failed to find liveview window");
        let draw_area: gtk::DrawingArea = builder
            .get_object("liveview_draw_area")
            .expect("Failed to find drawing area");

        let (tx, rx) = std::sync::mpsc::channel();

        let mut path = std::env::temp_dir();
        path.push(uuid::Uuid::new_v4().to_string());
        path.set_extension("pdf");

        let surface = PdfSurface::new(DEVICE_WIDTH as f64, DEVICE_HEIGHT as f64, path)
            .expect("Failed to create PDF");

        let surface_clone = surface.clone();

        window.connect_destroy(move |_| {
            debug!("Destroying thread");
            surface_clone.finish();
            let _ = tx.send(());
        });

        std::thread::spawn(move || {
            rx.recv();
            socket.join().expect("Couldn't join the socket");
        });

        window.show_all();

        draw_area.set_size_request(DEVICE_WIDTH as i32, DEVICE_HEIGHT as i32);

        LiveViewWindow {
            receiver,
            window,
            draw_area,
            surface,
        }
    }

    /**
     * shows the window
     */
    pub fn listen(self) {
        debug!("Listening for events");
        let draw = self.draw_area.clone();

        let surface = self.surface.clone();

        let receiver = self.receiver;

        receiver.attach(None, move |data| {
            debug!("Received data");

            let line = parse_binary_live_lines(data);

            let context = Context::new(&surface);
            context.set_source_rgb(1., 1., 1.);
            context.paint();

            match line {
                Ok(line) => {
                    trace!("Drawing {} points", line.points.len());

                    let points = line.points;

                    if points.len() > 0 {
                        context.save();
                        context.set_source_rgb(0., 0., 0.);

                        trace!("Starting at: {:?}", points[0]);
                        trace!(
                            "Width: {:?} {}",
                            draw.get_property_width_request(),
                            draw.get_property_height_request()
                        );

                        for p in points {
                            context.set_line_width(p.width);
                            context.arc(p.x, p.y, p.width, 0.0, 2.0 * PI)
                        }

                        context.fill_preserve();
                        context.restore();

                        context.stroke();
                    }

                    trace!("Finished");
                }
                Err(e) => {
                    warn!("Error while parsing lines data: {}", e);
                }
            };

            draw.queue_draw();

            Continue(true)
        });

        let draw = self.draw_area.clone();

        let surface = self.surface.clone();
        draw.connect_draw(move |area, cx| {
            trace!("Redrawing");

            cx.set_source_surface(&surface, 0.0, 0.0);
            cx.paint();
            cx.set_source_rgba(1., 1., 1., 1.);

            Inhibit(false)
        });

        let context = Context::new(&self.surface);
        context.set_source_rgb(1., 1., 1.);
        context.paint();

        draw.queue_draw();
    }
}
