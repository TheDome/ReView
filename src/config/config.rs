use std::env::home_dir;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use glib::base64_decode;
use log::{debug, trace, warn};

#[derive(Debug)]
pub struct Config {
    pub device_key: Option<String>,
    pub session_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device_key: None,
            session_key: None,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }


    pub fn load(&mut self, data: &str) -> Result<(), String> {
        debug!("Loading config");

        for line in data.split("\n") {
            match line.split(':').next() {
                Some("devicetoken") => {
                    self.device_key = line.split(':').nth(1).map(|v| String::from(v.trim()));
                }

                #[cfg(any(feature = "session_from_config", debug_assertions))]
                Some("usertoken") => {
                    self.session_key = line.split(':').nth(1).map(|v| String::from(v.trim()));
                }
                Some(v) => debug!("Ignoring {}", v),
                _ => {}
            };
        }

        debug!("Loaded config is: {:?}", self);

        Ok(())
    }

    /// Extract the auth0 id from the user session token.
    /// Will return None of not token can be found or the token is
    /// invalid
    pub fn auth0_id(&self) -> Option<String> {
        if let Some(key) = &self.session_key {
            if let Some(main_part) = key.split(".").collect::<Vec<&str>>().get(1) {
                let user_data = String::from_utf8(base64_decode(main_part));

                if let Err(e) = user_data {
                    warn!("Failed to decode user data: {}", e);
                    return None;
                }

                let object = json::parse(user_data.unwrap().as_str());

                if let Ok(data) = object {
                    let profile = &data["auth0-profile"];
                    let profile = &profile["UserID"];

                    debug!("Using profile: {}", profile);
                    return Some(profile.to_string());
                }
            }
        }

        None
    }

    pub fn create_config_content(&self) -> Option<&str> {
        trace!("usertoken: {:?}", self.session_key);
        trace!("devicetoken: {:?}", self.device_key);


        let mut config_file = String::new();

        if let Some(key) = &self.session_key {
            config_file.push_str("usertoken: ");
            config_file.push_str(key.as_str());
            config_file.push_str("\n");
        } else {
            return None;
        }

        if let Some(key) = &self.device_key {
            config_file.push_str("devicetoken: ");
            config_file.push_str(key.as_str());
            config_file.push_str("\n");
        } else {
            return None;
        }


        Some(config_file.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_config_return_format() {
        let mut config = Config::new();

        config.device_key = Some("device_key".to_string());

        let config_file = config.create_config_content();

        assert_eq!(config_file.unwrap(), "device_key");
    }
}
