use async_channel::{Receiver, Sender};
use gtk4::{ApplicationWindow, glib, prelude::*};
use iiko_api::error::ClientError;

use crate::{
    error::AppError,
    gui::{
        common::modal::{Modal, closer, label, wrapped},
        translation::{
            CurrentLanguage,
            Line::{
                ERROR_ADDRESS, ERROR_INTERNAL, ERROR_REQUEST, ERROR_RESPONSE, ERROR_UNAUTHORIZED,
            },
            translate,
        },
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
        let modal = Modal::install(window);
        let receiver = self.receiver.clone();

        glib::spawn_future_local(async move {
            while let Ok(error) = receiver.recv().await {
                let (heading, detail) = describe(&error, language);
                let title = label(heading, &["heading"]);
                let body = wrapped(&detail);
                let (close, closed) = closer(language);
                close.set_halign(gtk4::Align::End);

                modal
                    .show(
                        18,
                        &[title.upcast_ref(), body.upcast_ref(), close.upcast_ref()],
                        closed,
                    )
                    .await;
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
        AppError::Config(_) | AppError::NotLoggedIn => ERROR_INTERNAL,
    };
    (translate(language, line), error.to_string())
}
