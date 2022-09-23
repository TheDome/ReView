use std::sync::{Arc, Mutex};

use gio::prelude::*;
use glib::clone;
use gtk::{Application, DialogExt, GtkApplicationExt, GtkWindowExt};
use log::{debug, trace};

use crate::{
    application::{
        model::AppModelled,
        view::app_view::{build_about_dialog, AppView},
    },
    view::otp_view::OtpView,
};

pub struct AppController {
    model: Arc<Mutex<Box<dyn AppModelled>>>,
    view: Arc<AppView>,
    otp_view: Arc<OtpView>,
}

impl AppController {
    pub fn new(model: Box<dyn AppModelled>, view: AppView, otp_view: OtpView) -> AppController {
        debug!("AppController::new()");

        AppController {
            model: Arc::new(Mutex::new(model)),
            view: Arc::new(view),
            otp_view: Arc::new(otp_view),
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
        let channel = self.model.lock().unwrap().get_termination_channel();

        quit.connect_activate(clone!(@strong window => move |_, _| {
            debug!("Quit clicked");
            window.close();
            let _ =channel.send(());
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
        debug!("AppController::connect_application()");

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

        let _ = self.model.lock().unwrap().start_search();
    }

    fn check_and_show_login_dialog(&mut self) {
        let model = self.model.clone();
        let logged = model.lock().unwrap().is_logged_in();
        debug!("User is logged in: {}", logged);

        if !logged {
            self.otp_view.show_login_dialog();

            let rx = self.otp_view.connect_otp_channel();
            self.connect_otp_validation(rx);
        } else {
            self.start_search();
        }
    }

    fn connect_otp_validation(&self, channel: glib::Receiver<String>) {
        trace!("app_controller::connect_otp_validation()");

        let model = self.model.clone();
        let otp_view = self.otp_view.clone();

        channel.attach(None, move |otp| {
            trace!("OTP is: {}", otp);
            let result = model.lock().unwrap().login_user(otp);
            otp_view.show_validating_info();

            trace!("OTP Result: {:?}", result);

            match result {
                Ok(_) => {
                    debug!("OTP Validation passed!");
                    otp_view.close_login_dialog();
                    let _ = model.lock().unwrap().start_search();
                }
                Err(e) => {
                    debug!("OTP Validation failed: {}", e);
                    otp_view.show_info(e.as_str());
                }
            }

            glib::Continue(true)
        });
    }
}
