use std::{fmt::Display, fs};

use serde::Deserialize;

use crate::error::ConfigurationError;

#[derive(Debug, Deserialize, Clone, Default)]
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

impl Default for CfgServer {
    fn default() -> Self {
        let port = 7878;
        let host = String::from("127.0.0.1");
        let callback = format!("http://{host};{port}");

        Self {
            port,
            host,
            callback,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct CfgSubs {
    hub: String,
    topic: String,
}

impl Default for CfgSubs {
    fn default() -> Self {
        Self {
            hub: String::from("https://pubsubhubbub.appspot.com"),
            topic: String::from("https://www.youtube.com/xml/feeds/videos.xml?channel_id="),
        }
    }
}

impl CfgSubs {
    pub(super) fn topic_address<T: AsRef<str> + Display>(&self, id: T) -> String {
        format!("{}{}", self.topic, id)
    }

    pub(super) fn hub_address(&self) -> String {
        self.hub.to_owned()
    }
}
