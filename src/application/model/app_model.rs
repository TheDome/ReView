use std::fmt::Error;
use std::task::Context;
use std::time::{Duration, UNIX_EPOCH};

use log::{debug, info, trace};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::futures;

use crate::config::config::Config;
use crate::config::Expirable;
use crate::remarkable::web_socket::{await_message, create_socket, get_livesync_url};

#[derive(Debug)]
pub struct AppModel {
    config: Config,
    termination_sender: Sender<()>,
    termination_receiver: Receiver<()>,
}

impl AppModel {
    pub fn new(config: Config) -> Self {
        let (termination_sender, termination_receiver) = channel(1);
        AppModel {
            config,
            termination_receiver,
            termination_sender,
        }
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn start_search(self) -> Result<(), String> {
        let config = &self.config;

        let device_key = config.device_key.clone();
        let token = config.session_key.clone();

        trace!("Session token is: {:?}", &token);

        let mut session_key;
        if let Some(device_token) = token {
            session_key = device_token;
        } else {
            return Err("No session key found".into());
        }

        let mut rx = self.termination_receiver;

        let _search_task = tokio::spawn(async move {
            debug!("Searching for {:?}", device_key);

            let url = get_livesync_url();
            let mut client = create_socket(&url, &session_key).await;

            if client.is_err() {
                return;
            }

            let mut client = client.unwrap();

            loop {
                let message = await_message(&mut client).await;
                debug!("Searching for {:?}", device_key);
            }
        });

        Ok(())
    }

    /// check if the currently loaded config is logged in.
    /// This check will be performed like:
    ///
    /// - check session token for validity
    /// - perform request with token
    pub async fn is_logged_in(&self) -> bool {
        debug!("app_model::is_logged_in()");
        // 1. get exp of session token
        let session_exp = &self.config.get_expiry();

        if session_exp.is_err() {
            debug!("No session token found");
            return false;
        }

        if session_exp.is_ok() && session_exp.as_ref().unwrap().as_secs() > 0 {
            debug!("Session token is valid");
            return true;
        }

        return false;
    }

    pub fn get_termination_channel(&self) -> Sender<()> {
        self.termination_sender.clone()
    }
}
