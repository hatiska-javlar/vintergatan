use std::thread;
use std::time::Duration;

use ws::{listen, Sender, Handler, Result, Message, Handshake, CloseCode};

pub struct Server {}

impl Server {
    pub fn run(&self) {
        thread::spawn(move || listen("127.0.0.1:3012", |out| WebsocketListener { out: out }).unwrap());

        loop {
            println!("Server");
            thread::sleep(Duration::from_secs(1));
        }
    }
}

struct WebsocketListener {
    out: Sender
}

impl Handler for WebsocketListener {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("New connection is opened from {}", shake.peer_addr.unwrap());

        Ok(())
    }

    fn on_message(&mut self, message: Message) -> Result<()> {
        let raw = message.into_text().unwrap_or("".to_string());
        println!("{}", raw);

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Connection closed code = {:?}, reason = {}", code, reason);
    }
}
