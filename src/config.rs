use std::{fmt::Display, fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Config {
    pub(super) server: CfgServer,
    pub(super) youtube: Option<CfgSubs>,
    pub(super) twitch: Option<CfgSubs>,
}

impl Config {
    pub(super) fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&file)?;

        Ok(config)
    }

    pub(super) fn callback_address(&self) -> String {
        format!("{}:{}", self.server.callback, self.server.port)
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct CfgServer {
    pub(super) port: u16,
    pub(super) host: String,
    pub(super) callback: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct CfgSubs {
    hub: String,
    topic: String,
}

impl CfgSubs {
    pub(super) fn topic_address<T: AsRef<str> + Display>(&self, id: T) -> String {
        format!("{}{}", self.topic, id)
    }

    pub(super) fn hub_address(&self) -> String {
        self.hub.to_owned()
    }
}
