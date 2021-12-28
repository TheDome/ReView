use std::sync::mpsc::channel;

use futures_util::FutureExt;
use gio::prelude::*;
use glib::clone;
use gtk::{Application, DialogExt, EditableExt, GtkApplicationExt, GtkWindowExt};
use log::{debug, info, trace};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use crate::application::model::app_model::AppModel;
use crate::application::model::AppModelled;
use crate::application::view::app_view::{build_about_dialog, AppView};

pub struct AppController {
    model: Box<dyn AppModelled>,
    view: AppView,

    runtime: tokio::runtime::Runtime,
}

impl AppController {
    pub fn new(model: Box<dyn AppModelled>, view: AppView) -> AppController {
        debug!("AppController::new()");

        AppController {
            model,
            view,
            runtime: tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"),
        }
    }

    pub fn show_view(&self) {
        debug!("Showing Window");
        self.view.show_window();
    }

    fn connect_events(&mut self, application: &Application) {
        debug!("Connecting Events");

        let window = self.view.get_main_window();

        let window = window.clone();

        let about = gio::SimpleAction::new("about", None);

        about.connect_activate(|_, _| {
            debug!("About clicked");
            let p = build_about_dialog();
            p.run();
            p.close();
        });

        let quit = gio::SimpleAction::new("quit", None);
        let channel = self.model.get_termination_channel();

        quit.connect_activate(clone!(@strong window => move |_, _| {
            debug!("Quit clicked");
            window.close();
            channel.send(());
        }));

        let actions = vec![about, quit];

        for action in actions {
            application.add_action(&action);
        }

        debug!("Connecting Events Done");
    }

    pub fn try_login(&self, token: String) {
        debug!("Trying to login");
        trace!("Token: {}", token);
    }

    pub fn connect_application(&mut self, application: &gtk::Application) {
        debug!("Connecting Application");

        self.connect_events(application);

        let (app_menu, menu_bar) = self.view.get_menus();

        application.set_app_menu(Some(app_menu));
        application.set_menubar(Some(menu_bar));
        self.view.connect_application(application);
    }

    pub fn run(mut self) {
        debug!("AppController::run()");
        debug!("Running Application");

        self.check_and_show_login_dialog();

        self.show_view();
    }

    pub fn start_search(&mut self) {
        debug!("Searching");

            self.model.start_search();

    }

    fn check_and_show_login_dialog(&mut self) {
        let logged = self.model.is_logged_in();
        debug!("User is logged in: {}", logged);

        if !logged {
            self.view.show_login_dialog();
            let (tx, mut rx) = unbounded_channel::<String>();
            let mut otp_rx = self.view.connect_otp_channel(tx);

            self.connect_otp_validation(rx);
        } else {
            self.start_search();
        }
    }

    fn connect_otp_validation(&mut self, mut channel: UnboundedReceiver<String>) {
        trace!("app_controller::connect_otp_validation()");
        self.create_tokio_runtime().spawn(async move {
            loop {
                debug!("OTP Channel opened!");
                let token = channel.recv().await;
                debug!("{:?}", token);
                if let Some(token) = token {
                    debug!("Token: {}", token);
                } else {
                    break;
                }
            }

            debug!("FINISHED");
        });
    }

    fn create_tokio_runtime(&mut self) -> &Runtime {
        debug!("Creating Tokio Runtime");
        &self.runtime
    }
}
