use std::{fmt::Display, fs};

use serde::Deserialize;

use crate::error::ConfigurationError;

#[derive(Debug, Deserialize, Clone)]
pub(super) struct Config {
    pub(super) server: CfgServer,
    pub(super) youtube: CfgSubs,
}

impl Config {
    pub(super) fn from_file(path: &str) -> Result<Self, ConfigurationError> {
        let file = fs::read_to_string(path).map_err(ConfigurationError::MissingFile)?;
        let config: Config = toml::from_str(&file).map_err(ConfigurationError::ParseError)?;

        Ok(config)
    }

    pub(super) fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub(super) fn callback_address(&self) -> String {
        format!("{}:{}", self.server.callback, self.server.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct CfgServer {
    pub(super) port: u16,
    pub(super) host: String,
    pub(super) callback: String,
}

#[derive(Debug, Deserialize, Clone)]
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
