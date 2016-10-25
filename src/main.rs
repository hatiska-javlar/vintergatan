extern crate getopts;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use getopts::Options;
use std::env;
use std::thread;
use std::thread::JoinHandle;

mod server;
use server::Server;

mod client;
use client::Client;

fn run_server(server_address: Option<String>) -> Option<JoinHandle<()>> {
    if server_address.is_some() {
        println!("Starting server on {}", server_address.unwrap_or("127.0.0.1:9999".to_string()));
        let server_thread = thread::spawn(|| {
            (Server {}).run();
        });

        return Some(server_thread);
    }

    return None;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "client", "address and port of server to connect", "127.0.0.1:9999");
    opts.optopt("s", "server", "address and port for server binding", "127.0.0.1:9999");
    opts.optflag("h", "help", "print this help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
        return;
    }

    let server_address = matches.opt_str("s");
    let server_thread = run_server(server_address);

    let client_address = matches.opt_str("c");
    if client_address.is_some() {
        Client {}.run();
    }

    if client_address.is_none() && server_thread.is_some() {
        server_thread.unwrap().join().unwrap();
    }
}
