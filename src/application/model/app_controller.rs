use std::sync::mpsc::channel;

use futures_util::FutureExt;
use gio::prelude::*;
use glib::clone;
use gtk::{Application, DialogExt, EditableExt, GtkApplicationExt, GtkWindowExt};
use log::{debug, info, trace};

use crate::application::model::app_model::AppModel;
use crate::application::view::app_view::{build_about_dialog, AppView};

pub struct AppController {
    model: AppModel,
    view: AppView,
}

impl AppController {
    pub fn new(model: AppModel, view: AppView) -> AppController {
        debug!("AppController::new()");

        AppController { model, view }
    }

    pub fn show_view(&self) {
        debug!("Showing Window");
        self.view.show_window();
    }

    fn connect_events(&self, application: &Application) {
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

        let ok_action = self.view.login_window_ok_action.clone();
        let entry = self.view.otp_entry.clone();
        ok_action.connect_activate(move |_, _| {
            debug!("Login clicked");
            entry.insert_text("", &mut 0);
        });

        let actions = vec![about, quit, ok_action];

        for action in actions {
            application.add_action(&action);
        }

        debug!("Connecting Events Done");
    }

    pub fn try_login(&self, token: String) {
        debug!("Trying to login");
        trace!("Token: {}", token);
    }

    pub fn connect_application(&self, application: &gtk::Application) {
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
        self.start_search();
    }

    pub fn start_search(self) {
        debug!("Searching");
        self.model.start_search();
    }

    fn check_and_show_login_dialog(&self) {
        let logged = Self::create_tokio_runtime().block_on(self.model.is_logged_in());
        debug!("User is logged in: {}", logged);

        if (!logged) {
            self.view.show_login_dialog();
        }
    }

    fn create_tokio_runtime() -> tokio::runtime::Runtime {
        debug!("Creating Tokio Runtime");
        tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
    }
}
