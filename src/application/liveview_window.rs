use cairo::Context;
use cairo::PdfSurface;
use glib::Continue;
use gtk::{Inhibit, WidgetExt};
use gtk::prelude::*;
use log::{debug, trace, warn};
use std::f64::consts::PI;

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

const WINDOW_SCALER: f64 = 2.0;

impl LiveViewWindow {
    pub fn new(host: &str, auth0_id: &str, session_token: &str) -> Self {
        let (receiver, socket) = data_socket(host.to_string(), auth0_id, session_token.to_string());

        /*     let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT_IDLE);

          std::thread::spawn(move  ||{

              let data = include_bytes!("../remarkable/format/example.bin");

              std::thread::sleep(Duration::from_secs(5));

              sender.send(data.to_vec());

              std::thread::sleep(Duration::from_secs(10));

              let data = include_bytes!("../remarkable/format/example1.bin");

              sender.send(data.to_vec());

          });
  */
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

        let surface = PdfSurface::new(DEVICE_WIDTH / WINDOW_SCALER, DEVICE_HEIGHT / WINDOW_SCALER, path)
            .expect("Failed to create PDF");

        let surface_clone = surface.clone();

        window.connect_destroy(move |_| {
            debug!("Destroying thread");
            surface_clone.finish();
            let _ = tx.send(());
        });

        tokio::spawn(async move {
            rx.recv();
            socket.join().expect("Couldn't join the socket");
        });

        window.show_all();

        draw_area.set_size_request((DEVICE_WIDTH / WINDOW_SCALER) as i32, (DEVICE_HEIGHT / WINDOW_SCALER) as i32);

        LiveViewWindow { receiver, draw_area, window, surface }
    }

    /**
     * shows the window
     */
    pub fn listen(self) {
        debug!("Listening for events");
        let draw = self.draw_area.clone();

        let surface = self.surface.clone();

        let receiver = self.receiver;

        let context = Context::new(&surface);
        context.set_source_rgb(1., 1., 1.);
        context.paint();

        receiver.attach(None, move |data| {
            debug!("Received data");

            let line = parse_binary_live_lines(data);

            match line {
                Ok(line) => {
                    trace!("Drawing {} points", line.points.len());

                    let points = line.points;
                    let (r, g, b) = line.color.as_rgb();

                    if !points.is_empty() {
                        context.save();
                        context.set_source_rgb(r, g, b);

                        trace!("Starting at: {:?}", points[0]);
                        trace!(
                            "Width: {:?} {}",
                            draw.get_property_width_request(),
                            draw.get_property_height_request()
                        );

                        for p in points {
                            context.set_line_width(p.width);
                            context.arc(p.x / WINDOW_SCALER, p.y / WINDOW_SCALER, p.width / 2.0 / WINDOW_SCALER, 0.0, 2.0 * PI);
                            context.fill();
                        }

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
        draw.connect_draw(move |_area, cx| {
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
