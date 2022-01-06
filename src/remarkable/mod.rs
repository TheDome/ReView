use crate::remarkable::constants::{PROTOCOL, REMARKABLE_SESSION_BASE_API};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct BaseDomains {
    pub notifications: String,
    pub storage: String,
    pub livesync: String,
    pub sessions: String,
}

impl Default for BaseDomains {
    fn default() -> Self {
        let session_base = PROTOCOL.to_string() + &REMARKABLE_SESSION_BASE_API;
        BaseDomains {
            notifications: "https://notifications.remarkable.com".to_string(),
            storage: "https://storage.remarkable.com".to_string(),
            livesync: "https://livesync.remarkable.com".to_string(),
            sessions: session_base,
        }
    }
}

mod constants;

pub mod format;

pub mod tokens;
pub mod web_socket;

#[async_trait]
pub trait RMTokenInterface {
    /// Queries a new session token from the remarkable API.
    /// This token will be refreshed using the current user token
    async fn create_session_token(&self, user_token: &str) -> Result<String, String>;

    /// Queries a new user token from the remarkable API.
    /// This token needs a special OTP obtained from the remearkable service
    async fn login(&self, otp: &str) -> Result<String, String>;

    /// Checks if a session is still considered valid by performing a request to the remarkable API.
    async fn session_okay(&self, session_token: &str) -> bool;
}
