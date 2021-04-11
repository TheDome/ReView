use log::{debug, info};

mod application;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();

    info!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    application::application::run();
}
