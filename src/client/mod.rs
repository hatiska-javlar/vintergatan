mod client;
mod websocket_client;
mod client_command;

use client::client::Client;

pub fn run() {
    let mut client = Client::new();
    client.run();
}
