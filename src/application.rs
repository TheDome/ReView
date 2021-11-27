use gdk::prelude::*;
use gtk::prelude::*;

use crate::config::config::Config;

mod application_config;
mod view;

pub fn run() {
    gtk::init();

    let config = Config::default();

    let application = gtk::Application::new(
        Some(application_config::APPLICATION_IDENTIFIER),
        Default::default(),
    );

    let application = match application {
        Ok(application) => application,
        Err(error) => {
            view::error::show_error("Failed to INIT", error.to_string().as_str());
            panic!("Failed to initialize GTK Application: {}", error);
        }
    };

    let app_window = view::ui::build_app_window();
}
