mod client;
mod command;
mod game_event;
mod input_mapping;
mod json;
mod player;
mod squad;
mod waypoint;

use client::client::Client;

pub fn run(address: String) {
    let mut client = Client::new();
    client.run(address);
}