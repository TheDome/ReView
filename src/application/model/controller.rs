use gio::prelude::*;
use glib::clone;
use gtk::{Application, DialogExt, GtkApplicationExt, GtkWindowExt};
use log::{debug, info};

use crate::application::model::app_model::AppModel;
use crate::application::view::ui::{build_about_dialog, AppView};

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

        quit.connect_activate(clone!(@strong window => move |_, _| {
            debug!("Quit clicked");
            window.close();
        }));

        let actions = vec![about, quit];

        for action in actions {
            application.add_action(&action);
        }
    }

    pub fn connect_application(&self, application: &gtk::Application) {
        debug!("Connecting Application");

        self.connect_events(application);

        let (app_menu, menu_bar) = self.view.get_menus();

        application.set_app_menu(Some(app_menu));
        application.set_menubar(Some(menu_bar));
        self.view.connect_application(application);
    }

    pub fn run(&self) {
        debug!("Running Application");
        self.show_view();
    }
}
