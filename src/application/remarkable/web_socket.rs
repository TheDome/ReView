use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use glib::Continue;
use log::{debug, error, info, trace, warn};
use websocket;
use websocket::futures::future::err;
use websocket::header::{Authorization, Bearer, Headers};
use websocket::message::OwnedMessage::{Binary, Text};
use websocket::result::WebSocketOtherError::StatusCodeError;
use websocket::websocket_base::result::WebSocketError::Other;
use websocket::ClientBuilder;

use crate::application::remarkable::config::{
    REMARKABLE_LIVEVIEW_SUBSCRIBER_PATH, REMARKABLE_NOTIFICATION_DISCOVERY_PATH,
    REMARKABLE_NOTIFICATION_SOCKET_PATH,
};
use crate::application::remarkable::web_socket::SocketEvent::LiveSyncStarted;

const PROTOCOL: &str = "wss://";

/**
* an event captured from the ReMarkable APIs.
*/
#[derive(Debug)]
pub enum SocketEvent {
    DocAdded(String),
    /**
     * Containing (session_token, auth0 uid)
     */
    LiveSyncStarted(String, String),
}

pub async fn start_socket(base_url: &str, session_token: &str) -> Receiver<SocketEvent> {
    debug!("Connecting to remarkable socket");
    trace!("Using base: {}", base_url);
    trace!("Token: {}", session_token);


    let (tx, rx) = mpsc::channel::<SocketEvent>();

    let mut url = PROTOCOL.to_string();
    url.push_str(base_url);
    url.push_str(REMARKABLE_NOTIFICATION_SOCKET_PATH);

    let token = session_token.to_string();
    let session_token = session_token.to_string();

    trace!("Using url: {:?}", url);

    let _ = std::thread::Builder::new()
        .name("SyncSocket".into())
        .spawn(move || {
            trace!("Task spawned");
            let mut headers = Headers::new();
            headers.set(Authorization(Bearer { token }));

            let mut client = ClientBuilder::new(&url)
                .unwrap()
                .custom_headers(&headers)
                .connect(None)
                .expect("Failed to connect to socket");

            trace!("Awaiting messages");

            loop {
                for incoming_message in client.incoming_messages() {
                    debug!("Received message");
                    let message = match incoming_message {
                        Ok(data) => data,
                        Err(e) => {
                            warn!("Error at websocket connection: {:?}", e);
                            continue;
                        }
                    };
                    let data = match message {
                        Text(d) => {
                            trace!("Received {}", d);
                            d
                        }
                        _ => {
                            warn!("Could not parse message {:?}", message);
                            continue;
                        }
                    };

                    debug!("Processing data");

                    let json = json::parse(data.as_ref());

                    let data = match json {
                        Ok(json) => json,
                        Err(e) => {
                            debug!("Error at parsing json: {:?}", e);
                            continue;
                        }
                    };

                    debug!("Received data!");
                    trace!("{:?}", &data);

                    let event = data["message"]["attributes"]["event"].as_str();

                    debug!("Received event: {:?}", event);

                    match event {
                        Some("DocAdded") => {
                            debug!("Document has been added!");
                        }
                        Some("LivesyncStarted") => {
                            debug!("Livesync request received!");

                            let document_id = data["message"]["attributes"]["id"].as_str();
                            let auth0_id = data["message"]["attributes"]["auth0UserID"].as_str();

                            let session_token = String::from(&session_token);
                            let auth0_id = match auth0_id {
                                Some(id) => Some(String::from(id)),
                                None => None,
                            };

                            match document_id {
                                Some(d) => {
                                    if auth0_id.is_some() {
                                        tx.send(LiveSyncStarted(session_token, auth0_id.unwrap()));
                                    }
                                }
                                None => {}
                            };
                        }
                        Some(v) => {
                            trace!("Not parsed event: {}", v);
                        }
                        None => {
                            warn!("No event found!");
                        }
                    };
                }
            }
        });
    trace!("Leaving start_socket");

    rx
}

pub fn data_socket(
    base: String,
    auth0_id: &str,
    session_token: String,
) -> (glib::Receiver<Vec<u8>>, std::sync::mpsc::Sender<()>) {
    debug!("Listening on new data socket");

    let (datatx, datarx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let (stoptx, stoprx) = std::sync::mpsc::channel();

    let mut path = PROTOCOL.to_string();
    path.push_str(base.as_str());
    path.push_str(REMARKABLE_LIVEVIEW_SUBSCRIBER_PATH); // maybe we need to concat the auth0 id

    trace!("Using url: {}", &path);

    let token = session_token.to_string();

    let th = std::thread::Builder::new()
        .name(format!("livesync {}", &auth0_id))
        .spawn(move || {
            trace!("Dispatched thread");

            let mut headers = Headers::new();
            headers.set(Authorization(Bearer { token }));

            let  client = ClientBuilder::new(&path)
                .unwrap()
                .custom_headers(&headers)
                .connect(None);

            let mut client = match client {
                Ok(c) => c,
                Err(Other(e)) => {
                    return;
                }
                _ => {
                    return;
                }
            };

            trace!("Awaiting livesync messages");

            for incoming_message in client.incoming_messages() {
                let message = match incoming_message {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Errow while receiving message! {}", e);
                        continue;
                    }
                };

                let data = match message {
                    Binary(data) => data,
                    e => {
                        trace!("Ignoring message: {:?}", e);
                        continue;
                    }
                };

                match stoprx.recv_timeout(Duration::from_millis(1)) {
                    Ok(_) => {
                        trace!("Stopping thread");
                        break;
                    }
                    _ => {}
                }

                let _ = datatx.send(data);
            }
        });

    (datarx, stoptx)
}
