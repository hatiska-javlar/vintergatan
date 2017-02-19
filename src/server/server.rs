use std::cmp::min;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::mpsc::{
    channel,
    Receiver as ChannelReceiver
};
use std::thread;

use piston::event_loop::{
    Events,
    EventLoop
};
use piston::input::{
    RenderArgs,
    RenderEvent,
    UpdateArgs,
    UpdateEvent
};
use piston::window::{
    NoWindow,
    WindowSettings
};
use rand::{
    random,
    thread_rng,
    Rng
};
use ws::{
    listen,
    Sender
};

use common::id::Id;
use common::position::Position;
use common::websocket_handler::WebsocketHandler;
use server::command::Command;
use server::player::{Player, PlayerId};
use server::planet::Planet;

pub struct Server {
    planets: HashMap<Id, Planet>,
    players: HashMap<PlayerId, Player>
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: Self::generate_planets(),
            players: HashMap::new()
        }
    }

    pub fn run(&mut self) {
        let (tx, rx) = channel::<Command>();
        thread::spawn(move || listen("127.0.0.1:3012", |sender| WebsocketHandler::new(sender, tx.clone())).unwrap());

        let window_settings = WindowSettings::new("Vintergatan game server", [1, 1]);
        let mut no_window = NoWindow::new(&window_settings);

        let mut events = no_window
            .events()
            .ups(10)
            .max_fps(10);

        while let Some(e) = events.next(&mut no_window) {
            if let Some(u) = e.update_args() {
                self.process(&rx);
                self.update(&u);
            }

            if let Some(r) = e.render_args() {
                self.render(&r);
            }
        }
    }

    fn process(&mut self, rx: &ChannelReceiver<Command>) {
        while let Ok(command) = rx.try_recv() {
            match command {
                Command::Connect { sender } => self.add_player(sender),
                _ => { }
            }
        }
    }

    fn update(&mut self, args: &UpdateArgs) { }

    fn render(&mut self, args: &RenderArgs) {
        let planets_json = self.format_planets_as_json();
        let players_json = self.format_players_as_json();

        let players = self.players.values();
        for player in players {
            let message_json = format!(
                "{{\"planets\":{},\"players\":{},\"id\":{}}}",
                planets_json,
                players_json,
                player.id()
            );

            player.send(message_json);
        }
    }

    fn generate_planets() -> HashMap<Id, Planet> {
        let half_window_width = 640;
        let half_window_height = 400;
        let planets_density = 1.0;
        let grid_step = 40;

        let grid_x_start = -half_window_width + grid_step;
        let grid_x_end = half_window_width - grid_step;
        let grid_y_start = -half_window_height + grid_step;
        let grid_y_end = half_window_height - grid_step;

        let short_side = min(half_window_width, half_window_height);
        let content_short_side = (short_side - grid_step) * 2;
        let max_planets_count = content_short_side / grid_step;

        let mut grid_x_coordinates = vec![];
        let mut grid_y_coordinates = vec![];

        for x in (grid_x_start / grid_step)..(grid_x_end / grid_step) {
            grid_x_coordinates.push((x * grid_step) as f64);
        }
        for y in (grid_y_start / grid_step)..(grid_y_end / grid_step) {
            grid_y_coordinates.push((y * grid_step) as f64);
        }

        thread_rng().shuffle(&mut grid_x_coordinates);
        thread_rng().shuffle(&mut grid_y_coordinates);

        let mut planets = HashMap::new();
        for i in 0..(max_planets_count as f64 * planets_density) as usize {
            let id = random::<u64>();
            let position = Position(grid_x_coordinates[i], grid_y_coordinates[i]);

            let planet = Planet::new(id, position);
            planets.insert(id, planet);
        }

        planets
    }

    fn add_player(&mut self, sender: Sender) {
        let player = Player::new(sender);

        let ref mut players = self.players;
        players.insert(player.id(), player);
    }

    fn format_planets_as_json(&self) -> String {
        let formatted_planets = self.planets
            .values()
            .map(|planet| {
                let id = planet.id();
                let Position(x, y) = planet.position();
                let owner = Self::format_option_as_json(planet.owner());

                format!("{{\"id\":{},\"x\":{},\"y\":{},\"owner\":{}}}", id, x, y, owner)
            })
            .collect::<Vec<String>>();

        format!("[{}]", Self::join(formatted_planets, ","))
    }

    fn format_players_as_json(&self) -> String {
        let formatted_players = self.players
            .values()
            .map(|player| format!("{{\"id\":{}}}", player.id()))
            .collect::<Vec<String>>();

        format!("[{}]", Self::join(formatted_players, ","))
    }

    fn format_option_as_json<T: Display>(option: Option<T>) -> String {
        if let Some(value) = option {
            format!("{}", value)
        } else {
            "null".to_string()
        }
    }

    fn join<S: ToString>(vec: Vec<S>, sep: &str) -> String {
        vec
            .iter()
            .fold("".to_string(), |a, b| if a.len() > 0 { a + sep } else { a } + &b.to_string())
    }
}
