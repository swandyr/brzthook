#![allow(unused)]
use std::{collections::HashMap, fmt};

pub(super) struct Response<'a> {
    http_version: &'a str,
    status_code: &'a str,
    status_message: &'a str,
    headers: HashMap<&'a str, &'a str>,
    body: Option<&'a str>,
}

impl<'a> fmt::Display for Response<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.http_version, self.status_code, self.status_message
        )
    }
}

pub(super) fn parse_response(response: &str) -> Result<Response, Box<dyn std::error::Error>> {
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
                let (key, value) = line.split_once(": ").ok_or("error parsing header")?;
                headers.insert(key, value);
            }
        } else {
            break;
        }
    }

    let empty_line = response.find("\r\n\r\n");
    let body = empty_line.map(|i| &response[(i + 1)..]);

    Ok(Response {
        http_version: http_version.ok_or("HTTP version not specified")?,
        status_code: status_code.ok_or("Status code not specified")?,
        status_message: status_message.ok_or("Status message not specified")?,
        headers,
        body,
    })
}
