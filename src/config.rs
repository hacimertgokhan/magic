use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub reflect: ReflectConfig,
    pub user: UserConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub bind_address: String,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectConfig {
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub password: String,
    pub targets: Option<HashMap<String, UserCredentials>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("magic.toml")?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn create_default_config() -> Result<(), Box<dyn std::error::Error>> {
    let default = r#"
[server]
port = 7070
bind_address = "127.0.0.1"
protocol = "reflect"

[reflect]
targets = ["127.0.0.1:7878", "192.168.1.5:7878"]

"#;
    fs::write("magic.toml", default)?;
    Ok(())
}
