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

use common::{Id, PlayerId, Position};
use common::websocket_handler::WebsocketHandler;
use server::command::Command;
use server::player::Player;
use server::planet::Planet;
use server::squad::{Squad, SquadState};

pub struct Server {
    planets: HashMap<Id, Planet>,
    players: HashMap<PlayerId, Player>,
    squads: HashMap<Id, Squad>
}

impl Server {
    pub fn new() -> Self {
        Server {
            planets: Self::generate_planets(),
            players: HashMap::new(),
            squads: HashMap::new()
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

                Command::Move { sender, squad_id, x, y } => {
                    let player_id = sender.token().as_usize() as PlayerId;

                    let position = Self::find_planet_by_position(&self.planets, Position(x, y))
                        .map_or(Position(x, y), |planet| planet.position());

                    if let Some(mut squad) = self.squads.get_mut(&squad_id) {
                        if squad.owner() == player_id {
                            squad.move_to(position);
                        }
                    }
                },

                Command::Spawn { sender, planet_id } => {
                    let player_id = sender.token().as_usize() as PlayerId;

                    if let Some(planet) = self.planets.get(&planet_id) {
                        if let Some(player) = self.players.get_mut(&player_id) {
                            let is_owner = planet.owner().map_or(false, |owner| owner == player_id);

                            let gold = player.gold();
                            let has_gold = gold > 10_f64;

                            if is_owner && has_gold {
                                let squad_id = random::<Id>();
                                let position = planet.position();

                                let squad = Squad::new(squad_id, player_id, position);
                                self.squads.insert(squad_id, squad);

                                player.set_gold(gold - 10.0);
                            }
                        }
                    }
                },
                _ => { }
            }
        }
    }

    fn update(&mut self, args: &UpdateArgs) {
        for (player_id, player) in &mut self.players {
            let planets_count = self.planets
                .values()
                .filter(|planet| planet.owner().map_or(false, |owner| *player_id == owner))
                .count();

            let gold = player.gold() + 1.0 * planets_count as f64 * args.dt;
            player.set_gold(gold);
        }

        for squad in self.squads.values_mut() {
            match squad.state() {
                SquadState::Pending => {},

                SquadState::Moving { destination } => {
                    let Position(x, y) = squad.position();
                    let Position(destination_x, destination_y) = destination;

                    let target = (destination_x - x, destination_y - y);
                    let distance = (target.0.powi(2) + target.1.powi(2)).sqrt();

                    let max_step_distance = 50_f64 * args.dt;

                    if distance < max_step_distance {
                        squad.set_position(destination);

                        let state = Self::find_planet_by_position(&self.planets, destination)
                            .map_or(SquadState::Pending, |planet| SquadState::OnOrbit { planet_id: planet.id() });

                        squad.set_state(state);
                    } else {
                        let direction = (target.0 / distance, target.1 / distance);
                        let position = Position(
                            x + max_step_distance * direction.0,
                            y + max_step_distance * direction.1
                        );

                        squad.set_position(position);
                    }
                },

                SquadState::OnOrbit { .. } => {}
            }
        }

        for planet in self.planets.values_mut() {
            let squads_on_orbit = self.squads
                .values()
                .filter(|squad| squad.is_on_orbit(planet.id()))
                .collect::<Vec<_>>();

            if let Some(first_squad) = squads_on_orbit.first() {
                let owner = first_squad.owner();
                if squads_on_orbit.iter().all(|squad| squad.owner() == owner) {
                    planet.set_owner(Some(owner));
                }
            }
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        let planets_json = self.format_planets_as_json();
        let players_json = self.format_players_as_json();
        let squads_json = self.format_squads_as_json();

        let players = self.players.values();
        for player in players {
            let message_json = format!(
                "{{\"planets\":{},\"players\":{},\"squads\":{},\"id\":{},\"gold\":{}}}",
                planets_json,
                players_json,
                squads_json,
                player.id(),
                player.gold()
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

        let player_id = player.id();
        players.insert(player_id, player);

        let planet = self.planets.values_mut().find(|planet| planet.owner().is_none());
        if let Some(planet) = planet {
            planet.set_owner(Some(player_id));
        }
    }

    fn find_planet_by_position(planets: &HashMap<Id, Planet>, position: Position) -> Option<&Planet> {
        let Position(x, y) = position;

        planets
            .values()
            .find(|planet| {
                let Position(planet_x, planet_y) = planet.position();
                ((planet_x - x).powi(2) + (planet_y - y).powi(2)).sqrt() < 10_f64
            })
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

    fn format_squads_as_json(&self) -> String {
        let formatted_squads = self.squads
            .values()
            .map(|squad| {
                let id = squad.id();
                let owner = squad.owner();
                let Position(x, y) = squad.position();
                let count = squad.count();

                format!("{{\"id\":{},\"owner\":{},\"x\":{},\"y\":{},\"count\":{}}}", id, owner, x, y, count)
            })
            .collect::<Vec<String>>();

        format!("[{}]", Self::join(formatted_squads, ","))
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
