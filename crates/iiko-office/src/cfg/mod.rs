use std::{
    env::{home_dir, var_os},
    fs::{create_dir_all, read_to_string, write},
    io,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct OfficeConfig {
    servers: Vec<String>,
}

impl OfficeConfig {
    pub fn load_config() -> Self {
        config_path()
            .and_then(|path| read_to_string(path).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn write_config(&self) -> io::Result<()> {
        let path = config_path()
            .ok_or_else(|| io::Error::other("no configuration directory available"))?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        let string = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
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

fn config_path() -> Option<PathBuf> {
    let base = var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| home_dir().map(|home| home.join(".config")))?;

    Some(base.join("iikoOffice").join("config.json"))
}
