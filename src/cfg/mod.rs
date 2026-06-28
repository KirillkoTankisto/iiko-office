use std::{
    env::home_dir,
    fs::{create_dir_all, write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]

pub struct OfficeConfig {
    servers: Vec<String>,
}

impl OfficeConfig {
    pub fn load_config() -> Self {
        let buffer = std::fs::read_to_string(get_config_path()).unwrap_or_default();
        serde_json::from_str(&buffer).unwrap_or_default()
    }

    pub fn write_config(&self) {
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            create_dir_all(parent).expect("failed to create config dir");
        }
        let string = serde_json::to_string_pretty(self).unwrap_or_default();
        write(&path, string).expect("failed to write to config file");
    }

    pub fn servers(&self) -> Vec<String> {
        self.servers.clone()
    }

    pub fn add_server(&mut self, address: &str) {
        let address = address.to_string();
        if !self.servers.contains(&address) {
            self.servers.push(address)
        }
    }

    pub fn remove_server(&mut self, address: &str) {
        self.servers.retain(|s| s != address);
    }
}

fn get_config_path() -> PathBuf {
    let mut path = home_dir().expect("failed to get home dir");
    path.push(".config");
    path.push("iikoOffice");
    path.push("config.json");
    path
}
