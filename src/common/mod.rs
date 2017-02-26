use rustc_serialize::json::ParserError;

use ws::Error;

pub mod to_command;
pub mod utils;
pub mod websocket_handler;

pub type Id = u64;
pub type PlayerId = usize;

#[derive(Copy, Clone)]
pub struct Position(pub f64, pub f64);

#[derive(Debug)]
pub enum ParseCommandError {
    ParserError(ParserError),
    BrokenCommand(Error),
    MissedProperty(String),
    IncompatibleType(String),
    UnsupportedAction
}

pub type ParseCommandResult<T> = Result<T, ParseCommandError>;