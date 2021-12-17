pub mod app_view;
pub mod error;

/// The name of the app
pub const MAIN_WINDOW_NAME: &str = concat!(env!("CARGO_PKG_NAME"));

/// The system wide identifier
pub const APPLICATION_IDENTIFIER: &str = "com.github.thedome.review";

/// The version of the app
pub const APPLICATION_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+", env!("GIT_HASH"));
