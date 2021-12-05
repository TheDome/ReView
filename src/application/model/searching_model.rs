use crate::config::config::Config;
use log::{debug, info};
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};

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

    pub fn search(self) {
        let config = &self.config;

        let device_key = config.device_key.clone();

        let mut rx = self.termination_receiver;

        let _search_task = tokio::spawn(async move {
            debug!("Searching for {:?}", device_key);

            loop {
                if rx.poll_recv().await.is_some() {
                    break;
                }
                debug!("Searching for {:?}", device_key);
            }
        });
    }

    pub fn get_termination_channel(&self) -> Sender<()> {
        self.termination_sender.clone()
    }
}
