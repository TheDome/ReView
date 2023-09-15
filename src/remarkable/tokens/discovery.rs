use log::{debug, trace, warn};

use crate::remarkable::{
    constants::{
        PROTOCOL, REMARKABLE_LIVESYNC_DISCOVERY_PARAMS, REMARKABLE_LIVESYNC_DISCOVERY_PATH,
        REMARKABLE_NOTIFICATION_DISCOVERY_PARAMS, REMARKABLE_NOTIFICATION_DISCOVERY_PATH,
        REMARKABLE_STORAGE_DISCOVERY_PARAMS, REMARKABLE_STORAGE_DISCOVERY_PATH, WS_PROTOCOL,
    },
    tokens::get_host,
    BaseDomains,
};

pub async fn discover_with_base(base: String) -> Result<BaseDomains, String> {
    debug!("Performing service discovery");
    trace!("Using base: {}", base);

    let storage_url = format!("{}{}", base, REMARKABLE_STORAGE_DISCOVERY_PATH);

    let notification_url = format!("{}{}", base, REMARKABLE_NOTIFICATION_DISCOVERY_PATH);

    let livesync_url = format!("{}{}", base, REMARKABLE_LIVESYNC_DISCOVERY_PATH);

    debug!(
        "Requesting url storage: {}, notfication: {}, livesync: {}",
        storage_url.to_string(),
        notification_url.to_string(),
        livesync_url.to_string()
    );

    let client = reqwest::Client::new();

    let storage_client = client.clone();
    let storage_builder = storage_client.get(&storage_url);
    let storage_response = storage_builder
        .query(&REMARKABLE_STORAGE_DISCOVERY_PARAMS)
        .send();

    let notification_client = client.clone();
    let notification_builder = notification_client.get(&notification_url);
    let notification_response = notification_builder
        .query(&REMARKABLE_NOTIFICATION_DISCOVERY_PARAMS)
        .send();

    let livesync_client = client.clone();
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
            let sthost = get_host(storage.text().await.unwrap());
            let lvhost = get_host(livesync.text().await.unwrap());
            let nthost = get_host(notification.text().await.unwrap());

            trace!("Got data: storage:{:?}", &sthost);
            trace!("Got data: notification:{:?}", &nthost);
            trace!("Got data: livesync:{:?}", &lvhost);

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

    debug!("Service discovery completed");
    trace!("Got data: {:?}", &extraction_result);

    let (sthost, lvhost, nthost) = extraction_result.unwrap();

    let result = BaseDomains {
        storage: PROTOCOL.to_owned() + &sthost.ok_or("No storage host")?,
        notifications: WS_PROTOCOL.to_owned() + &nthost.ok_or("No notification host")?,
        livesync: WS_PROTOCOL.to_owned() + &lvhost.unwrap_or("".into()),
        sessions: BaseDomains::default().sessions,
    };

    debug!("Returning  {:?}", &result);

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use httpmock::MockServer;

    use super::*;

    #[tokio::test]
    async fn test_discover() {
        env_logger::try_init();

        let mut host = HashMap::new();
        host.insert("Host".to_string(), "test".to_string());
        let json_res = json::stringify(host);

        let server = MockServer::start();
        let mocks = [
            REMARKABLE_STORAGE_DISCOVERY_PATH,
            REMARKABLE_NOTIFICATION_DISCOVERY_PATH,
            REMARKABLE_LIVESYNC_DISCOVERY_PATH,
        ]
        .map(|path| {
            server.mock(|when, then| {
                when.path(path);
                then.status(200)
                    .header("Content-Type", "application/json")
                    .body(json_res.clone());
            })
        });

        let result = super::discover_with_base(server.base_url()).await;

        assert!(result.is_ok());
        for mock in mocks {
            mock.assert_async().await;
        }
    }
}
