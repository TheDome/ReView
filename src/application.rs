use std::env::args;

use gio::prelude::*;
use log::{debug, info};

use crate::application::view::APPLICATION_IDENTIFIER;
use crate::config::config::Config;

mod application_config;
mod model;
mod view;

pub fn run() {
    gtk::init().expect("Failed to init GTK+ application");

    let application = gtk::Application::new(Some(APPLICATION_IDENTIFIER), Default::default());

    let application = match application {
        Ok(application) => application,
        Err(error) => {
            view::error::show_error("Failed to INIT", error.to_string().as_str());
            panic!("Failed to initialize GTK Application: {}", error);
        }
    };

    application.connect_activate(move |app| {
        info!("Application activated");
        let app = app.clone();
        let config = Config::default();

        let app_view = view::app_view::AppView::new();
        let app_model = model::app_model::AppModel::new(config);

        let app_controller = model::app_controller::AppController::new(app_model, app_view);

        debug!("Running AppController");
        app_controller.connect_application(&app);

        app_controller.run();
    });

    application.run(&args().collect::<Vec<_>>());
}
