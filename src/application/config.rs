use std::env::home_dir;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf};


use glib::base64_decode;
use log::{debug, trace};



const CONFIG_FOLDER: &str = ".config";
const CONFIG_FILE_PATH: &str = "rmapi";
const CONFIG_FILE_NAME: &str = "rmapi.conf";
const CONFIG_PATH: [&str; 3] = [CONFIG_FOLDER, CONFIG_FILE_PATH, CONFIG_FILE_NAME];

#[cfg(test)]
mod test {
    use std::ffi::OsStr;
    use std::path::PathBuf;

    use crate::application::config::CONFIG_PATH;

    #[test]
    fn test_dir_ok() {
        let path: PathBuf = CONFIG_PATH.iter().collect();
        assert_eq!(path, OsStr::new(".config/rmapi/rmapi.conf"))
    }
}

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

    fn get_config_path() -> PathBuf {
        let mut home_path = home_dir().expect("Failed to load home dir!");
        let config_path: PathBuf = CONFIG_PATH.iter().collect();
        home_path.push(config_path.as_path());

        home_path
    }

    pub fn load(&mut self) {
        debug!("Loading config");

        let home_path = Config::get_config_path();

        trace!("Loading from {:?}", home_path);

        let config_file = File::open(home_path.as_path());

        match config_file {
            Ok(file) => {
                trace!("File opened!");

                let reader = BufReader::new(file);

                for line in reader.lines() {
                    if line.is_err() {
                        continue;
                    }

                    let line = line.unwrap();

                    match line.split(':').next() {
                        Some("devicetoken") => {
                            self.device_key = line.split(':').nth(1).map(|v| String::from(v.trim()));
                        }

                        #[cfg(any(feature = "session_from_config", debug_assertions))]
                        Some("usertoken") => {
                            self.session_key = line.split(':').nth(1).map(|v| String::from(v.trim()));
                        }
                        Some(v) => trace!("Failed to load {}", v),
                        _ => {}
                    };
                }
                debug!("config read");
                trace!("{:?}", self);
            }
            Err(e) => {
                debug!("File not found. Using stock config");
                trace!("{:?}", e);
            }
        }
    }

    pub fn auth0_id(&self) -> Option<String> {
        match &self.session_key {
            Some(key) => {
                let token = String::from(key.as_str());

                let pieces = token.split('.').collect::<Vec<&str>>();

                let data = pieces.get(1);

                debug!("Result: {:?}", data);

                match data {
                    Some(data) => {
                        let userdata = base64_decode(*data);
                        let userdata =
                            json::parse(String::from_utf8_lossy(userdata.as_slice()).as_ref());

                        match userdata {
                            Ok(data) => {
                                let profile = &data["auth0-profile"];
                                let profile = &profile["UserID"];

                                debug!("Using profile: {}", profile);
                                Some(profile.to_string())
                            }
                            Err(_) => None,
                        }
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    #[cfg(debug_assertions)]
    pub fn save(&self) {
        let home_path = Config::get_config_path();

        debug!("Saving config to file {:?}", home_path);

        trace!("usertoken: {:?}", self.session_key);
        trace!("devicetoken: {:?}", self.device_key);
    }

    #[cfg(not(debug_assertions))]
    pub fn save(&self) {
        let home_path = Config::get_config_path();

        debug!("Saving config to file {:?}", home_path);

        let file = File::open(home_path);
    }
}
