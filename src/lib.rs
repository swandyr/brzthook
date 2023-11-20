use std::{
    fs::File,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Sender, Arc},
};

mod config;
mod error;
mod notification;
mod parse;
pub mod prelude;
mod request;
mod response;

use config::Config;
use prelude::*;
use tracing::{debug, error, info, warn};

use crate::error::{HandleConnectionError, ParseRequestError};

const CONFIG_PATH: &str = "brzthook.toml";

#[derive(Debug)]
pub struct HookListener {
    listener: Arc<TcpListener>,
    config: Config,
    updated: bool,
}

//TODO: Handle resubscription before the expiration delay (5 days for youtube) (or maybe let caller
//handle it)
//TODO: Write function to automatically get hu address

impl Default for HookListener {
    fn default() -> Self {
        let config = Config::default();
        let addr = config.server_address();

        Self {
            listener: Arc::new(TcpListener::bind(&addr).expect("Cannot bind to localhost:7878")),
            config,
            updated: false,
        }
    }
}

impl HookListener {
    /// Create a new listener binded to the server's "{host}:{port}" address  in the webhook.toml
    /// config file, then instanciate a threadpool of four threads.
    ///
    /// This function returns the listener and the channel's receiver where notification will be
    /// sent.
    ///
    /// # Panics:
    ///
    /// Configuration file does not exist or is malformed.
    ///
    /// TcpListener can not bind to the address.
    pub fn new() -> Result<Self, Error> {
        let config = Config::from_file(CONFIG_PATH)?;
        info!("Config loaded");
        let addr = config.server_address();
        let updated = false;

        let listener = Self {
            listener: Arc::new(TcpListener::bind(&addr)?),
            config,
            updated,
        };
        info!("TCPListener binded to {}", &addr);

        Ok(listener)
    }

    /// Start listening for incoming streams.
    pub fn listen(&self, sender: &Sender<Result<Notification, Error>>) {
        info!("Start listening.");

        let listener = Arc::clone(&self.listener);
        let sender = sender.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => match handle_connection(stream) {
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

    pub fn addresses(&self, id: &str) -> (String, String, String) {
        let callback_address = self.config.callback_address();
        let topic_address = self.config.youtube.topic_address(id);
        let hub_address = self.config.youtube.hub_address();
        (callback_address, topic_address, hub_address)
    }

    /// Reload the webhook.toml configuration file.
    ///
    /// # Panics:
    ///
    /// Configuration file does not exist or is malformed.
    pub fn reload_config(&mut self) -> Result<(), Error> {
        let config = Config::from_file(CONFIG_PATH)?;
        self.config = config;
        info!("Config reloaded");
        Ok(())
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
    pub fn subscribe(&self, id: impl AsRef<str>, mode: impl AsRef<str>) -> Result<(), Error> {
        let id = id.as_ref();
        let mode = mode.as_ref();

        info!("Initiating {mode} request with id: {id}");

        if !(mode == "subscribe" || mode == "unsubscribe") {
            return Err(Error::SubscriptionModeError(mode.into()));
        }

        let subscription = &self.config.youtube;
        let topic_url = subscription.topic_address(id);
        let callback_url = self.config.callback_address();
        let hub = subscription.hub_address();

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

fn handle_connection(mut stream: TcpStream) -> Result<Option<Notification>, Error> {
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
                return Err(Error::TcpError(e));
            }
        }
    }
    info!("Received {n_bytes} bytes");

    received.retain(|&b| b != 0);
    let message = String::from_utf8(received).map_err(HandleConnectionError::FormatUtf8Error)?;
    let message_lines: Vec<&str> = message.trim().lines().collect();

    debug!("Message:\n{message}");

    let request_line = message_lines.first().ok_or(HandleConnectionError::Empty)?;
    debug!("Received request line: {request_line}");

    let notification = match *request_line {
        // The hub send 202 Accepted if the subscription request is accepted
        "HTTP/1.1 202 ACCEPTED" => {
            info!("Subscription accepted");

            None
        }

        // The hub senf a GET request for the verification of intent
        // The scubscriber must answer with a 2XX status code and echo the hub.challenge value
        request if request.starts_with("GET") => {
            info!("Received GET request");
            let request_line = request::parse_request_line(request)?;
            let params = request_line.params.ok_or_else(|| {
                ParseRequestError::ParameterError("No parameters in request".to_string())
            })?;
            let challenge = params
                .get("hub.challenge")
                .ok_or_else(|| ParseRequestError::NotFound("hub.challenge".to_string()))?;

            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", challenge);
            info!("Sending: {response:?}");
            stream.write_all(response.as_bytes())?;

            None
        }

        // Request when a new resource is published
        "POST / HTTP/1.1" => {
            info!("Received POST request");

            let response = "HTTP/1.1 200 OK\r\n\r\n";
            info!("Sending: {response}");
            stream.write_all(response.as_bytes())?;

            //write_to_file(&message)?;
            //info!("New message saved in 'out.txt'");

            let crlf = message
                .find("\r\n\r\n")
                .ok_or(HandleConnectionError::NoBodyError)?;
            let xml = &message.as_str()[crlf..];

            let notification = Notification::try_parse(xml)?;

            Some(notification)
        }

        // Unhandled
        _ => {
            warn!("Unhandled request:");
            dbg!(&message_lines);
            for line in &message_lines {
                warn!("{line}");
            }

            None
        }
    };

    debug!("End of handle_connection: {notification:#?}");
    stream.flush()?;
    Ok(notification)
}

#[allow(unused)]
fn write_to_file(message: &str) -> Result<(), std::io::Error> {
    let mut f = File::options().create(true).append(true).open("out.txt")?;
    writeln!(&mut f, "========== New Message ==========")?;
    writeln!(&mut f, "{message}")?;
    writeln!(&mut f)?;
    Ok(())
}
