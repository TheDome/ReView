use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::decode;
use json::parse;
use log::{debug, trace};

use crate::config::{Expirable, Identifiable, KeyStore, Serializable};

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

impl KeyStore for Config {
    fn get_device_key(&self) -> Result<String, String> {
        todo!()
    }

    fn get_session_key(&self) -> Result<String, String> {
        todo!()
    }
}

impl Config {
    pub(crate) fn deserialize(data: &str) -> Result<Self, String> {
        debug!("Loading config");

        let mut config = Config::default();

        for line in data.split("\n") {
            match line.split(':').next() {
                Some("devicetoken") => {
                    config.device_key = line.split(':').nth(1).map(|v| String::from(v.trim()));
                }

                #[cfg(any(feature = "session_from_config", debug_assertions))]
                Some("usertoken") => {
                    config.session_key = line.split(':').nth(1).map(|v| String::from(v.trim()));
                }
                Some(v) => debug!("Ignoring {}", v),
                _ => {}
            };
        }

        debug!("Loaded config is: {:?}", config);

        Ok(config)
    }
}

impl Serializable for Config {
    fn serialize(&self) -> Result<String, String> {
        trace!("usertoken: {:?}", self.session_key);
        trace!("devicetoken: {:?}", self.device_key);

        let mut config_file = String::new();

        if let Some(key) = &self.session_key {
            config_file.push_str("usertoken: ");
            config_file.push_str(key.as_str());
            config_file.push_str("\n");
        } else {
            return Err(String::from("No session key found"));
        }

        if let Some(key) = &self.device_key {
            config_file.push_str("devicetoken: ");
            config_file.push_str(key.as_str());
            config_file.push_str("\n");
        } else {
            return Err(String::from("No device key found"));
        }

        Ok(config_file)
    }
}

impl Identifiable for Config {
    /// Extract the auth0 id from the user session token.
    /// Will return None of not token can be found or the token is
    /// invalid
    fn get_session_id(&self) -> Result<String, String> {
        if let Some(key) = &self.session_key {
            debug!("Extracting auth0 id from session key");
            trace!("Session key is: {}", key);
            if let Some(main_part) = key.split(".").collect::<Vec<&str>>().get(1) {
                let decoded = decode(main_part).map_err(|e| e.to_string())?;

                let user_data = String::from_utf8(decoded).map_err(|e| e.to_string())?;

                trace!("User data is: {:?}", user_data);

                let object = json::parse(user_data.as_ref());

                trace!("User data is: {:?}", object);

                return if let Ok(data) = object {
                    let profile = &data["auth0-profile"];
                    let profile = &profile["UserID"];

                    if profile.to_string() == "null" {
                        return Err(String::from("No user id found"));
                    }

                    debug!("Using profile: {}", profile);
                    Ok(profile.to_string())
                } else {
                    debug!(
                        "Failed to parse user data: {} -> {:?}",
                        object.unwrap_err(),
                        user_data
                    );
                    return Err(String::from("Failed to parse user data"));
                };
            }
            debug!("Failed to extract main part");
        }

        Err(String::from("No session key found"))
    }
}

impl Expirable for Config {
    fn get_expiry(&self) -> Result<Duration, String> {
        let token = &self.session_key;
        if token.is_none() {
            return Err(String::from("No session key found"));
        }
        let token = token.as_ref().unwrap();
        let token = token.split(".").collect::<Vec<&str>>();
        let main_part = token.get(1);

        let main_part = match main_part {
            Some(v) => v,
            None => {
                debug!("Failed to extract main part from token");
                return Err(String::from("Failed to extract main part from token"));
            }
        };

        let decoded = decode(main_part);

        let decoded = match decoded {
            Ok(v) => v,
            Err(e) => {
                debug!("Failed to decode token: {}", e);
                return Err(String::from("Failed to decode token"));
            }
        };

        let json = parse(&String::from_utf8(decoded).unwrap());

        if let Ok(json) = json {
            let exp = json["exp"].as_u64();
            let exp = exp.unwrap();
            let now = SystemTime::now();
            let exp = now.checked_add(Duration::from_secs(exp));
            let exp = exp.unwrap();
            let exp = exp.duration_since(UNIX_EPOCH).unwrap();
            return Ok(exp);
        }

        Err(String::from("Failed to parse token"))
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
