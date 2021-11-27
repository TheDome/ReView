use gio::prelude::*;
use gio::{Menu, MenuItem};
use gtk::prelude::*;
use gtk::AboutDialogExt;
use log::trace;

use crate::application::application_config::{APPLICATION_VERSION, MAIN_WINDOW_NAME};

pub fn build_menu_bar(builder: &gtk::Builder) -> Menu {
    let menu_bar = Menu::new();

    let app_emenu = MenuItem::new(Some("File"), None);
    let file_menu = Menu::new();

    app_emenu.set_submenu(Some(&file_menu));

    let about_button = MenuItem::new(Some("About"), Some("_about"));

    menu_bar.append_item(&app_emenu);
    menu_bar.append_item(&about_button);

    add_actions(builder);

    menu_bar
}

fn add_actions(builder: &gtk::Builder) {
    // Insert the version since we extract it while building
    trace!("Creating about dialog");
    let about_dialog: gtk::AboutDialog = builder
        .get_object("mainAboutDialog")
        .expect("Could not find about dialog");
    about_dialog.set_version(Some(APPLICATION_VERSION));
    about_dialog.set_authors(
        env!("CARGO_PKG_AUTHORS")
            .split(';')
            .collect::<Vec<&str>>()
            .as_slice(),
    );
    about_dialog.set_logo(None);

    let about_action = gio::SimpleAction::new("_about", None);

    about_action.connect_change_state(move |_, _| {
        about_dialog.run();
        about_dialog.hide();
    });

    about_action.set_enabled(true);
}

pub fn build_app_window() -> gtk::ApplicationWindow {
    let window_str = include_str!("Windows.glade");
    let builder = gtk::Builder::from_string(window_str);

    let window: gtk::ApplicationWindow = builder
        .get_object("mainWindow")
        .expect("Could not find main window");

    window.set_title(MAIN_WINDOW_NAME);
    window.set_default_size(800, 600);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window
}
