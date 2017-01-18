mod server;
mod websocket_listener;
mod world_command;

use server::server::Server;

pub fn run() {
    let mut server = Server::new();
    server.run();
}
