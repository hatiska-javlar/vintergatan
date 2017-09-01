extern crate getopts;
extern crate piston;
extern crate piston_window;
extern crate opengl_graphics;
extern crate ws;
extern crate rand;
extern crate rustc_serialize;
#[macro_use]
extern crate conrod;
extern crate gfx_device_gl;
extern crate vecmath;
extern crate fps_counter;
#[macro_use]
extern crate glium;
extern crate glium_text_rusttype;
extern crate image;

use getopts::Options;
use std::env;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

mod common;
mod client;
mod server;

fn run_server(server_address: Option<String>) -> Option<JoinHandle<()>> {
    server_address.map(|address| {
        println!("Starting server on {}", address);
        thread::spawn(|| server::run(address))
    })
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
    match client_address {
        Some(address) => {
            thread::sleep(Duration::from_secs(1));
            client::run(address);
        },

        None => {
            server_thread.map(|thread| thread.join());
        }
    }
}