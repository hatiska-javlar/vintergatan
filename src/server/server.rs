use std::thread;
use std::time::Duration;
use std::cmp::min;

use ws::{listen, Sender};

use std::sync::mpsc::channel;
use rand::{thread_rng, Rng, random};
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
        let HALF_WINDOW_WIDTH = 640;
        let HALF_WINDOW_HEIGHT = 400;
        let PLANETS_DENSITY = 0.9;
        let GRID_STEP = 20;

        let grid_x_start = -HALF_WINDOW_WIDTH + GRID_STEP;
        let grid_x_end = HALF_WINDOW_WIDTH - GRID_STEP;
        let grid_y_start = -HALF_WINDOW_HEIGHT + GRID_STEP;
        let grid_y_end = HALF_WINDOW_HEIGHT - GRID_STEP;

        let short_side = min(HALF_WINDOW_WIDTH, HALF_WINDOW_HEIGHT);
        let content_short_side = (short_side - GRID_STEP) * 2;
        let max_planets_count = content_short_side / GRID_STEP / 2;

        let mut grid_x_coordinates = vec![];
        let mut grid_y_coordinates = vec![];

        for x in grid_x_start / GRID_STEP .. grid_x_end / GRID_STEP {
            grid_x_coordinates.push((x * GRID_STEP) as f64);
        }
        for y in grid_y_start / GRID_STEP .. grid_y_end / GRID_STEP {
            grid_y_coordinates.push((y * GRID_STEP) as f64);
        }

        thread_rng().shuffle(&mut grid_x_coordinates);
        thread_rng().shuffle(&mut grid_y_coordinates);

        let mut planets = HashMap::new();
        for i in 0 .. (max_planets_count as f64 * PLANETS_DENSITY) as usize {
            let id = random::<u64>();
            let planet = PlanetServer {
                id: id,
                x: grid_x_coordinates[i],
                y: grid_y_coordinates[i]
            };

            planets.insert(id, planet);
        }

        planets
    }

    fn add_player(&mut self, sender: Sender) {
        self.players.push(sender);
    }
}
