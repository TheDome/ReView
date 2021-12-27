use log::info;

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

    application::run();
}
