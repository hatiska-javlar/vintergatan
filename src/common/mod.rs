pub mod to_command;
pub mod websocket_handler;

pub type Id = u64;
pub type PlayerId = usize;

#[derive(Copy, Clone)]
pub struct Position(pub f64, pub f64);