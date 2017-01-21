mod command;
mod server;

use server::server::Server;

pub fn run() {
    let mut server = Server::new();
    server.run();
}
