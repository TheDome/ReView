use std::{borrow::Borrow, collections::HashMap};

use async_trait::async_trait;
use log::{debug, trace, warn};
use reqwest::{Response, StatusCode};
use uuid::Uuid;
mod discovery;

use crate::remarkable::{
    constants::{
        PROTOCOL, REMARKABLE_DEVICE_DESCRIPTION, REMARKABLE_SERVICE_BASE_API,
        REMARKABLE_SESSION_TOKEN_NEW, REMARKABLE_SESSION_TOKEN_NEW_DEVICE, REMARKABLE_STORAGE_PATH,
    },
    tokens::discovery::discover_with_base,
    BaseDomains, RMTokenInterface,
};

pub struct RMTokens {
    base_domains: BaseDomains,
}

impl RMTokens {
    pub fn new(base_domains: BaseDomains) -> RMTokens {
        RMTokens { base_domains }
    }
}

pub async fn discover() -> Result<BaseDomains, String> {
    discover_with_base(format!("{}{}", PROTOCOL, REMARKABLE_SERVICE_BASE_API)).await
}

#[async_trait]
impl RMTokenInterface for RMTokens {
    async fn create_session_token(&self, user_token: &str) -> Result<String, String> {
        debug!("Creating a new session token");
        trace!("Current user token: {}", user_token);

        let mut url = self.base_domains.sessions.clone();
        url.push_str(REMARKABLE_SESSION_TOKEN_NEW);

        let client = reqwest::Client::new();
        let client = client
            .post(&url)
            .bearer_auth(user_token)
            .header("content-length", 0);

        match client.send().await {
            Ok(d) => request2result(d).await,
            Err(e) => Err(e.to_string()),
        }
    }

    async fn login(&self, otp: &str) -> Result<String, String> {
        debug!("Logging in.");
        trace!("Using otp: {}", otp);

        let uuid = Uuid::new_v4().to_string();

        let mut url = self.base_domains.sessions.clone();
        url.push_str(REMARKABLE_SESSION_TOKEN_NEW_DEVICE);

        let mut map = HashMap::new();
        map.insert("code", otp);
        map.insert("deviceDesc", REMARKABLE_DEVICE_DESCRIPTION);
        map.insert("deviceID", &uuid);

        let client = reqwest::Client::new().post(&url).bearer_auth("").json(&map);

        trace!("{:?}", client);

        match client.send().await {
            Ok(d) => request2result(d).await,
            Err(e) => Err(e.to_string()),
        }
    }

    async fn session_okay(&self, session_token: &str) -> bool {
        debug!("Checking session validity");
        trace!("Using credentials: {}", session_token);

        let client = reqwest::Client::new();

        let mut url = self.base_domains.storage.clone();
        url.push_str(REMARKABLE_STORAGE_PATH);

        let builder = client.get(&url).bearer_auth(session_token);

        trace!("{:?}", builder);

        match builder.send().await {
            Ok(d) => {
                trace!("Got result: {:?}", d);
                d.status() == StatusCode::OK
            }
            Err(e) => {
                warn!("Error while checking token: {}", e);
                false
            }
        }
    }
}

fn get_host(data: String) -> Option<String> {
    let json = json::parse(&data);

    match json {
        Ok(json) => json["Host"].as_str().map(|e| e.to_string()),
        Err(e) => {
            warn!("Failed to parse {}: {}", &data, e);
            None
        }
    }
}

/// Helper method to turn a reqwest to a Rust result by awaiting it.
async fn request2result(request: Response) -> Result<String, String> {
    match request.borrow().status() {
        StatusCode::OK => match request.text().await {
            Ok(e) => Ok(e),
            Err(e) => Err(e.to_string()),
        },
        _ => match request.text().await {
            Ok(e) => Err(e),
            Err(e) => Err(e.to_string()),
        },
    }
}

#[cfg(test)]
mod test {
    use httpmock;

    use super::*;

    #[tokio::test]
    async fn login() {
        let server = httpmock::MockServer::start();
        let domains = BaseDomains {
            notifications: server.base_url(),
            storage: server.base_url(),
            sessions: server.base_url(),
            livesync: server.base_url(),
        };

        server.mock(|when, then| {
            when.method("POST")
                .path(REMARKABLE_SESSION_TOKEN_NEW_DEVICE)
                .header("authorization", "Bearer");

            then.status(200)
                .header("Content-Type", "application/json")
                .body(json::stringify("OK"));
        });

        let token = RMTokens::new(domains);

        let res = token.login("1234").await;
        assert_eq!(res.is_ok(), true);
    }

    #[tokio::test]
    async fn session_token_new() {
        let server = httpmock::MockServer::start();
        let domains = BaseDomains {
            notifications: server.base_url(),
            storage: server.base_url(),
            sessions: server.base_url(),
            livesync: server.base_url(),
        };

        server.mock(|when, then| {
            when.method("POST")
                .path(REMARKABLE_SESSION_TOKEN_NEW)
                .header("Authorization", "Bearer 1234");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json::stringify("OK"));
        });

        let token = RMTokens::new(domains);

        let res = token.create_session_token("1234").await;
        assert_eq!(res.is_ok(), true);
    }

    #[tokio::test]
    async fn session_token_ok() {
        let server = httpmock::MockServer::start();
        let domains = BaseDomains {
            notifications: server.base_url(),
            storage: server.base_url(),
            sessions: server.base_url(),
            livesync: server.base_url(),
        };

        server.mock(|when, then| {
            when.method("GET")
                .path(REMARKABLE_STORAGE_PATH)
                .header("authorization", "Bearer 1234");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json::stringify("OK"));
        });

        let token = RMTokens::new(domains);

        let res = token.session_okay("1234").await;
        assert_eq!(res, true);
    }
}
