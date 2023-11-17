use std::{
    fs::File,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

mod config;
mod error;
mod notification;
mod parse;
mod pool;
mod prelude;
mod request;
mod response;

use config::Config;
use log::{debug, error, info, warn};
use pool::ThreadPool;
use prelude::*;

type BoxedError = Box<dyn std::error::Error>;

const CONFIG_PATH: &str = "webhook.toml";

pub struct HookListener {
    listener: TcpListener,
    pool: ThreadPool,
    config: Config,
    sender: mpsc::Sender<Notification>,
}

//TODO: Handle resubscription before the expiration delay (5 days for youtube)
//TODO: Write documentation
//TODO: Write more custom errors

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
    pub fn new() -> Result<(Self, mpsc::Receiver<Notification>), BoxedError> {
        let config = Config::from_file(CONFIG_PATH)?;
        info!("Config loaded");
        let addr = format!("{}:{}", config.server.host, config.server.port);

        let (sender, receiver) = mpsc::channel();

        let listener = Self {
            listener: TcpListener::bind(&addr)?,
            pool: ThreadPool::new(4),
            config,
            sender,
        };
        info!("TCPListener binded to {}", &addr);

        Ok((listener, receiver))
    }

    /// Start listening for incoming streams.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Create the listener
    /// let (listener, receiver) = HookListener::new()?;
    ///
    /// // Start the listener in the background
    /// std::thead::spawn(|| {
    ///     if let Err(e) = listener.listen() {
    ///     println!("error: {e}");
    ///     }
    /// });
    ///
    /// // Wait for notification
    /// loop {
    ///     if let Ok(message) = receiver.try_recv() {
    ///         println!("New video: {message:?}");
    ///     }
    /// }
    ///
    /// ```
    pub fn listen(&self) -> Result<(), BoxedError> {
        info!("Start listener with {} threads", self.pool.num_threads());

        for stream in self.listener.incoming() {
            let stream = stream?;
            let sender = self.sender.clone();

            self.pool.execute(|| {
                if let Err(e) = handle_connection(stream, sender) {
                    error!("Connection handle: {e}");
                };
            });
        }

        Ok(())
    }

    /// Reload the webhook.toml configuration file.
    ///
    /// # Panics:
    ///
    /// Configuration file does not exist or is malformed.
    pub fn reload_config(&mut self) -> Result<(), BoxedError> {
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
    pub fn subscribe(&self, id: impl AsRef<str>, mode: impl AsRef<str>) -> Result<(), BoxedError> {
        let id = id.as_ref();
        let mode = mode.as_ref();

        info!("Initiating {mode} request with id: {id}");

        if !(mode == "subscribe" || mode == "unsubscribe") {
            return Err(Box::new(MyError {
                source: CallbackError::SubscriptionMode,
            }));
        }

        let subscription = self
            .config
            .youtube
            .as_ref()
            .ok_or("Youtube configuration not found")?;
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

        // Send the stream to handle_connection() to manage the answer
        //handle_connection(stream)?;

        Ok(())
    }
}

const BUF_SIZE: usize = 1024;

fn handle_connection(
    mut stream: TcpStream,
    sender: mpsc::Sender<Notification>,
) -> Result<(), BoxedError> {
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
            }
        }
    }
    info!("Received {n_bytes} bytes");

    received.retain(|&b| b != 0);
    let message = String::from_utf8(received)?;
    let message_lines: Vec<&str> = message.trim().lines().collect();

    debug!("Message:\n{message}");

    let request_line = message_lines.first().ok_or("Message is empty")?;

    match *request_line {
        // The hub send 202 Accepted if the subscription request is accepted
        "HTTP/1.1 202 ACCEPTED" => {
            info!("Subscription accepted");
        }

        // The hub senf a GET request for the verification of intent
        // The scubscriber must answer with a 2XX status code and echo the hub.challenge value
        request if request.starts_with("GET") => {
            info!("Received GET request");
            let request_line = request::parse_request_line(request)?;
            let params = request_line
                .params
                .ok_or("Verification of intent: no parameters found")?;
            let challenge = params
                .get("hub.challenge")
                .ok_or("Verification of intent: no hub.challenge found")?;

            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", challenge);
            info!("Sending: {response:?}");
            stream.write_all(response.as_bytes())?;
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
                .ok_or("No empty line found after headers")?;
            let xml = &message.as_str()[crlf..];

            let t = std::time::Instant::now();
            let _result = Notification::try_parse(xml)?;
            debug!("quick_xml: {} µs", t.elapsed().as_micros());

            let t = std::time::Instant::now();
            let result = Notification::try_my_parse(xml)?;
            debug!("mine: {} µs", t.elapsed().as_micros());

            sender.send(result)?;
        }

        // Unhandled
        _ => {
            warn!("Unhandled request:");
            dbg!(&message_lines);
            for line in &message_lines {
                warn!("{line}");
            }
        }
    }

    stream.flush()?;
    Ok(())
}

#[allow(unused)]
fn write_to_file(message: &str) -> Result<(), BoxedError> {
    let mut f = File::options().create(true).append(true).open("out.txt")?;
    writeln!(&mut f, "========== New Message ==========")?;
    writeln!(&mut f, "{message}")?;
    writeln!(&mut f)?;
    Ok(())
}
