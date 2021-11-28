use base64::decode;
use log::{debug, trace};

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
            debug!("Extracting auth0 id from session key");
            trace!("Session key is: {}", key);
            if let Some(main_part) = key.split(".").collect::<Vec<&str>>().get(1) {
                let decoded = decode(main_part);

                if let Err(e) = decoded {
                    debug!("Failed to decode session token: {}", e);
                    return None;
                }

                let user_data = String::from_utf8(decoded.unwrap());

                if let Err(e) = user_data {
                    debug!("Failed to decode user data: {}", e);
                    return None;
                }

                trace!("User data is: {}", user_data.as_ref().unwrap());

                let object = json::parse(user_data.as_ref().unwrap());

                trace!("User data is: {:?}", object);

                return if let Ok(data) = object {
                    let profile = &data["auth0-profile"];
                    let profile = &profile["UserID"];

                    if profile.to_string() == "null" {
                        return None;
                    }

                    debug!("Using profile: {}", profile);
                    Some(profile.to_string())
                } else {
                    debug!(
                        "Failed to parse user data: {} -> {:?}",
                        object.unwrap_err(),
                        user_data.as_ref().unwrap()
                    );
                    None
                };
            }
            debug!("Failed to extract main part");
        }

        None
    }

    pub fn create_config_content(&self) -> Option<String> {
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

        Some(config_file)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_config_return_format() {
        let mut config = Config {
            device_key: Some(String::from("device_key")),
            session_key: Some(String::from("session_key")),
        };

        let config_file = config.create_config_content();

        assert_eq!(
            config_file.unwrap(),
            "usertoken: session_key\ndevicetoken: device_key\n"
        );
    }

    #[test]
    fn get_auth0_id_not_given() {
        let mut config = Config::default();
        // This key does not contain an auth0 id
        config.session_key = Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".into());

        assert_eq!(config.auth0_id(), None);
    }

    #[test]
    fn get_auth0_id() {
        let mut config = Config {
            session_key: Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJhdXRoMC1wcm9maWxlIjp7IlVzZXJJRCI6InRlc3QifX0.\
        2Bmk995Tp6wp_8j2HsGtaEXxDyz3GTUh4EGfAemTHA0".into()),
            device_key: None,
        };

        assert_eq!(config.auth0_id(), Some("test".into()));
    }

    #[test]
    fn get_auth0_id_no_b64() {
        let mut config = Config {
            session_key: Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJhdXRoMC1wcm9maWxlIjp7IlVzZXJJRCI6InRlc3QifX5.\
        2Bmk995Tp6wp_8j2HsGtaEXxDyz3GTUh4EGfAemTHA0".into()),
            device_key: None,
        };

        assert_eq!(config.auth0_id(), None);
    }

    #[test]
    fn test_load_config() {
        let mut config = Config::default();
        let res = config.load("usertoken: session_key\ndevicetoken: device_key\n");

        assert_eq!(res, Ok(()));

        assert_eq!(config.session_key, Some("session_key".into()));
        assert_eq!(config.device_key, Some("device_key".into()));
    }
}
