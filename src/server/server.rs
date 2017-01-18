use std::thread;
use std::time::Duration;

use ws::{listen, Sender};

use std::sync::mpsc::channel;
use rand::random;
use planet::PlanetServer;
use std::collections::HashMap;

use server::websocket_listener::WebsocketListener;
use server::world_command::WorldCommand;

pub struct Server {
    pub planets: HashMap<u64, PlanetServer>,
    pub players: Vec<Sender>
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: Self::generate_planets(),
            players: vec![]
        }
    }

    pub fn run(&mut self) {
        let (tx, rx) = channel::<WorldCommand>();
        thread::spawn(move || listen("127.0.0.1:3012", |out| WebsocketListener::new(out, tx.clone())).unwrap());

        loop {
            while let Ok(world_command) = rx.try_recv() {
                match world_command {
                    WorldCommand::Connect { sender } => self.add_player(sender)
                }
            }

            for player in &self.players {
                let s = self.planets.values().map(|planet| {
                    format!("{{\"id\":{},\"x\":{},\"y\":{}}}", planet.id, planet.x, planet.y)
                });

                let x = format!("{{\"planets\": [{}]}}", s.fold("".to_string(), |a, b| if a.len() > 0 { a + &",".to_string() } else { a } + &b.to_string()));

                player.send(x).unwrap();
            }

            thread::sleep(Duration::from_secs(1));
        }
    }

    fn generate_planets() -> HashMap<u64, PlanetServer> {
        let min = -400.0;
        let max = 400.0;

        let mut planets = HashMap::new();
        for _ in 0..20 {
            let id = random::<u64>();
            let planet = PlanetServer {
                id: id,
                x: random::<f64>() * (max - min) + min,
                y: random::<f64>() * (max - min) + min
            };

            planets.insert(id, planet);
        }

        planets
    }

    fn add_player(&mut self, sender: Sender) {
        self.players.push(sender);
    }
}
