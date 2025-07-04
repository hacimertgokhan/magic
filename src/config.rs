use serde::Deserialize;
use std::fs;


#[derive(Deserialize, Debug, Clone)]
pub struct ReflectConfig {
    pub targets: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub bind_address: String,
    pub protocol: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub reflect: ReflectConfig,
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
