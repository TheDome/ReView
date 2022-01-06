use tokio::sync::mpsc::Sender;

use crate::config::{Config, Expirable, Identifiable, KeyStore, UnserializableConfig};

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
    fn start_search(&self) -> Result<(), String>;

    /// Updates the config with the given config
    fn update_config(&mut self, config: Box<dyn UnserializableConfig>);

    fn get_termination_channel(&self) -> Sender<()>;

    /// Refeshed the session token by obtaining a new token from the device_key.
    fn refresh_session_token(&mut self) -> Result<(), String>;
}

pub trait AppControllerable {}
