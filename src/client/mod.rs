mod client;
mod command;
mod planet;
mod player;
mod squad;

use client::client::Client;

pub fn run() {
    let mut client = Client::new();
    client.run();
}
