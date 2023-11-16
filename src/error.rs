use std::{error::Error, fmt};

#[derive(Debug)]
pub struct MyError {
    pub source: CallbackError,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum CallbackError {
    SubscriptionMode,
}

impl fmt::Display for CallbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SubscriptionMode => "Invalid subscription mode",
        };

        write!(f, "{message}")
    }
}

impl Error for CallbackError {}
