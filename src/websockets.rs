use auth;
use errors::*;
use events::*;
use tungstenite::stream::MaybeTlsStream;
use url::Url;

use tungstenite::handshake::client::Response;
use tungstenite::protocol::WebSocket;
use tungstenite::{connect, Message};

use std::net::TcpStream;
use std::sync::mpsc::{self, channel};

static INFO: &'static str = "info";
static SUBSCRIBED: &'static str = "subscribed";
static AUTH: &'static str = "auth";
static WEBSOCKET_URL: &'static str = "wss://api.bitfinex.com/ws/2";
static DEAD_MAN_SWITCH_FLAG: u8 = 4;

pub trait EventHandler {
    fn on_connect(&mut self, event: NotificationEvent);
    fn on_auth(&mut self, event: NotificationEvent);
    fn on_subscribed(&mut self, event: NotificationEvent);
    fn on_data_event(&mut self, event: DataEvent);
    fn on_error(&mut self, message: Error);
}

pub enum EventType {
    Funding,
    Trading,
}

#[derive(Debug)]
pub enum WsMessage {
    Close,
    Text(String),
}

pub struct WebSockets {
    pub socket: Option<(WebSocket<MaybeTlsStream<TcpStream>>, Response)>,
    sender: Sender,
    pub rx: mpsc::Receiver<WsMessage>,
}

impl WebSockets {
    pub fn new() -> WebSockets {
        let (tx, rx) = channel::<WsMessage>();
        let sender = Sender { tx: tx };

        WebSockets {
            socket: None,
            sender: sender,
            rx: rx,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        let wss: String = format!("{}", WEBSOCKET_URL);
        let url = Url::parse(&wss)?;

        match connect(url) {
            Ok(answer) => {
                self.socket = Some(answer);
                Ok(())
            }
            Err(e) => {
                bail!(format!("Error during handshake {}", e))
            }
        }
    }

    /// Authenticates the connection.
    ///
    /// The connection will be authenticated until it is disconnected.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key
    /// * `api_secret` - The API secret
    /// * `dms` - Whether the dead man switch is enabled. If true, all account orders will be
    ///           cancelled when the socket is closed.
    pub fn auth<S>(&mut self, api_key: S, api_secret: S, dms: bool, filters: &[&str]) -> Result<()>
    where
        S: AsRef<str>,
    {

        match self.socket {
            None => return Err(crate::errors::ErrorKind::Internal("authenticate first".to_string()).into()),
            Some((ref mut socket, _)) =>{
                let nonce = auth::generate_nonce()?;
                let auth_payload = format!("AUTH{}", nonce);
                let signature =
                    auth::sign_payload(api_secret.as_ref().as_bytes(), auth_payload.as_bytes())?;

                let msg = json!({
                    "event": "auth",
                    "apiKey": api_key.as_ref(),
                    "authSig": signature,
                    "authNonce": nonce,
                    "authPayload": auth_payload,
                    "dms": if dms {Some(DEAD_MAN_SWITCH_FLAG)} else {None},
                    "filters": filters,
                });

                socket.write_message(Message::Text(msg.to_string()))?;
            }
        }

        Ok(())
    }

    pub fn subscribe_ticker<S>(&mut self, symbol: S, et: EventType)
    where
        S: Into<String>,
    {
        let local_symbol = self.format_symbol(symbol.into(), et);
        let msg = json!({"event": "subscribe", "channel": "ticker", "symbol": local_symbol });

        if let Err(error_msg) = self.sender.send(&msg.to_string()) {
            self.error_hander(error_msg);
        }
    }

    pub fn subscribe_trades<S>(&mut self, symbol: S, et: EventType)
    where
        S: Into<String>,
    {
        let local_symbol = self.format_symbol(symbol.into(), et);
        let msg = json!({"event": "subscribe", "channel": "trades", "symbol": local_symbol });

        if let Err(error_msg) = self.sender.send(&msg.to_string()) {
            self.error_hander(error_msg);
        }
    }

    pub fn subscribe_candles<S>(&mut self, symbol: S, timeframe: S)
    where
        S: Into<String>,
    {
        let key: String = format!("trade:{}:t{}", timeframe.into(), symbol.into());
        let msg = json!({"event": "subscribe", "channel": "candles", "key": key });

        if let Err(error_msg) = self.sender.send(&msg.to_string()) {
            self.error_hander(error_msg);
        }
    }

    pub fn subscribe_books<S, P, F>(&mut self, symbol: S, et: EventType, prec: P, freq: F, len: u32)
    where
        S: Into<String>,
        P: Into<String>,
        F: Into<String>,
    {
        let msg = json!(
        {
            "event": "subscribe",
            "channel": "book",
            "symbol": self.format_symbol(symbol.into(), et),
            "prec": prec.into(),
            "freq": freq.into(),
            "len": len
        });

        if let Err(error_msg) = self.sender.send(&msg.to_string()) {
            self.error_hander(error_msg);
        }
    }

    pub fn subscribe_raw_books<S>(&mut self, symbol: S, et: EventType)
    where
        S: Into<String>,
    {
        let msg = json!(
        {
            "event": "subscribe",
            "channel": "book",
            "prec": "R0",
            "pair": self.format_symbol(symbol.into(), et)
        });

        if let Err(error_msg) = self.sender.send(&msg.to_string()) {
            self.error_hander(error_msg);
        }
    }

    fn error_hander(&mut self, error_msg: Error) {
        println!("Error {error_msg:?}")
        // if let Some(ref mut h) = self.event_handler {
        //     h.on_error(error_msg);
        // }
    }

    fn format_symbol(&mut self, symbol: String, et: EventType) -> String {
        let local_symbol = match et {
            EventType::Funding => format!("f{}", symbol),
            EventType::Trading => format!("t{}", symbol),
        };

        local_symbol
    }

    pub fn event_loop(&mut self) -> Result<()> {
        loop {
            if let Some(ref mut socket) = self.socket {
                loop {
                    match self.rx.try_recv() {
                        Ok(msg) => match msg {
                            WsMessage::Text(text) => {
                                socket.0.write_message(Message::Text(text))?;
                            }
                            WsMessage::Close => {
                                return socket.0.close(None).map_err(|e| e.into());
                            }
                        },
                        Err(mpsc::TryRecvError::Disconnected) => {
                            bail!("Disconnected")
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                    }
                }

                let message = socket.0.read_message()?;
                match message {
                    Message::Text(text) => {
                        if text.find(INFO) != None {
                            let event: NotificationEvent = serde_json::from_str(&text)?;
                        } else if text.find(SUBSCRIBED) != None {
                            let event: NotificationEvent = serde_json::from_str(&text)?;
                            println!("{:?}", event);
                        } else if text.find(AUTH).is_some() {
                            let event: NotificationEvent = serde_json::from_str(&text)?;
                            println!("{:?}", event);
                        } else {
                            let event: DataEvent = serde_json::from_str(&text)?;
                            if let DataEvent::HeartbeatEvent(_a, _b) = event {
                                continue;
                            } else {
                                println!("{:?}", event);
                            }
                        }
                    }
                    Message::Binary(_) => {}
                    Message::Ping(_) | Message::Pong(_) => {}
                    Message::Close(e) => {
                        bail!(format!("Disconnected {:?}", e));
                    }
                    _ => {
                        bail!("Should never happen");
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<WsMessage>,
}

impl Sender {
    pub fn send(&self, raw: &str) -> Result<()> {
        self.tx
            .send(WsMessage::Text(raw.to_string()))
            .map_err(|e| Error::with_chain(e, "Not able to send a message"))?;
        Ok(())
    }

    pub fn shutdown(&self) -> Result<()> {
        self.tx
            .send(WsMessage::Close)
            .map_err(|e| Error::with_chain(e, "Error during shutdown"))
    }
}
