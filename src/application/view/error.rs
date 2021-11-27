use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType, Window, WindowType};

/// Displays an error message.
pub fn show_error(title: &str, message: &str) {
    let error_message = MessageDialog::new(
        Some(&Window::new(WindowType::Toplevel)),
        DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        message,
    );

    error_message.set_title(title);

    error_message.show_all();
    error_message.close();
}
