use std::borrow::Borrow;
use std::collections::HashMap;
use std::env::join_paths;
use std::error::Error;

use log::{debug, info, trace, warn};
use reqwest::header::AUTHORIZATION;
use reqwest::{Client, Response, StatusCode};
use uuid::Uuid;

use crate::remarkable::constants::{
    PROTOCOL, REMARKABLE_DEVICE_DESCRIPTION, REMARKABLE_LIVESYNC_DISCOVERY_PARAMS,
    REMARKABLE_LIVESYNC_DISCOVERY_PATH, REMARKABLE_NOTIFICATION_DISCOVERY_PARAMS,
    REMARKABLE_NOTIFICATION_DISCOVERY_PATH, REMARKABLE_SERVICE_BASE_API,
    REMARKABLE_SESSION_BASE_API, REMARKABLE_SESSION_TOKEN_NEW, REMARKABLE_SESSION_TOKEN_NEW_DEVICE,
    REMARKABLE_STORAGE_DISCOVERY_PARAMS, REMARKABLE_STORAGE_DISCOVERY_PATH,
    REMARKABLE_STORAGE_PATH,
};

#[derive(Debug, Clone)]
pub struct BaseDomains {
    pub notifications: String,
    pub storage: String,
    pub livesync: String,
}

fn get_host(data: String) -> Option<String> {
    let json = json::parse(&data);

    match json {
        Ok(json) => match json["Host"].as_str() {
            Some(e) => Some(e.to_string()),
            _ => None,
        },
        Err(e) => {
            warn!("Failed to parse {}: {}", &data, e);
            None
        }
    }
}

pub async fn discover() -> Result<BaseDomains, String> {
    debug!("Performing service discovery");

    let mut storage_url = PROTOCOL.to_string();
    storage_url.push_str(REMARKABLE_SERVICE_BASE_API);
    storage_url.push_str(REMARKABLE_STORAGE_DISCOVERY_PATH);

    let mut notification_url = PROTOCOL.to_string();
    notification_url.push_str(REMARKABLE_SERVICE_BASE_API);
    notification_url.push_str(REMARKABLE_NOTIFICATION_DISCOVERY_PATH);

    let mut livesync_url = PROTOCOL.to_string();
    livesync_url.push_str(REMARKABLE_SERVICE_BASE_API);
    livesync_url.push_str(REMARKABLE_LIVESYNC_DISCOVERY_PATH);

    debug!(
        "Requesting url storage: {}, notfication: {}, livesync: {}",
        storage_url.to_string(),
        notification_url.to_string(),
        livesync_url.to_string()
    );

    let storage_client = Client::new();
    let storage_builder = storage_client.get(&storage_url);
    let storage_response = storage_builder
        .query(&REMARKABLE_STORAGE_DISCOVERY_PARAMS)
        .send();

    let notification_client = Client::new();
    let notification_builder = notification_client.get(&notification_url);
    let notification_response = notification_builder
        .query(&REMARKABLE_NOTIFICATION_DISCOVERY_PARAMS)
        .send();

    let livesync_client = Client::new();
    let livesync_res = livesync_client
        .get(&livesync_url)
        .query(&REMARKABLE_LIVESYNC_DISCOVERY_PARAMS)
        .send();

    let extraction_result = match (
        storage_response.await,
        notification_response.await,
        livesync_res.await,
    ) {
        (Ok(storage), Ok(notification), Ok(livesync)) => {
            trace!("Got data: storage:{:?}", storage);
            trace!("Got data: notification:{:?}", notification);
            trace!("Got data: livesync:{:?}", livesync);

            let sthost = get_host(storage.text().await.unwrap());
            let lvhost = get_host(livesync.text().await.unwrap());
            let nthost = get_host(notification.text().await.unwrap());

            Ok((sthost, lvhost, nthost))
        }
        e => {
            warn!("Error at loading remarkable servers");
            debug!("{:?}", e);
            Err("Error while connecting")
        }
    };

    if extraction_result.is_err() {
        return Err(extraction_result.err().unwrap().to_string());
    };

    let (sthost, lvhost, nthost) = extraction_result.unwrap();

    let result = BaseDomains {
        storage: sthost.unwrap(),
        notifications: nthost.unwrap(),
        livesync: lvhost.unwrap_or("".into()),
    };

    debug!("Returning  {:?}", &result);

    Ok(result)
}

/**
* Creates a new session token based on the user token
*/
pub async fn create_session_token(user_token: &str) -> Result<String, String> {
    debug!("Creating a new session token");
    trace!("Current user token: {}", user_token);

    let mut url = PROTOCOL.to_string();
    url.push_str(REMARKABLE_SESSION_BASE_API);
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

/**
* Creates a new token pair using the provided OTP
*/
pub async fn login(otp: &str) -> Result<String, String> {
    debug!("Logging in.");
    trace!("Using otp: {}", otp);

    let uuid = Uuid::new_v4().to_string();

    let mut url = PROTOCOL.to_string();
    url.push_str(REMARKABLE_SESSION_BASE_API);
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

pub async fn session_okay(storage: &str, session_token: &str) -> bool {
    debug!("Checking session validity");
    trace!("Using credentials: {}", session_token);

    let client = reqwest::Client::new();

    let mut url = PROTOCOL.to_string();
    url.push_str(storage);
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
