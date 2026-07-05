use async_channel::{Receiver, Sender};
use gtk4::{AlertDialog, ApplicationWindow, glib};
use iiko_api::error::ClientError;

use crate::{
    error::AppError,
    gui::translation::{
        CurrentLanguage,
        Line::{
            CLOSE, ERROR_ADDRESS, ERROR_INTERNAL, ERROR_REQUEST, ERROR_RESPONSE, ERROR_UNAUTHORIZED,
        },
        translate,
    },
};

pub struct MessageBus {
    sender: Sender<AppError>,
    receiver: Receiver<AppError>,
}

impl MessageBus {
    pub fn new() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { sender, receiver }
    }

    pub fn emit(&self, error: AppError) {
        let _ = self.sender.send_blocking(error);
    }

    pub fn attach(&self, window: &ApplicationWindow, language: CurrentLanguage) {
        let receiver = self.receiver.clone();
        let window = window.clone();

        glib::spawn_future_local(async move {
            while let Ok(error) = receiver.recv().await {
                let (heading, detail) = describe(&error, language);
                AlertDialog::builder()
                    .modal(true)
                    .message(heading)
                    .detail(detail)
                    .buttons([translate(language, CLOSE)])
                    .build()
                    .show(Some(&window));
            }
        });
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

fn describe(error: &AppError, language: CurrentLanguage) -> (&'static str, String) {
    let line = match error {
        AppError::Api(api) => match api {
            ClientError::Url(_) => ERROR_ADDRESS,
            ClientError::Http(_) => ERROR_REQUEST,
            ClientError::Json(_) | ClientError::Xml(_) => ERROR_RESPONSE,
            ClientError::Unauthorized => ERROR_UNAUTHORIZED,
        },
        AppError::Config(_) | AppError::NotLoggedIn | AppError::Internal => ERROR_INTERNAL,
    };
    (translate(language, line), error.to_string())
}
