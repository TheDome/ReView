use std::time::Duration;

pub mod config;
pub mod config_io;

pub trait UnserializableConfig: KeyStore + Identifiable + Expirable {}

pub trait Config: KeyStore + Identifiable + Expirable + Serializable {}

pub trait KeyStore {
    /// Returns the inner loaded device key as JWT
    fn get_device_key(&self) -> Result<String, String>;

    /// Returns the inner loaded session key as JWT
    fn get_session_key(&self) -> Result<String, String>;

    /// Stores the device key
    fn set_device_key(&mut self, key: String);

    /// Stored the session key
    fn set_session_key(&mut self, key: String);
}

pub trait Expirable {
    /// Determines the remeining time for a JWT to be valid
    fn get_expiry(&self) -> Result<Duration, String>;
}

pub trait Identifiable {
    fn get_session_id(&self) -> Result<String, String>;
}

pub trait Serializable {
    /// Serializes the Config object into a string.
    fn serialize(&self) -> Result<String, String>;
}
