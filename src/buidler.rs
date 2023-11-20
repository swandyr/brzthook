#![allow(unused)]
use crate::error::BuilderError;
use crate::HookListener;
use std::{fmt::Display, io, net::TcpListener, sync::Arc};

#[derive(Debug, Default)]
pub struct HookListenerBuilder {
    listener: Option<TcpListener>,
    callback: Option<String>,
    new_only: bool,
}

impl HookListenerBuilder {
    pub fn listener(mut self, address: impl Into<String>, port: u32) -> Result<Self, BuilderError> {
        let bind = format!("{}:{}", address.into(), port);
        self.listener = Some(TcpListener::bind(bind).map_err(BuilderError::CannotBind)?);
        Ok(self)
    }

    pub fn callback(mut self, callback: impl Into<String>) -> Self {
        self.callback = Some(callback.into());
        self
    }

    pub fn new_only(mut self, new_only: bool) -> Self {
        self.new_only = new_only;
        self
    }

    pub fn build(self) -> Result<HookListener, BuilderError> {
        Ok(HookListener {
            listener: Arc::new(self.listener.ok_or_else(|| BuilderError::MissingListener)?),
            callback: self.callback.ok_or_else(|| BuilderError::MissingCallback)?,
            new_only: self.new_only,
        })
    }
}
