mod client;
mod command;
mod planet;

use client::client::Client;

pub fn run() {
    let mut client = Client::new();
    client.run();
}
