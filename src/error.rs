use crate::prelude::Notification;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Builder error")]
    Builder(#[from] BuilderError),
    #[error("TcpStream error")]
    TcpError(#[from] std::io::Error),
    #[error("Request error")]
    RequestLine(#[from] ParseRequestError),
    #[error("Subscription rejected")]
    SubscriptionError(String),
    #[error("Error while handling connection")]
    HandleConnection(#[from] HandleConnectionError),
    #[error("Notfication error")]
    Notification(#[from] NotificationError),
}

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("TCPListener cannot bind to address")]
    CannotBind(#[from] std::io::Error),
    #[error("Missing TCP Listener")]
    MissingListener,
    #[error("Missing callback URL")]
    MissingCallback,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseRequestError {
    #[error("{0} not found in request")]
    NotFound(String),
    #[error("{0}")]
    ParameterError(String),
    #[error("Requested resource does not exists")]
    UriError,
    #[error("No request line")]
    RequestLineError,
}

#[derive(Debug, thiserror::Error)]
pub enum HandleConnectionError {
    #[error("Message is empty")]
    Empty,
    #[error("Message has non-utf8 characters")]
    FormatUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Message has no body")]
    NoBodyError,
    #[error("Send error")]
    SendError(#[from] Box<std::sync::mpsc::SendError<Notification>>),
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Missing parameter {0}")]
    MissingParameter(String),
    #[error("OffsetDateTime parse error")]
    DateTimeError(#[from] time::error::Parse),
}
