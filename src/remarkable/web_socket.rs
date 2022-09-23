use futures_util::StreamExt;
use log::{debug, error, trace, warn};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http, Error, Message::Text},
    MaybeTlsStream, WebSocketStream,
};

use crate::remarkable::{constants::REMARKABLE_NOTIFICATION_SOCKET_PATH, BaseDomains};

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

pub async fn create_socket(
    url: &str,
    token: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
    debug!("Connecting to remarkable socket");
    trace!("Using base: {}", url);
    trace!("Token: {}", token);

    let token = token.to_string();

    let req = http::Request::builder().uri(url);

    let req = req.header("Authorization", format!("Bearer {}", token));

    let req = req.body(())?;

    let req = req.into_client_request()?;

    let res = connect_async(req).await?;

    debug!("Resonse was: {:?}", res.1);

    Ok(res.0)
}

pub async fn await_message(
    socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Result<SocketEvent, Error> {
    let msg = socket.next().await;

    match msg {
        Some(Ok(msg)) => {
            trace!("Received message: {:?}", msg);
            match msg {
                Text(text) => {
                    let text = text.as_str();
                    if text.starts_with("{\"type\":\"doc_added\"") {
                        let doc_added_msg = text.to_string();
                        return Ok(SocketEvent::DocAdded(doc_added_msg));
                    } else if text.starts_with("{\"type\":\"live_sync_started\"") {
                        let live_sync_started_msg = text.to_string();
                        return Ok(SocketEvent::LiveSyncStarted(
                            live_sync_started_msg,
                            token_from_msg(text),
                        ));
                    }
                }
                _ => {
                    warn!("Received unexpected message: {:?}", msg);
                }
            }
        }
        Some(Err(e)) => {
            error!("Error receiving message: {:?}", e);
        }
        None => {
            error!("Socket closed");
        }
    }
    Err(Error::AlreadyClosed)
}

fn token_from_msg(msg: &str) -> String {
    let json = json::parse(msg).unwrap();
    let session_token = json["session_token"].as_str().unwrap();
    session_token.to_string()
}

pub fn get_livesync_url(base: &BaseDomains) -> String {
    format!(
        "{}{}",
        base.notifications, REMARKABLE_NOTIFICATION_SOCKET_PATH
    )
}
