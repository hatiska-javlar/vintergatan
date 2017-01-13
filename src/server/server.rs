use std::thread;
use std::time::Duration;

use ws::{listen, Sender, Handler, Result, Message, Handshake, CloseCode};

use std::sync::mpsc::Sender as ChanelSender;
use std::sync::mpsc::channel;
use rand::random;
use planet::PlanetServer;
use world_command::WorldCommand;
use std::collections::HashMap;

pub struct Server {
    pub planets: HashMap<u64, PlanetServer>,
    pub players: Vec<Sender>
}

impl Server {
    pub fn run(&mut self) {
        let min = -400.0;
        let max = 400.0;

        self.planets = HashMap::new();
        for _ in 0..20 {
            let id = random::<u64>();
            let planet = PlanetServer {
                id: id,
                x: random::<f64>() * (max - min) + min,
                y: random::<f64>() * (max - min) + min
            };

            self.planets.insert(id, planet);
        }

        self.players = vec![];

        let (tx, rx) = channel::<WorldCommand>();

        thread::spawn(move || listen("127.0.0.1:3012", |out| WebsocketListener { out: out, tx: tx.clone() }).unwrap());

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

    fn add_player(&mut self, sender: Sender) {
        self.players.push(sender);
    }
}

struct WebsocketListener {
    out: Sender,
    tx: ChanelSender<WorldCommand>
}

impl Handler for WebsocketListener {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("New connection is opened from {}", shake.peer_addr.unwrap());

        self.tx.send(WorldCommand::Connect {sender: self.out.clone()});

        Ok(())
    }

    fn on_message(&mut self, message: Message) -> Result<()> {
        let raw = message.into_text().unwrap_or("".to_string());
        println!("{}", raw);

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Connection closed code = {:?}, reason = {}", code, reason);
    }
}
