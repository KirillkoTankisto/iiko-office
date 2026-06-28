use std::sync::Arc;

use crate::api::error::ClientError;
use crate::gui::{GlobalData, UserData};

use gtk4::glib;
use gtk4::prelude::*;

pub fn spawn_workflow<T, W, U>(
    gdata: Arc<GlobalData>,
    button: Option<&gtk4::Button>,
    work: W,
    ui: U,
) where
    T: Send + 'static,
    W: FnOnce(UserData) -> Result<T, ClientError> + Send + 'static,
    U: FnOnce(T) + 'static,
{
    let button = button.cloned();

    if let Some(button) = &button {
        button.set_sensitive(false);
    }

    let (sender, receiver) = async_channel::bounded(1);

    std::thread::spawn(move || {
        let result = gdata.get_credentials().and_then(work);
        let _ = sender.send_blocking(result);
    });

    glib::spawn_future_local(async move {
        match receiver.recv().await {
            Ok(Ok(v)) => ui(v),
            Ok(Err(e)) => eprintln!("{e}"),
            Err(_) => {}
        }
        if let Some(b) = button {
            b.set_sensitive(true);
        }
    });
}
