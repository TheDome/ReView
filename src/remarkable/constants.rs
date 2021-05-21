


#[cfg(other)]
pub const REMARKABLE_DEVICE_DESCRIPTION: &str = "browser-chrome";

#[cfg(target_os = "linux")]
pub const REMARKABLE_DEVICE_DESCRIPTION: &str = "desktop-linux";

#[cfg(target_os = "windows")]
pub const REMARKABLE_DEVICE_DESCRIPTION: &str = "desktop-windows";

#[cfg(target_os = "macos")]
pub const REMARKABLE_DEVICE_DESCRIPTION: &str = "desktop-macos";

const AUTH0_USER: &str = "auth0|5ff43c03c9f7b3013eeec9a7";

pub const PROTOCOL: &str = "https://";

pub const REMARKABLE_SESSION_BASE_API: &str = "webapp-production-dot-remarkable-production.appspot.com";
pub const REMARKABLE_SESSION_TOKEN_NEW: &str = "/token/json/2/user/new";
pub const REMARKABLE_SESSION_TOKEN_NEW_DEVICE: &str = "/token/json/2/device/new";

pub const REMARKABLE_SERVICE_BASE_API: &str =
    "service-manager-production-dot-remarkable-production.appspot.com";

pub const REMARKABLE_STORAGE_DISCOVERY_PATH: &str = "/service/json/1/document-storage";
pub const REMARKABLE_NOTIFICATION_DISCOVERY_PATH: &str = "/service/json/1/notifications";
pub const REMARKABLE_LIVESYNC_DISCOVERY_PATH: &str = "/service/json/1/livesync";

pub const REMARKABLE_NOTIFICATION_SOCKET_PATH: &str = "/notifications/ws/json/1";

pub const REMARKABLE_STORAGE_PATH: &str = "/document-storage/json/2/docs";

pub const REMARKABLE_STORAGE_DISCOVERY_PARAMS: [(&str, &str); 3] = [
    ("environment", "production"),
    ("group", AUTH0_USER),
    ("apiVer", "2"),
];
pub const REMARKABLE_LIVESYNC_DISCOVERY_PARAMS: [(&str, &str); 4] = [
    ("environment", "production"),
    ("group", AUTH0_USER),
    ("apiVer", "2"),
    ("role", "sub"),
];
pub const REMARKABLE_NOTIFICATION_DISCOVERY_PARAMS: [(&str, &str); 3] = [
    ("environment", "production"),
    ("group", AUTH0_USER),
    ("apiVer", "1"),
];

pub const REMARKABLE_LIVEVIEW_SUBSCRIBER_PATH: &str = concat!(
    "/livesync/ws/json/2/",
    "auth0|5ff43c03c9f7b3013eeec9a7",
    "/sub"
);
