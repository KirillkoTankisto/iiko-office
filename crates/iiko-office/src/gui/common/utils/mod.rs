use std::sync::Arc;

use crate::error::AppError;
use crate::gui::GlobalData;

use gtk4::glib;
use gtk4::prelude::*;
use iiko_api::IikoSession;

pub fn spawn_task<T, W, U>(gdata: Arc<GlobalData>, button: Option<&gtk4::Button>, work: W, ui: U)
where
    T: Send + 'static,
    W: FnOnce() -> Result<T, AppError> + Send + 'static,
    U: FnOnce(T) + 'static,
{
    let button = button.cloned();

    if let Some(button) = &button {
        button.set_sensitive(false);
    }

    let (sender, receiver) = async_channel::bounded(1);

    std::thread::spawn(move || {
        let _ = sender.send_blocking(work());
    });

    glib::spawn_future_local(async move {
        match receiver.recv().await {
            Ok(Ok(v)) => ui(v),
            Ok(Err(e)) => gdata.message_send(e),
            Err(_) => {}
        }
        if let Some(b) = button {
            b.set_sensitive(true);
        }
    });
}

pub fn spawn_workflow<T, E, W, U>(
    gdata: Arc<GlobalData>,
    button: Option<&gtk4::Button>,
    work: W,
    ui: U,
) where
    T: Send + 'static,
    E: Into<AppError> + Send + 'static,
    W: FnOnce(Arc<IikoSession>) -> Result<T, E> + Send + 'static,
    U: FnOnce(T) + 'static,
{
    let gdata_work = gdata.clone();
    spawn_task(
        gdata,
        button,
        move || {
            let session = gdata_work.session()?;
            work(session).map_err(Into::into)
        },
        ui,
    );
}
