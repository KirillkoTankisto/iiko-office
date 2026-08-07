use std::sync::{Arc, Mutex, MutexGuard};

use gtk4::ApplicationWindow;
use iiko_api::IikoSession;

use crate::gui::language::get_language;
use crate::gui::translation::CurrentLanguage;
use crate::{cfg::OfficeConfig, error::AppError, gui::common::message_bus::MessageBus};

pub struct GlobalData {
    session: Mutex<Option<Arc<IikoSession>>>,
    language: CurrentLanguage,
    config: Mutex<OfficeConfig>,
    message_bus: MessageBus,
}

/// lock the value and recover if value is poisoned
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl GlobalData {
    pub fn new() -> Arc<GlobalData> {
        Arc::new(GlobalData {
            session: Mutex::new(None),
            language: get_language(),
            config: Mutex::new(OfficeConfig::load_config()),
            message_bus: MessageBus::new(),
        })
    }

    pub fn language(&self) -> CurrentLanguage {
        self.language
    }

    pub fn session(&self) -> Result<Arc<IikoSession>, AppError> {
        lock(&self.session).clone().ok_or(AppError::NotLoggedIn)
    }

    pub fn set_session(&self, session: IikoSession) {
        *lock(&self.session) = Some(Arc::new(session));
    }

    pub fn take_session(&self) -> Option<Arc<IikoSession>> {
        lock(&self.session).take()
    }

    pub fn servers(&self) -> Vec<String> {
        lock(&self.config).servers().to_vec()
    }

    pub fn add_server(&self, address: &str) {
        lock(&self.config).add_server(address);
    }

    pub fn remove_server(&self, address: &str) {
        lock(&self.config).remove_server(address);
    }

    pub fn write_config(&self) -> Result<(), AppError> {
        Ok(lock(&self.config).write_config()?)
    }

    pub fn message_send(&self, error: AppError) {
        self.message_bus.emit(error);
    }

    pub fn message_attach(&self, window: &ApplicationWindow) {
        self.message_bus.attach(window, self.language);
    }
}
