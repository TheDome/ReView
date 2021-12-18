use std::sync::mpsc::channel;

use futures_util::FutureExt;
use gio::prelude::*;
use glib::clone;
use gtk::{Application, DialogExt, GtkApplicationExt, GtkWindowExt};
use log::{debug, info};

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

        let actions = vec![about, quit];

        for action in actions {
            application.add_action(&action);
        }

        debug!("Connecting Events Done");
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

        self.model.is_logged_in().then(|logged| {
            if !logged {
                debug!("User is NOT logged in");
                self.view.show_login_required();
            }
            async {}
        });

        self.show_view();
        self.start_search();
    }

    pub fn start_search(self) {
        debug!("Searching");
        self.model.start_search();
    }
}
