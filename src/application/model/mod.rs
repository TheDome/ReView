use tokio::sync::mpsc::Sender;

use crate::config::UnserializableConfig;

pub mod app_controller;
pub mod app_model;
pub mod liveview;

pub trait AppModelled {
    /// check if the currently loaded config is logged in.
    /// This check will be performed like:
    ///
    /// - check session token for validity
    /// - perform request with token
    fn is_logged_in(&mut self) -> bool;
    /// Starts the search for connections
    fn start_search(&mut self) -> Result<(), String>;

    /// Updates the config with the given config
    fn update_config(&mut self, config: Box<dyn UnserializableConfig>);

    fn get_termination_channel(&self) -> Sender<()>;

    /// Refeshed the session token by obtaining a new token from the device_key.
    /// @returns the new session token
    fn refresh_session_token(&mut self) -> Result<String, String>;

    /// Performa a user logn using the OTP provided from remarkable
    fn login_user(&mut self, otp: String) -> Result<(), String>;

    /// Attempts to find a valid session key to use.
    fn get_session_key(&mut self) -> Result<String, String>;
}

pub trait AppControllerable {}
