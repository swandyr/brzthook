use std::{
    collections::HashMap,
    fmt::{self},
    path::Path,
};

use crate::error::ParseError;

#[derive(Debug)]
pub(super) struct RequestLine<'a> {
    pub(super) method: &'a str,
    pub(super) path: &'a Path,
    pub(super) params: Option<HashMap<&'a str, &'a str>>,
    pub(super) http_version: &'a str,
}

impl<'a> fmt::Display for RequestLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = if let Some(map) = &self.params {
            let mut params = String::from("?");
            for (k, v) in map {
                params.push_str(&format!("{k}={v}&"));
            }
            params.pop();
            params
        } else {
            String::new()
        };
        write!(
            f,
            "{} {}{} {}\r\n",
            self.method,
            self.path.display(),
            params,
            self.http_version
        )
    }
}

#[allow(unused)]
#[derive(Debug)]
pub(super) struct Request<'a> {
    pub(super) request_line: RequestLine<'a>,
    pub(super) headers: HashMap<&'a str, &'a str>,
    pub(super) body: Option<&'a str>,
}

impl<'a> Request<'a> {
    pub(super) fn try_parse(request: &'a str) -> Result<Self, ParseError> {
        let mut request_line = None;
        let mut headers = HashMap::new();

        for (i, line) in request.lines().enumerate() {
            if !line.is_empty() {
                if i == 0 {
                    request_line = Some(parse_request_line(line)?);
                } else {
                    let (key, value) = line
                        .split_once(": ")
                        .ok_or_else(|| ParseError::ParameterError("error in header".to_string()))?;
                    headers.insert(key, value);
                }
            } else {
                break;
            }
        }

        let empty_line = request.find("\r\n\r\n");
        let body = empty_line.map(|i| &request[(i + 1)..]);

        Ok(Self {
            request_line: request_line
                .ok_or_else(|| ParseError::NotFound("request line".to_string()))?,
            headers,
            body,
        })
    }
}

pub(super) fn parse_request_line(request_line: &str) -> Result<RequestLine, ParseError> {
    let mut parts = request_line.split_whitespace();

    let method = parts
        .next()
        .ok_or_else(|| ParseError::NotFound("Method".to_string()))?;

    let uri = parts
        .next()
        .ok_or_else(|| ParseError::NotFound("URI".to_string()))?;

    let (path, params) = {
        if uri.contains('?') {
            let (path, params_string) = uri.split_once('?').ok_or_else(|| {
                ParseError::ParameterError("No parameters in request".to_string())
            })?;

            let params_vec = params_string.split('&');
            let mut params = HashMap::new();
            for p in params_vec {
                let (key, value) = p.split_once('=').ok_or_else(|| {
                    ParseError::ParameterError("Parameter found is not key=value".to_string())
                })?;
                params.insert(key, value);
            }

            (Path::new(path), Some(params))
        } else {
            (Path::new(uri), None)
        }
    };

    let norm_uri = path.to_str().expect("Invalid unicode!");

    const ROOT: &str = ".";

    if !Path::new(&format!("{ROOT}{norm_uri}")).exists() {
        Err(ParseError::UriError)?;
    }

    let http_version = parts
        .next()
        .ok_or_else(|| ParseError::NotFound("HTTP version".to_string()))?;

    Ok(RequestLine {
        method,
        path,
        params,
        http_version,
    })
}
