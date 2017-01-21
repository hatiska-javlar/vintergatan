mod client;
mod command;

use client::client::Client;

pub fn run() {
    let mut client = Client::new();
    client.run();
}
