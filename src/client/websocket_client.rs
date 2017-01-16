use std::sync::mpsc::Sender as ChannelSender;
use rustc_serialize::json::{Json, Object};
use ws::{Sender, Handler, Handshake, Message, CloseCode, Result};
use client::client_command::ClientCommand;
use planet::PlanetClient;

pub struct WebsocketClient {
    out: Sender,
    tx: ChannelSender<ClientCommand>
}

impl WebsocketClient {
    pub fn new(out: Sender, tx: ChannelSender<ClientCommand>) -> Self {
        WebsocketClient {
            out: out,
            tx: tx
        }
    }
}

impl Handler for WebsocketClient {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("New connection is opened to {}", shake.peer_addr.unwrap());

        Ok(())
    }

    fn on_message(&mut self, message: Message) -> Result<()> {
        let raw = message.into_text().unwrap_or("".to_string());
        let parsed = Json::from_str(&raw).unwrap_or(Json::Object(Object::new()));

        let empty_json_object = Object::new();
        let params = parsed.as_object().unwrap_or(&empty_json_object);

        if let Some(planets_json) = params.get("planets") {
            let planets = planets_json.as_array().unwrap().into_iter().map(|planet_json| {
                PlanetClient {
                    id: planet_json.as_object().unwrap().get("id").unwrap().as_u64().unwrap(),
                    x: planet_json.as_object().unwrap().get("x").unwrap().as_f64().unwrap(),
                    y: planet_json.as_object().unwrap().get("y").unwrap().as_f64().unwrap(),
                    color: [0.125490196, 0.752941176, 0.870588235, 1.0],
                    size: 10.0
                }
            }).collect();

            self.tx.send(ClientCommand::Process {
                planets: planets
            }).unwrap();
        }

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Connection closed code = {:?}, reason = {}", code, reason);
    }
}
