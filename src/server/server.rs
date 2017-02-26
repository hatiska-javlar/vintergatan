use std::cmp::min;
use std::collections::HashMap;
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
use server::json;
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

    pub fn run(&mut self, address: String) {
        let (tx, rx) = channel::<Command>();
        thread::spawn(move || listen(&address[..], |sender| WebsocketHandler::new(sender, tx.clone())).unwrap());

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

                Command::SquadMove { sender, squad_id, x, y } => {
                    let player_id = sender.token().as_usize() as PlayerId;

                    let position = Self::find_planet_by_position(&self.planets, Position(x, y))
                        .map_or(Position(x, y), |planet| planet.position());

                    if let Some(mut squad) = self.squads.get_mut(&squad_id) {
                        if squad.owner() == player_id {
                            squad.move_to(position);
                        }
                    }
                },

                Command::SquadSpawn { sender, planet_id } => {
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
        let dt = args.dt;

        self.update_players(dt);
        self.update_squads(dt);
        self.update_planets();

        self.update_fight(dt);
    }

    fn update_players(&mut self, dt: f64) {
        for player in self.players.values_mut() {
            let planets_count = self.planets
                .values()
                .filter(|planet| planet.owner().map_or(false, |owner| player.id() == owner))
                .count();

            let gold = player.gold() + 1.0 * planets_count as f64 * dt;
            player.set_gold(gold);
        }
    }

    fn update_squads(&mut self, dt: f64) {
        for squad in self.squads.values_mut() {
            match squad.state() {
                SquadState::InSpace => { },

                SquadState::Moving { destination } => {
                    let Position(x, y) = squad.position();
                    let Position(destination_x, destination_y) = destination;

                    let target = (destination_x - x, destination_y - y);
                    let distance = (target.0.powi(2) + target.1.powi(2)).sqrt();

                    let max_step_distance = 50_f64 * dt;

                    if distance < max_step_distance {
                        squad.set_position(destination);

                        let state = Self::find_planet_by_position(&self.planets, destination)
                            .map_or(SquadState::InSpace, |planet| SquadState::OnOrbit { planet_id: planet.id() });

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

                SquadState::OnOrbit { .. } => { }
            }
        }
    }

    fn update_planets(&mut self) {
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

    fn update_fight(&mut self, dt: f64) {
        let hits = self.get_squads_hits();

        for (squad_id, hit) in hits {
            let squad_life = self.squads.get(&squad_id)
                .map(|squad| squad.life());

            if let Some(mut squad_life) = squad_life {
                squad_life -= hit * dt;
                if squad_life < 0_f64 {
                    self.squads.remove(&squad_id);
                } else {
                    self.squads.get_mut(&squad_id)
                        .map(|squad| squad.set_life(squad_life));
                }
            }
        }
    }

    fn get_squads_hits(&self) -> HashMap<Id, f64> {
        let mut hits: HashMap<Id, f64> = HashMap::new();

        let combat_squads = self.squads
            .values()
            .filter(|squad| {
                match squad.state() {
                    SquadState::InSpace | SquadState::OnOrbit { .. } => true,
                    SquadState::Moving { .. } => false
                }
            })
            .collect::<Vec<_>>();

        for combat_squad in &combat_squads {
            let attacked_squads = combat_squads
                .iter()
                .filter(|attacked_squad| {
                    attacked_squad.owner() != combat_squad.owner() &&
                        attacked_squad.id() != combat_squad.id() &&
                        attacked_squad.position().distance_to(combat_squad.position()) < 10_f64
                })
                .collect::<Vec<_>>();

            let attack = 1_f64 / attacked_squads.len() as f64;
            for attacked_squad in attacked_squads {
                let hit = hits.get(&attacked_squad.id()).unwrap_or(&0_f64) + attack;
                hits.insert(attacked_squad.id(), hit);
            }
        }

        hits
    }

    fn render(&mut self, args: &RenderArgs) {
        let planets_json = json::format_planets(&self.planets);
        let players_json = json::format_players(&self.players);
        let squads_json = json::format_squads(&self.squads);

        for player in self.players.values() {
            let process_command_json = json::format_process_command(
                player,
                &planets_json,
                &players_json,
                &squads_json
            );

            player.send(process_command_json);
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
}