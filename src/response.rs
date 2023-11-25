#![allow(unused)]
use std::{collections::HashMap, fmt};

use crate::error::ParseError;

#[derive(Debug)]
pub(super) struct ResponseLine<'a> {
    pub(super) http_version: &'a str,
    pub(super) status_code: &'a str,
    pub(super) status_message: &'a str,
}

#[derive(Debug)]
pub(super) struct Response<'a> {
    pub(super) status_line: ResponseLine<'a>,
    pub(super) headers: HashMap<&'a str, &'a str>,
    pub(super) body: Option<&'a str>,
}

impl<'a> fmt::Display for Response<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.status_line.http_version,
            self.status_line.status_code,
            self.status_line.status_message
        )
    }
}

impl<'a> Response<'a> {
    pub(super) fn parse(response: &'a str) -> Result<Self, ParseError> {
        let mut http_version = None;
        let mut status_code = None;
        let mut status_message = None;
        let mut headers = HashMap::new();
        for (i, line) in response.lines().enumerate() {
            if !line.is_empty() {
                if i == 0 {
                    let mut status_line = line.split_whitespace();
                    http_version = status_line.next();
                    status_code = status_line.next();
                    status_message = status_line.next();
                } else {
                    let (key, value) = line
                        .split_once(": ")
                        .ok_or_else(|| ParseError::HeaderError(line.to_string()))?;
                    headers.insert(key, value);
                }
            } else {
                break;
            }
        }
        let status_line = ResponseLine {
            http_version: http_version
                .ok_or_else(|| ParseError::NotFound("HTTP version".to_string()))?,
            status_code: status_code
                .ok_or_else(|| ParseError::NotFound("Status code".to_string()))?,
            status_message: status_message
                .ok_or_else(|| ParseError::NotFound("Status message".to_string()))?,
        };

        let empty_line = response.find("\r\n\r\n");
        let body = empty_line.map(|i| &response[(i + 1)..]);

        Ok(Self {
            status_line,
            headers,
            body,
        })
    }
}
