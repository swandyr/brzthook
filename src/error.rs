#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid subscription mode \"{0}\"; valid modes are \"subscribe\" or \"unsubscribe\"")]
    SubscriptionModeError(String),
    #[error("Configuration error")]
    ConfigurationError(#[from] ConfigurationError),
    #[error("TcpStream error")]
    TcpError(#[from] std::io::Error),
    #[error("Missing parameter")]
    NotificationError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Configuration file not found")]
    MissingFile(#[from] std::io::Error),
    #[error("Configuration file can't be read")]
    ParseError(#[from] toml::de::Error),
    #[error("Publisher {0} not found")]
    PublisherNotFoundError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseRequestError {
    #[error("{0} not found in request")]
    NotFound(String),
    #[error("{0}")]
    ParameterError(String),
    #[error("Requested resource does not exists")]
    UriError,
}
