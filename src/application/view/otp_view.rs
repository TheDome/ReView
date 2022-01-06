use crate::view::APP_WINDOWS_STRING;
use gio::ActionMapExt;
use glib::Receiver;
use gtk::prelude::BuilderExtManual;
use gtk::{
    Button, ButtonExt, Entry, EntryExt, GtkWindowExt, Inhibit, Label, LabelExt, WidgetExt,
    WindowPosition,
};
use log::{debug, warn};

pub struct OtpView {
    otp_dialog: gtk::Window,

    otp_entry: Entry,
    otp_button: Button,
    otp_information_label: Label,
}

impl OtpView {
    pub fn new() -> Self {
        let builder = gtk::Builder::from_string(APP_WINDOWS_STRING);
        let otp_entry = builder.get_object("otp_entry").unwrap();
        let otp_ok_button: Button = builder.get_object("otp_ok_button").unwrap();
        let otp_information_label: Label = builder.get_object("otp_information_label").unwrap();

        let otp_dialog = build_otp_dialog(None, &builder);

        let login_window_action = gio::SimpleActionGroup::new();
        otp_dialog.insert_action_group("otp_dialog", Some(&login_window_action));

        let login_window_ok_action = gio::SimpleAction::new("ok", None);
        login_window_action.add_action(&login_window_ok_action);

        OtpView {
            otp_dialog,

            otp_button: otp_ok_button,
            otp_entry,
            otp_information_label,
        }
    }
    pub fn show_login_dialog(&self) {
        debug!("app_view::show_login_dialog");

        self.otp_dialog.set_position(WindowPosition::CenterOnParent);
        self.otp_dialog.set_keep_above(true);
        self.otp_dialog.show_all();
    }

    pub fn get_otp_text(&self) -> String {
        self.otp_entry.get_text().to_string()
    }

    /// Returns a channel where the value of the otp_entry will be sent when the button has been pressed
    pub fn connect_otp_channel(&self) -> Receiver<String> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let entry = self.otp_entry.clone();
        self.otp_button.connect_clicked(move |_| {
            let otp_text = entry.get_text().to_string();
            if let Err(e) = tx.send(otp_text) {
                warn!("Error sending otp: {:?}", e);
            }
        });
        rx
    }

    pub fn show_info(&self, info: &str) {
        self.otp_information_label.set_text(info);
    }

    pub fn clear_info(&self) {
        self.otp_information_label.set_text("");
    }

    pub fn show_validating_info(&self) {
        self.otp_information_label.set_text("Validating OTP...");
    }
}

fn build_otp_dialog(parent: Option<&gtk::Window>, builder: &gtk::Builder) -> gtk::Window {
    let window: gtk::Window = builder
        .get_object("login_window")
        .expect("Could not find login_window in glade file");

    window.connect_delete_event(|_, _| Inhibit(false));
    if parent.is_some() {
        window.set_transient_for(parent);
    }

    window
}
