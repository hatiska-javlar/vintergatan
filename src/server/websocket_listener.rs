use std::sync::mpsc::Sender as ChannelSender;
use ws::{Sender, Handler, Result, Message, Handshake, CloseCode};

use server::world_command::WorldCommand;

pub struct WebsocketListener {
    out: Sender,
    tx: ChannelSender<WorldCommand>
}

impl WebsocketListener {
    pub fn new(out: Sender, tx: ChannelSender<WorldCommand>) -> Self {
        WebsocketListener {
            out: out,
            tx: tx
        }
    }
}

impl Handler for WebsocketListener {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("New connection is opened from {}", shake.peer_addr.unwrap());

        self.tx.send(WorldCommand::Connect {sender: self.out.clone()});

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
