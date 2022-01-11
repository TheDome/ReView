pub mod app_view;
pub mod error;
pub mod otp_view;

/// The name of the app
pub const MAIN_WINDOW_NAME: &str = concat!(env!("CARGO_PKG_NAME"));

/// The system wide identifier
pub const APPLICATION_IDENTIFIER: &str = "com.github.thedome.review";

/// The version of the app
pub const APPLICATION_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+", env!("GIT_HASH"));

/// The string to use for the current app window
pub const APP_WINDOWS_STRING: &str = include_str!("app_window.glade");
