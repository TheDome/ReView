use std::sync::mpsc::Sender;

use gio::{Action, ActionMapExt, Menu, SimpleAction};
use glib::{clone, GString, Receiver};
use gtk::prelude::*;
use gtk::{AboutDialogExt, Builder, Button, Entry, Widget};
use gtk::{HeaderBar, Label, MenuBar, MenuItem, WindowPosition};
use log::{debug, info, trace, warn};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::application::model::app_controller::AppController;
use crate::application::view::{APPLICATION_VERSION, MAIN_WINDOW_NAME};
use crate::view::APP_WINDOWS_STRING;

#[derive(Debug, Clone)]
pub struct AppView {
    window: gtk::ApplicationWindow,
    about_dialog: gtk::AboutDialog,
    about_menu: MenuItem,
    app_menu: Menu,
    menu_bar: Menu,
}

impl AppView {
    pub fn new() -> Self {
        let app_menu = Menu::new();
        let menu_bar = Menu::new();

        let builder = gtk::Builder::from_string(APP_WINDOWS_STRING);

        let about_menu = build_about_menu();
        let about_dialog = build_about_dialog();
        let window = build_app_window(&builder);

        let more_menu = Menu::new();

        more_menu.append(Some("About"), Some("app.about"));

        app_menu.append(Some("Quit"), Some("app.quit"));
        menu_bar.append_submenu(Some("?"), &more_menu);

        AppView {
            window,
            about_dialog,
            about_menu,
            app_menu,
            menu_bar,
        }
    }

    pub fn show_window(&self) {
        self.window.show_all();
    }

    pub fn get_main_window(&self) -> &gtk::ApplicationWindow {
        &self.window
    }

    pub fn get_menus(&self) -> (&Menu, &Menu) {
        (&self.app_menu, &self.menu_bar)
    }

    pub fn connect_application(&self, app: &gtk::Application) {
        debug!("Connecting Application");
        self.window.set_application(Some(app));
    }
}

fn build_about_menu() -> MenuItem {
    let about_menu = MenuItem::new();
    about_menu.set_label("About");

    about_menu
}

pub fn build_about_dialog() -> gtk::AboutDialog {
    let about_dialog = gtk::AboutDialog::new();

    about_dialog.set_program_name(MAIN_WINDOW_NAME);
    about_dialog.set_version(Some(APPLICATION_VERSION));
    about_dialog.set_copyright(Some("Copyright © 2021"));
    about_dialog.set_comments(Some(env!("CARGO_PKG_DESCRIPTION")));
    about_dialog.set_license_type(gtk::License::Custom);
    about_dialog.set_license(Some(include_str!("../../../LICENSE")));
    about_dialog.set_website(Some(env!("CARGO_PKG_REPOSITORY")));
    about_dialog.set_website_label(Some("GitHub"));
    about_dialog.set_authors(
        env!("CARGO_PKG_AUTHORS")
            .split(";")
            .collect::<Vec<&str>>()
            .as_slice(),
    );
    about_dialog.set_logo(None);

    about_dialog.set_modal(false);
    about_dialog.set_destroy_with_parent(true);
    about_dialog.set_position(gtk::WindowPosition::CenterAlways);
    about_dialog.set_icon_name(Some("gtk-about"));
    about_dialog.set_wrap_license(true);

    about_dialog
}

pub fn build_app_window(builder: &gtk::Builder) -> gtk::ApplicationWindow {
    let window: gtk::ApplicationWindow = builder
        .get_object("mainWindow")
        .expect("Could not find main window");

    window.set_title(MAIN_WINDOW_NAME);
    window.set_default_size(800, 600);
    window.set_position(gtk::WindowPosition::Center);

    window
}
