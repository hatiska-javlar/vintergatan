use std::thread;
use std::time::Duration;

pub struct Server {}

impl Server {
    pub fn run(&self) {
        loop {
            println!("Server");
            thread::sleep(Duration::from_secs(1));
        }
    }
}
