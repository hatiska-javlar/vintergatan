mod command;
mod json;
mod planet;
mod player;
mod server;
mod squad;

use server::server::Server;

pub fn run(address: String) {
    let mut server = Server::new();
    server.run(address);
}
