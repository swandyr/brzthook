use std::fmt;

pub(super) struct Response<'a> {
    http_version: &'a str,
    status_code: &'a str,
    status_message: &'a str,
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

#[allow(unused)]
pub(super) fn parse_status_line(status_line: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let mut parts = status_line.split_whitespace();

    let http_version = parts.next().ok_or("HTTP version not specified")?;
    let status_code = parts.next().ok_or("Status Code not specified")?;
    let status_message = parts.next().ok_or("Status Message not specified")?;

    Ok(Response {
        http_version,
        status_code,
        status_message,
    })
}
