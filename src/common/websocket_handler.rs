use std::sync::mpsc::Sender as ChannelSender;
use ws::{
    CloseCode,
    Handler,
    Handshake,
    Result,
    Sender,
    Message
};

use common::to_command::ToCommand;

pub struct WebsocketHandler<C> {
    sender: Sender,
    tx: ChannelSender<C>
}

impl<C> WebsocketHandler<C> {
    pub fn new(sender: Sender, tx: ChannelSender<C>) -> Self {
        WebsocketHandler { sender: sender, tx: tx }
    }
}

impl<C: ToCommand> Handler for WebsocketHandler<C> {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        let connect_command = C::connect(self.sender.clone());
        self.tx.send(connect_command);

        Ok(())
    }

    fn on_message(&mut self, message: Message) -> Result<()> {
        if let Ok(command) = C::process(self.sender.clone(), message) {
            self.tx.send(command);
        }

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        let disconnect_command = C::disconnect(self.sender.clone());
        self.tx.send(disconnect_command);
    }
}