use crate::config::config::Config;

pub struct AppModel {
    config: Config,
}

impl AppModel {
    pub fn new(config: Config) -> Self {
        AppModel { config }
    }
}
