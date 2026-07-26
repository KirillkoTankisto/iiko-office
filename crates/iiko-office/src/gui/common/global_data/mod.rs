use std::sync::{Arc, Mutex};

use gtk4::ApplicationWindow;
use iiko_api::IikoSession;

use crate::{
    cfg::OfficeConfig,
    error::AppError,
    gui::{common::message_bus::MessageBus, get_language, translation::CurrentLanguage},
};

pub struct GlobalData {
    session: Mutex<Option<Arc<IikoSession>>>,
    language: CurrentLanguage,
    config: Mutex<OfficeConfig>,
    message_bus: MessageBus,
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
        self.session
            .lock()
            .map_err(|_| AppError::Internal)?
            .clone()
            .ok_or(AppError::NotLoggedIn)
    }

    pub fn set_session(&self, session: IikoSession) {
        if let Ok(mut locked) = self.session.lock() {
            *locked = Some(Arc::new(session))
        }
    }

    pub fn take_session(&self) -> Option<Arc<IikoSession>> {
        self.session.lock().ok()?.take()
    }

    pub fn servers(&self) -> Vec<String> {
        self.config
            .lock()
            .map(|config| config.servers().to_vec())
            .unwrap_or_default()
    }

    pub fn add_server(&self, address: &str) {
        if let Ok(mut config) = self.config.lock() {
            config.add_server(address);
        }
    }

    pub fn remove_server(&self, address: &str) {
        if let Ok(mut config) = self.config.lock() {
            config.remove_server(address);
        }
    }

    pub fn write_config(&self) -> Result<(), AppError> {
        let config = self.config.lock().map_err(|_| AppError::Internal)?;
        Ok(config.write_config()?)
    }

    pub fn message_send(&self, error: AppError) {
        self.message_bus.emit(error);
    }

    pub fn message_attach(&self, window: &ApplicationWindow) {
        self.message_bus.attach(window, self.language);
    }
}
