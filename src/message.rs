use crate::{prelude::Error, request::Request, response::Response};

#[derive(Debug)]
pub enum Message<'a> {
    Request(Request<'a>),
    Response(Response<'a>),
}

impl<'a> Message<'a> {
    pub(super) fn from_str(message: &'a str) -> Result<Self, Error> {
        if message.starts_with("HTTP/") {
            Ok(Self::Response(Response::parse(message)?))
        } else {
            Ok(Self::Request(Request::try_parse(message)?))
        }
    }
}

