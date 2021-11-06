mod application_config;
mod config;
mod liveview_window;

mod ui;

pub mod application {
    
    use std::env::args;
    use std::sync::mpsc;
    use std::sync::mpsc::{channel, TryRecvError,RecvTimeoutError};
    use std::thread;
    

    use gio::prelude::*;
    use glib;
    
    use glib::{Continue, DateTime};
    use gtk::prelude::*;
    use gtk::{
        ApplicationWindow, Builder, ButtonsType, DialogFlags, MessageDialog, MessageType, Window,
    };
    use log::{debug, error, info, trace, warn};

    use crate::application::application_config::{APPLICATION_IDENTIFIER};
    use crate::application::config::Config;
    use crate::application::liveview_window::LiveViewWindow;
    use crate::application::ui::build_menu_bar;
    use crate::remarkable;
    use crate::remarkable::tokens::BaseDomains;
    use crate::remarkable::web_socket::SocketEvent;
    use crate::remarkable::web_socket::SocketEvent::LiveSyncStarted;
    use std::time::Duration;


    pub const WINDOWS_STRING: &str = include_str!("gui/Windows.glade");

    pub fn run() {
        gtk::init().expect("Failed to INIT gtk");
        debug!("Creating application");
        let app = gtk::Application::new(Some(APPLICATION_IDENTIFIER), Default::default())
            .expect("GTK INIT failed!");

        let builder = gtk::Builder::from_string(WINDOWS_STRING);

        info!("Loading API");
        let config = init_config();

        app.connect_activate(move |app| {
            debug!("Loaded");

            let service_directories = match discover_services() {
                Ok(domains) => domains,
                Err(e) => {
                    show_error(e);
                    return;
                }
            };

            let application_menu = build_menu_bar(&builder);
            app.set_menubar(Some(&application_menu));

            let window: ApplicationWindow = builder
                .get_object("app_window")
                .expect("Failed to find window");
            window.set_application(Some(app));

            let container: gtk::Box = builder
                .get_object("main_window_box")
                .expect("Failed to find main container");

            window.add(&container);
            window.show_all();

            let config = init_config();

            let session_token = match config.session_key {
                Some(key) => key,
                None => "".into(),
            };

            let device_token = match config.device_key {
                Some(key) => key,
                None => "".into(),
            };

            let rx = init_session(device_token.as_str(), session_token.as_str(), &builder);

            let nofifc_url = String::from(&service_directories.notifications);
            let livesync_host = String::from(&service_directories.livesync);

            let button_list: gtk::Box = builder
                .get_object("main_window_devices_box")
                .expect("Failed to find button box");

            let (lstx, lsrx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT_IDLE);

            // LiveViewWindow::new(String::from(&livesync_host).as_str(),"","").listen();

            tokio::spawn(async move {
                let not = nofifc_url;
                let live = livesync_host;

                match rx.recv() {
                    Ok((device_token, session_token)) => {
                        debug!("Tokens found!");
                        trace!("Device token: {}", device_token);
                        trace!("Session token: {}", session_token);

                        let socket = look_for_devices(not, session_token);
                        listen_socket_events(live, socket, lstx);
                    }
                    Err(e) => {
                        warn!("Error while receiving: {}", e);
                    }
                };
            });

            let host = service_directories.livesync;
            let list = button_list;

            lsrx.attach(None, move |(session_token, auth0_id)| {
                debug!("Creating new button");
                let host = String::from(&host);

                let btn = gtk::Button::with_label("New Client");

                btn.connect_clicked(move |btn| {
                    let window = LiveViewWindow::new(host.as_str(), &auth0_id, &session_token);
                    window.listen();
                    btn.hide();
                });

                list.add(&btn);

                btn.show_all();

                Continue(true)
            });
        });

        info!("Loaded");
        app.run(&args().collect::<Vec<_>>());

        config.save();
    }

    fn init_config() -> Config {
        debug!("Populating config");

        let mut config = Config::new();
        config.load();
        config
    }

    /**
     * Init data for the session. This includes:
     * - check if thes session token is okay
     * - if not generate one with the device token
     * - if not generate one with the OTP
     */
    fn init_session(
        device_token: &str,
        session_token: &str,
        builder: &Builder,
    ) -> std::sync::mpsc::Receiver<(String, String)> {
        let (tx, rx) = channel();

        let json = json::parse(session_token);

        let exp = match json {
            Ok(da) => da["exp"].as_number(),
            Err(_) => None,
        };

        let okay = match exp {
            Some(d) => {
                let exp = d.as_fixed_point_i64(0);

                let date = DateTime::from_unix_utc(exp.unwrap_or(0));
                let diff = date.expect("Failed to parse DateTime").difference(&DateTime::new_now_utc().expect("Failed to get current time"));

                debug!("Diff is : {}", diff);

                diff > 0
            }
            None => false,
        };

        if okay {
            debug!("Session token is Okay!");
            trace!("Using token {}", session_token);
            tx.send((String::from(device_token), String::from(session_token)));
        } else {
            let result = create_session_token(device_token).recv().unwrap();

            match result {
                Ok(token) => {
                    trace!("Using session token: {}", token);
                    tx.send((String::from(device_token), token));
                }
                Err(_) => {
                    let dev_token_r = create_login(builder);
                    thread::spawn(move || {
                        let dev_token = dev_token_r.recv().unwrap();
                        let session_token = create_session_token(&dev_token).recv().unwrap();

                        match session_token {
                            Ok(token) => {
                                debug!("Generated session token");
                                trace!("{}", token);
                                tx.send((dev_token, token));
                            }
                            Err(e) => {
                                error!("Failed to generate tokens: {}", e);
                                //TODO: Proper handeling
                            }
                        };
                    });
                }
            }
        }

        rx
    }

    /**
     * Probe, if the currently loaded config is okay. So if the current access tokens are correct. This will check the session token. If this one is okay, it will exit.
     */
    fn probe_session_token(
        storage_url: &str,
        session_key: &str,
    ) -> std::sync::mpsc::Receiver<bool> {
        debug!("Checking token validity");

        let (tx, rx) = channel();

        let storage_url = String::from(storage_url);
        let session_key = String::from(session_key);

        tokio::spawn(async move {
            let token_valid = remarkable::tokens::session_okay(&storage_url, &session_key).await;
            debug!("Token is valid: {}", token_valid);
            tx.send(token_valid);
        });
        rx
    }

    fn create_session_token(device_key: &str) -> std::sync::mpsc::Receiver<Result<String, String>> {
        debug!("Creating new session token");
        trace!("Device key is: {}", device_key);

        let (tx, rx) = channel();

        let token = String::from(device_key);

        tokio::spawn(async move {
            let token_res = remarkable::tokens::create_session_token(&token).await;

            let _ = tx.send(token_res);
        });

        rx
    }

    fn discover_services() -> Result<BaseDomains, String> {
        debug!("Discovering services");

        let (tx, rx) = std::sync::mpsc::channel();

        tokio::spawn(async move {
            let urls = remarkable::tokens::discover().await;

            tx.send(urls);
        });

        match rx.recv().expect("Error while sharing discovery") {
            Ok(d) => {
                debug!("Urls discovered: {:?}", &d);
                Ok(d)
            }
            Err(e) => {
                error!("Error at discovery of url: {}", e);
                Err(e)
            }
        }
    }

    /**
     * create a login window. This will log the user in
     * it uses the remarkable OTP API to create the tokens
     *
     * @returns A valid token
     */
    fn create_login(builder: &Builder) -> std::sync::mpsc::Receiver<String> {
        debug!("Creating login window");

        let (tx1, rx) = channel();

        let login_window: gtk::Window = builder
            .get_object("login_window")
            .expect("Failed to find login window");
        let login_field: gtk::Entry = builder
            .get_object("otp_input")
            .expect("Failed to find login window");
        let login_button: gtk::Button = builder
            .get_object("login_button")
            .expect("Failed to find login button");
        let login_status_label: gtk::Label = builder
            .get_object("login_status_label")
            .expect("Failed to find login label");

        login_button.connect_clicked(move |_| {
            login_status_label.set_text("Logging in");

            let text = String::from(login_field.get_text());

            let (tx, rx) = channel();

            tokio::spawn(async move {
                let result = remarkable::tokens::login(&text).await;
                let _ = tx.send(result);
            });

            let data = rx.recv().unwrap();

            match data {
                Ok(token) => {
                    debug!("Token found!");
                    trace!("{}", token);
                    tx1.send(token);
                }
                Err(e) => {
                    debug!("Login failed");
                    login_status_label.set_text(&format!("Error: {}", e));
                }
            }
        });

        login_window.show_all();

        rx
    }

    /**
     * This will perform a request to wait for devices to start the live view feature
     */
    fn look_for_devices(
        notification_url: String,
        session_key: String,
    ) -> mpsc::Receiver<SocketEvent> {
        let host = notification_url;
        let key = session_key;

        let (tx, rx) = mpsc::channel();


            let socket = remarkable::web_socket::start_socket(&host, &key);

        tokio::spawn(async move {
            loop {
                match socket.recv_timeout(Duration::from_secs(10)) {
                    Ok(ev) => {
                        trace!("Fetched event {:?}", ev);
                        let _ = tx.send(ev);
                    },Err(RecvTimeoutError::Disconnected) => {
                        break;
                    },
                    Err(RecvTimeoutError::Timeout) => {
                        break;
                    }
                };
            }
        });



        rx
    }

    fn listen_socket_events(
        livesync_url: String,
        rx: mpsc::Receiver<SocketEvent>,
        container: glib::Sender<(String, String)>,
    ) {
        let host = livesync_url;

        debug!("Listening for events..");

        std::thread::Builder::new()
            .name("event_listener".into())
            .spawn(move || loop {
                match rx.recv_timeout(Duration::from_secs(10)) {
                    Ok(ev) => {
                        debug!("{:?}", host);

                        let host = host.as_str();

                        match ev {
                            LiveSyncStarted(session, auth0_id) => {
                                debug!("Dispatching liveview");

                                let host = String::from(host);

                                trace!("Using host: {}", host);
                                trace!("Session: {}, auth0: {}", session, auth0_id);

                                container.send((session, auth0_id));
                            }
                            ev => {
                                trace!("Ignoring {:?}", ev);
                            }
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        break;
                    },
                    _=>{}
                };
                thread::sleep(Duration::from_millis(500));
            });
    }

    pub fn show_error(description: String) {
        glib::idle_add_local(move || {
            let diag = MessageDialog::new(
                None::<&Window>,
                DialogFlags::empty(),
                MessageType::Error,
                ButtonsType::Ok,
                &description,
            );

            diag.run();
            diag.hide();

            Continue(false)
        });
    }
}
