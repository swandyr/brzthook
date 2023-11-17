use std::{
    collections::HashMap,
    fmt::{self},
    path::Path,
};

use crate::error::ParseRequestError;

pub(super) struct Request<'a> {
    pub(super) method: &'a str,
    pub(super) path: &'a Path,
    pub(super) params: Option<HashMap<&'a str, &'a str>>,
    pub(super) http_version: &'a str,
}

impl<'a> fmt::Display for Request<'a> {
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

pub(super) fn parse_request_line(request: &str) -> Result<Request, ParseRequestError> {
    let mut parts = request.split_whitespace();

    let method = parts
        .next()
        .ok_or_else(|| ParseRequestError::NotFound("Method".to_string()))?;

    let uri = parts
        .next()
        .ok_or_else(|| ParseRequestError::NotFound("URI".to_string()))?;

    let (path, params) = {
        if uri.contains('?') {
            let (path, params_string) = uri.split_once('?').ok_or_else(|| {
                ParseRequestError::ParameterError("No parameters in request".to_string())
            })?;

            let params_vec = params_string.split('&');
            let mut params = HashMap::new();
            for p in params_vec {
                let (key, value) = p.split_once('=').ok_or_else(|| {
                    ParseRequestError::ParameterError(
                        "Parameter found is not key=value".to_string(),
                    )
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
        Err(ParseRequestError::UriError)?;
    }

    let http_version = parts
        .next()
        .ok_or_else(|| ParseRequestError::NotFound("HTTP version".to_string()))?;

    Ok(Request {
        method,
        path,
        params,
        http_version,
    })
}
