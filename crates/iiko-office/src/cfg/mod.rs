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
        std::fs::read_to_string(get_config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn write_config(&self) -> std::io::Result<()> {
        let path = get_config_path();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let string = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        write(&path, string)
    }

    pub fn servers(&self) -> &[String] {
        &self.servers
    }

    pub fn add_server(&mut self, address: &str) {
        if !self.servers.iter().any(|server| server == address) {
            self.servers.push(address.to_string());
        }
    }

    pub fn remove_server(&mut self, address: &str) {
        self.servers.retain(|s| s != address);
    }
}

fn get_config_path() -> PathBuf {
    if let Some(mut path) = home_dir() {
        path.push(".config");
        path.push("iikoOffice");
        path.push("config.json");
        path
    } else {
        PathBuf::from("~/.config/iikoOffice/config.json")
    }
}
