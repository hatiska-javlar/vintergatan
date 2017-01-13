use std::collections::HashMap;

mod server;
use server::server::Server;

pub fn run() {
    Server { planets: HashMap::new(), players: vec![] }.run();
}
