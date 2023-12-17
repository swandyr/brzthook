mod buidler;
mod error;
mod message;
mod notification;
mod parse;
pub mod prelude;
mod request;
mod response;

use std::{
    fmt,
    fs::File,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Sender, Arc},
    time::Duration,
};

use message::Message;
use prelude::*;
use request::Request;
use response::Response;
use tracing::{debug, error, info, warn};

use crate::buidler::HookListenerBuilder;
use crate::error::{Error::SubscriptionError, HandleConnectionError, ParseError};

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Subscribe,
    Unsubscribe,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Subscribe => "subscribe",
                Self::Unsubscribe => "unsubscribe",
            }
        )
    }
}

#[derive(Debug)]
pub struct HookListener {
    pub listener: Arc<TcpListener>,
    pub callback: String,
    pub new_only: bool,
}

impl HookListener {
    pub fn builder() -> HookListenerBuilder {
        HookListenerBuilder::default()
    }

    /// Start listening for incoming streams.
    pub fn listen(&self, sender: &Sender<Result<Notification, Error>>) {
        info!("Start listening.");

        let listener = Arc::clone(&self.listener);
        let sender = sender.clone();
        let new_only = self.new_only;

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => match handle_connection(stream, new_only) {
                        Ok(reponse) => {
                            if let Some(notification) = reponse {
                                info!("Sending new notification");
                                sender.send(Ok(notification)).unwrap();
                            }
                        }
                        Err(e) => sender.send(Err(e)).unwrap(),
                    },
                    Err(e) => sender.send(Err(Error::TcpError(e))).unwrap(),
                }
            }
        });
    }

    /// Send a subscription/unsubscription request to the hub.
    ///
    /// It sends a POST request to the hub with the formatted topic url
    /// , the callback url and the subscription mode.
    ///
    /// # Panics:
    ///
    /// Subscription mode is not "subscribe" or "unsubscribe".
    ///
    /// Publisher configuration is not found.
    ///
    /// Request can not be streamed to the hub address.
    pub fn subscribe(&self, id: impl AsRef<str>, mode: Mode) -> Result<(), Error> {
        let id = id.as_ref();

        info!("Initiating {mode} request with id: {id}");

        //TODO: not hardcode this
        let topic_url = format!("https://www.youtube.com/xml/feeds/videos.xml?channel_id={id}");
        let hub = "https://pubsubhubbub.appspot.com";
        let callback_url = &self.callback;

        info!(
            r#"
Making subscription request to {hub} with:
hub.callback={}
hub.mode={}
hub.topic={}"#,
            &callback_url, mode, &topic_url
        );

        // Building the subscription request
        let body = format!("hub.callback={callback_url}&hub.mode={mode}&hub.topic={topic_url}");
        let len = body.as_bytes().len();
        let post_request = format!("POST / HTTP/1.1\r\nHost: pubsubhubbub.appspot.com\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {len}\r\n\r\n{body}");
        debug!("{post_request:?}");

        // Connect a socket and send the request
        let hub_addr = format!(
            "{}:80",
            hub.trim_start_matches("https://")
                .trim_start_matches("https://"),
        );
        let mut stream = TcpStream::connect(hub_addr)?;
        stream.write_all(post_request.as_bytes())?;
        stream.flush()?;

        Ok(())
    }
}

const BUF_SIZE: usize = 1024;

fn handle_connection(mut stream: TcpStream, new_only: bool) -> Result<Option<Notification>, Error> {
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    let mut buf_reader = BufReader::new(&mut stream);

    let mut n_bytes = 0;
    let mut received = vec![];
    loop {
        let mut buf = [0; BUF_SIZE];
        match buf_reader.read(&mut buf) {
            Ok(n) => {
                n_bytes += n;
                received.extend_from_slice(&buf);
                if n < BUF_SIZE {
                    break;
                }
            }
            Err(e) => {
                error!("Error reading buffer: {e}");
                stream.flush()?;
                return Err(Error::TcpError(e));
            }
        }
    }
    info!("Received {n_bytes} bytes");

    received.retain(|&b| b != 0);
    let message = String::from_utf8(received).map_err(HandleConnectionError::FormatUtf8Error)?;
    let message = Message::from_str(&message)?;

    debug!("Message:\n{message:#?}");

    let notification = match message {
        Message::Request(request) => handle_request(request, stream, new_only)?,
        Message::Response(response) => {
            handle_response(response)?;
            None
        }
    };

    debug!("End of handle_connection: {notification:#?}");
    Ok(notification)
}

fn handle_request(
    request: Request,
    mut stream: TcpStream,
    new_only: bool,
) -> Result<Option<Notification>, Error> {
    let from = request
        .headers
        .get("From")
        .ok_or_else(|| ParseError::NotFound("From header".to_string()))?;
    if *from != "googlebot(at)googlebot.com" {
        error!("unknown source : {from}");
        return Err(HandleConnectionError::Empty.into());
    }

    let method = request.request_line.method;
    let _path = request.request_line.path;

    let notification = match method {
        // The hub send a GET request for the verification of intent
        // The scubscriber must answer with a 2XX status code and echo the hub.challenge value
        "GET" => {
            info!("Received GET request");
            let params = request
                .request_line
                .params
                .ok_or_else(|| ParseError::ParameterError("No paramater in request".to_string()))?;

            if let Some(reason) = params.get("hub.reason") {
                return Err(SubscriptionError((*reason).to_string()));
            }

            let challenge = params
                .get("hub.challenge")
                .ok_or_else(|| ParseError::NotFound("hub.challenge".to_string()))?;
            let response = format!("HTTP/1.1 200 OK\r\n\r\n{challenge}");
            info!("Sending: {response:?}");
            stream.write_all(response.as_bytes())?;

            None
        }

        // Request when a new resource is published
        "POST" => {
            info!("Received POST request");

            let response = "HTTP/1.1 200 OK\r\n\r\n";
            info!("Sending: {response}");
            stream.write_all(response.as_bytes())?;

            let notification = Notification::try_parse(
                request
                    .body
                    .ok_or_else(|| HandleConnectionError::NoBodyError)?,
            )?;

            if new_only && !notification.is_new() {
                info!("It's an updated video; pass");
                None
            } else {
                Some(notification)
            }
        }
        _ => {
            warn!("Unhandled request: {request:#?}");

            None
        }
    };

    stream.flush()?;

    Ok(notification)
}

fn handle_response(response: Response) -> Result<(), Error> {
    let from = response
        .headers
        .get("From")
        .ok_or_else(|| ParseError::NotFound("From header".to_string()))?;
    if *from != "googlebot(at)googlebot.com" {
        error!("unknown source : {from}");
        return Err(HandleConnectionError::Empty.into());
    }
    let status_code = response.status_line.status_code;
    let _status_message = response.status_line.status_message;

    match status_code {
        "202" => {
            info!("Request accepted")
        }
        code if code.starts_with('4') || code.starts_with('5') => {
            let reason = response
                .body
                .ok_or_else(|| HandleConnectionError::NoBodyError)?;
            return Err(SubscriptionError(reason.to_string()));
        }
        _ => {
            warn!("Unhandled response: {response:#?}");
        }
    }

    Ok(())
}

#[allow(unused)]
fn write_to_file(message: &str) -> Result<(), std::io::Error> {
    let mut f = File::options().create(true).append(true).open("out.txt")?;
    writeln!(&mut f, "========== New Message ==========")?;
    writeln!(&mut f, "{message}")?;
    writeln!(&mut f)?;
    Ok(())
}
