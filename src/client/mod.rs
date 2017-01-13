mod client;
use client::client::Client;

pub fn run() {
    Client { cursor_position: [0.0, 0.0] }.run();
}
