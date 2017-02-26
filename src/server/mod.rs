mod command;
mod json;
mod planet;
mod player;
mod server;
mod squad;

use server::server::Server;

pub fn run() {
    let mut server = Server::new();
    server.run();
}
