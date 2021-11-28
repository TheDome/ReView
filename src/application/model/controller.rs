use gio::prelude::*;
use gtk::{Application, GtkApplicationExt};
use log::{debug, info};

use crate::application::model::app_model::AppModel;
use crate::application::view::ui::AppView;

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

    pub fn connect_application(&self, application: &gtk::Application) {
        debug!("Connecting Application");

        let actions = self.view.connect_actions();

        for action in actions {
            application.add_action(&action);
        }

        let (app_menu, menu_bar) = self.view.get_menus();

        application.set_app_menu(Some(app_menu));
        application.set_menubar(Some(menu_bar));
        self.view.connect_application(application);
    }

    pub fn run(&self) {
        debug!("Running Application");
        self.view.connect_controller(self);

        self.show_view();
    }

    pub fn show_about_dialog(&self) {
        info!("About Menu Pressed");
    }
}
