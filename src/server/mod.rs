mod command;
mod json;
mod player;
mod server;
mod squad;
mod waypoint;

use server::server::Server;

pub fn run(address: String) {
    let mut server = Server::new();
    server.run(address);
}
