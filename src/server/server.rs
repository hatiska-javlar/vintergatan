use std::cmp::min;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver as ChannelReceiver};
use std::thread;

use time;
use rand::{random, thread_rng, Rng};
use ws::{listen, Sender};

use common::{Id, PlayerId, Position};
use common::websocket_handler::WebsocketHandler;
use server::command::Command;
use server::json;
use server::player::Player;
use server::planet::Planet;
use server::squad::{Squad, SquadState};

enum ServerState {
    Waiting,
    Playing,
    Finished
}

pub struct Server {
    state: ServerState,
    planets: HashMap<Id, Planet>,
    players: HashMap<PlayerId, Player>,
    squads: HashMap<Id, Squad>
}

impl Server {
    pub fn new() -> Self {
        Server {
            state: ServerState::Waiting,
            planets: Self::generate_planets(),
            players: HashMap::new(),
            squads: HashMap::new()
        }
    }

    pub fn run(&mut self, address: String) {
        let (tx, rx) = channel::<Command>();
        thread::spawn(move || listen(&address[..], |sender| WebsocketHandler::new(sender, tx.clone())).unwrap());

        let mut time = time::precise_time_s();
        loop {
            self.process(&rx);

            let dt = time::precise_time_s() - time;
            time = time::precise_time_s();

            self.update(dt);
            self.render();

            thread::sleep(::std::time::Duration::from_millis(100));
        }
    }

    fn process(&mut self, rx: &ChannelReceiver<Command>) {
        while let Ok(command) = rx.try_recv() {
            match command {
                Command::Connect { sender } => self.add_player(sender),

                Command::Ready { sender } => {
                    let player_id = sender.token().0 as PlayerId;
                    self.players.get_mut(&player_id)
                        .map(|player| player.set_ready_state());
                },

                Command::SquadMove { sender, squad_id, x, y, cut_count } => {
                    let player_id = sender.token().0 as PlayerId;

                    let position = Self::find_planet_by_position(&self.planets, Position(x, y))
                        .map_or(Position(x, y), |planet| planet.position());

                    match cut_count {
                        Some(count) => {
                            let squad_data = self.squads
                                .get(&squad_id)
                                .map(|squad| (squad.owner(), squad.life(), squad.position()));

                            if let Some((squad_owner_id, squad_life, squad_position)) = squad_data {
                                let is_owner = squad_owner_id == player_id;
                                let is_can_cut = squad_life > count as f64;

                                if is_owner && is_can_cut {
                                    self.squads.get_mut(&squad_id).map(|squad| {
                                        let life = squad.life();
                                        squad.set_life(life - count as f64);
                                    });

                                    let mut squad = Squad::new(random::<Id>(), player_id, squad_position);
                                    squad.set_life(count as f64);
                                    squad.move_to(position);

                                    self.squads.insert(squad.id(), squad);
                                }
                            }
                        },

                        None => {
                            self.squads.get_mut(&squad_id).map(|squad| {
                                if squad.owner() == player_id {
                                    squad.move_to(position);
                                }
                            });
                        }
                    }
                },

                Command::SquadSpawn { sender, planet_id } => {
                    let player_id = sender.token().0 as PlayerId;

                    if let Some(planet) = self.planets.get(&planet_id) {
                        if let Some(player) = self.players.get_mut(&player_id) {
                            let is_owner = planet.owner().map_or(false, |owner| owner == player_id);

                            let gold = player.gold();
                            let has_gold = gold > 10_f64;

                            if is_owner && has_gold {
                                let squad_id = random::<Id>();
                                let position = planet.position();

                                let mut squad = Squad::new(squad_id, player_id, position);
                                squad.set_state(SquadState::OnOrbit { planet_id: planet.id() });

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

    fn update(&mut self, dt: f64) {
        self.update_server_state();

        if !self.is_playing() {
            return;
        }

        self.update_players(dt);
        self.update_squads(dt);
        self.update_planets();

        self.merge_squads();
        self.update_fight(dt);
    }

    fn update_server_state(&mut self) {
        if self.players.len() == 0 {
            return;
        }

        match self.state {
            ServerState::Waiting => {
                let is_all_ready = self.players
                    .values()
                    .all(|player| player.is_ready());

                if is_all_ready {
                    self.state = ServerState::Playing;

                    for player in self.players.values_mut() {
                        player.set_playing_state();
                    }
                }
            },

            ServerState::Playing => {
                let has_winner = self.players
                    .values()
                    .any(|player| player.is_win());

                if has_winner {
                    self.state = ServerState::Finished
                }
            },

            ServerState::Finished => { }
        }
    }

    fn is_playing(&self) -> bool {
        match self.state {
            ServerState::Playing => true,
            _ => false
        }
    }

    fn update_players(&mut self, dt: f64) {
        for player in self.players.values_mut() {
            let planets_count = self.planets
                .values()
                .filter(|planet| planet.owner().map_or(false, |owner| player.id() == owner))
                .count();

            if planets_count == 0 {
                player.set_loose_state();
            }

            if planets_count == self.planets.len() {
                player.set_win_state();
            }

            let gold = player.gold() + 1_f64 * (planets_count as f64).powf(1_f64 / 3_f64) * dt;
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

    fn merge_squads(&mut self) {
        let merged_squads = self.get_merged_squads();

        for (squad_id, squad_life) in merged_squads {
            match squad_life {
                Some(life) => {
                    self.squads.get_mut(&squad_id)
                        .map(|squad| squad.set_life(life));
                },

                None => {
                    self.squads.remove(&squad_id);
                }
            }
        }
    }

    fn get_merged_squads(&mut self) -> HashMap<Id, Option<f64>> {
        let mut merged_squads = HashMap::new();

        let squads = self.squads
            .values()
            .filter(|squad| squad.is_standing())
            .collect::<Vec<_>>();

        for squad in &squads {
            if merged_squads.contains_key(&squad.id()) {
                continue;
            }

            let other_squads = squads
                .iter()
                .filter(|other_squad| {
                    other_squad.id() != squad.id() &&
                        other_squad.owner() == squad.owner() &&
                        other_squad.position().distance_to(squad.position()) < 5_f64
                })
                .collect::<Vec<_>>();

            if other_squads.is_empty() {
                continue;
            }

            let other_squads_life = other_squads
                .iter()
                .fold(0_f64, |life, squad| life + squad.life());

            merged_squads.insert(squad.id(), Some(squad.life() + other_squads_life));

            for other_squad in other_squads {
                merged_squads.insert(other_squad.id(), None);
            }
        }

        merged_squads
    }

    fn update_fight(&mut self, dt: f64) {
        let hits = self.get_squads_hits();

        for (squad_id, hit) in hits {
            let squad_life = self.squads.get(&squad_id)
                .map(|squad| squad.life());

            if let Some(mut squad_life) = squad_life {
                squad_life -= hit.min(squad_life.ceil()) * dt;
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
            .filter(|squad| squad.is_standing())
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

            let attack = 1_f64 * combat_squad.life().ceil() / attacked_squads.len() as f64;
            for attacked_squad in attacked_squads {
                let hit = hits.get(&attacked_squad.id()).unwrap_or(&0_f64) + attack;
                hits.insert(attacked_squad.id(), hit);
            }
        }

        hits
    }

    fn render(&mut self) {
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
        if self.is_playing() {
            return;
        }

        let player_name = format!("Player #{}", self.players.len() + 1);
        let player = Player::new(sender, player_name);

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