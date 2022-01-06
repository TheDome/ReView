use std::env::args;
use std::os::unix::prelude::OsStringExt;

use gio::prelude::*;
use log::{debug, info};
use tokio::runtime::Runtime;

use crate::application::view::APPLICATION_IDENTIFIER;
use crate::config::config::Config;
use crate::config::config_io::{load_config_from_file, resolve_config_path};

use crate::application::{model, view};

mod application;
mod config;
mod remarkable;

fn main() {
    env_logger::init();

    info!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

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
        let config_path = resolve_config_path();
        let config = match config_path {
            Ok(config_path) => load_config_from_file(
                String::from_utf8_lossy(config_path.into_os_string().into_vec().as_slice())
                    .as_ref(),
            )
            .unwrap_or(config),
            Err(error) => {
                view::error::show_error("Failed to load config", error.to_string().as_str());
                config
            }
        };

        let app_view = view::app_view::AppView::new();
        let app_model = model::app_model::AppModel::new(config);

        let mut app_controller =
            model::app_controller::AppController::new(Box::new(app_model), app_view);

        debug!("Running AppController");
        app_controller.connect_application(&app);

        app_controller.run();
    });

    application.run(&args().collect::<Vec<_>>());
}
