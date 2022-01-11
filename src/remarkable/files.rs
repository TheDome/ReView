use gtk::targets_include_rich_text;
use log::{debug, info, trace};
use std::borrow::Borrow;

use crate::remarkable::constants::{
    PROTOCOL, REMARKABLE_STORAGE_DISCOVERY_PATH, REMARKABLE_STORAGE_PATH,
};
use crate::remarkable::remarkable_tree;
use crate::remarkable::remarkable_tree::{File, Item};

pub struct APIConnection<'a> {
    pub storage_url: &'a str,
    pub session_key: &'a str,
}

/**
 * transforms a remalable internal ID into a string presentable to the user
 */
pub async fn info_for_item(
    storage_url: &str,
    user_token: &str,
    file_id: &str,
) -> Result<remarkable_tree::File<'static>, String> {
    debug!("Looking up file");
    trace!("Name: {}", file_id);
    trace!("Auth: {}", user_token);

    let mut url = PROTOCOL.to_string();
    url.push_str(storage_url);
    url.push_str(REMARKABLE_STORAGE_DISCOVERY_PATH);

    trace!("Url: {:?}", url);

    let client = reqwest::Client::new();
    let client = client
        .post(&url)
        .bearer_auth(user_token)
        .query(&[("doc", file_id)]);

    match client.send().await {
        Ok(d) => {
            let json = json::parse(d.text().await.unwrap().as_str()).expect("");

            let item = json.borrow()[0].borrow();

            let success = item["Success"].as_bool();
            let message = item["Message"].as_str();

            if message.is_some()
                && message.unwrap().ne("")
                && success.is_some()
                && success.unwrap() != true
            {
                return Err(message.unwrap().to_string());
            }

            let id = item["ID"].borrow().as_str();
            let name = item["VissibleName"].as_str();
            let page = item["CurrentPage"].as_u32();
            let version = item["Version"].as_u32();

            if id.is_none() || name.is_none() || page.is_none() || version.is_none() {
                return Err("Failed to parse".to_string());
            }

            let item: Item = Item {
                name: name.unwrap().to_string(),
                id: id.unwrap().to_string(),
                parent: None,
            };

            Ok(File {
                item,
                current_page: page.unwrap(),
                version: version.unwrap(),
            })
        }
        Err(e) => Err(e.to_string()),
    }
}

pub async fn get_file_name(connection: APIConnection<'_>, file_id: &str) {}

pub async fn download_blob(connection: APIConnection<'_>, file_id: &str) -> Result<String, String> {
    debug!("Looking up file");
    trace!("Name: {}", file_id);
    trace!("Auth: {}", connection.session_key);

    let mut url = PROTOCOL.to_string();
    url.push_str(connection.storage_url);
    url.push_str(REMARKABLE_STORAGE_PATH);

    trace!("Url: {:?}", url);

    let user_token = connection.session_key;

    let client = reqwest::Client::new();
    let client = client
        .post(&url)
        .bearer_auth(user_token)
        .query(&[("doc", file_id), ("withBlob", "true")]);

    let url = match client.send().await {
        Ok(d) => {
            let json = json::parse(d.text().await.unwrap().as_str()).expect("");

            let item = json.borrow()[0].borrow();

            let success = item["Success"].as_bool();
            let message = item["Message"].as_str();

            if message.is_some()
                && message.unwrap().ne("")
                && success.is_some()
                && success.unwrap() != true
            {
                return Err(message.unwrap().to_string());
            }

            let url = item["BlobURLGet"].borrow().to_string();

            Ok(url)
        }
        Err(e) => Err(e.to_string()),
    };

    if url.is_err() {
        return url;
    }
    let url = url.unwrap();

    // Download the file now
    debug!("Got url: {}", url);

    let client = reqwest::Client::new();
    let client = client.get(&url);

    match client.send().await {
        Ok(res) => {
            trace!("Got body: {:?}", res.bytes().await);

            Ok("TEST".to_string())
        }
        Err(e) => Err(e.to_string()),
    }
}
