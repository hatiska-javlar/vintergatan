mod client;
mod camera;
mod command;
mod game_cursor;
mod game_event;
mod game_ui;
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