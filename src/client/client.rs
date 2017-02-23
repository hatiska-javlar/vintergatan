use std::thread;
use std::collections::HashMap;
use std::env::current_dir;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::window::{Window, Size};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use ws::{connect, Sender};
use std::sync::mpsc::{channel, Receiver as ChannelReceiver};

use client::command::Command;
use client::data::{PlanetData, PlayerData, SquadData};
use client::planet::Planet;
use client::player::Player;
use client::squad::Squad;
use common::id::Id;
use common::PlayerId;
use common::position::Position;
use common::websocket_handler::WebsocketHandler;


pub struct Client {
    window: GlutinWindow,
    gl: GlGraphics,
    glyph_cache: GlyphCache<'static>,
    rx: Option<ChannelReceiver<Command>>,

    cursor_position: [f64; 2],
    planets: HashMap<Id, Planet>,
    players: HashMap<PlayerId, Player>,
    gold: f64,

    current_selected_planet: Option<Id>,
    current_selected_squad: Option<Id>,
    sender: Option<Sender>
}

impl Client {
    pub fn new() -> Self {
        let opengl = OpenGL::V3_2;

        let window = WindowSettings::new("Vintergatan game", [1280, 800])
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let gl = GlGraphics::new(opengl);

        Client {
            window: window,
            gl: gl,
            glyph_cache: GlyphCache::new(current_dir().unwrap().join("assets/Exo2-Regular.ttf")).unwrap(),
            rx: None,

            cursor_position: [0f64, 0f64],
            planets: HashMap::new(),
            players: HashMap::new(),
            gold: 0.0,

            current_selected_planet: None,
            current_selected_squad: None,
            sender: None
        }
    }

    pub fn run(&mut self) {
        let (tx, rx) = channel::<Command>();

        self.rx = Some(rx);
        thread::spawn(move || connect("ws://127.0.0.1:3012", |sender| WebsocketHandler::new(sender, tx.clone())).unwrap());

        let mut events = self.window.events();
        while let Some(e) = events.next(&mut self.window) {
            if let Some(r) = e.render_args() {
                self.render(&r);
            }

            if let Some(u) = e.update_args() {
                self.update(&u);
            }

            if let Some(m) = e.mouse_cursor_args() {
                self.cursor_position = m;
            }

            match e {
                Event::Input(Input::Press(Button::Mouse(MouseButton::Left))) => {
                    let cursor_position = self.cursor_position;
                    self.select_planet(cursor_position);
                    self.select_squad(cursor_position);
                }

                Event::Input(Input::Press(Button::Mouse(MouseButton::Right))) => {
                    if let Some(squad_id) = self.current_selected_squad {
                        let cursor_position = self.cursor_position;

                        let Size { width, height } = self.window.size();

                        let x = cursor_position[0] - width as f64 / 2.0;
                        let y = -cursor_position[1] + height as f64 / 2.0;

                        if let Some(ref sender) = self.sender {
                            sender.send(format!("{{\"move\":{},\"x\":{},\"y\":{}}}", squad_id, x, y));
                        }
                    }
                }

                Event::Input(Input::Press(Button::Keyboard(Key::B))) => {
                    if let Some(planet_id) = self.current_selected_planet {
                        if let Some(ref sender) = self.sender {
                            sender.send(format!("{{\"spawn\":{}}}", planet_id));
                        }
                    }
                }

                _ => {}
            }

        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const SPACE_COLOR:[f32; 4] = [0.015686275, 0.129411765, 0.250980392, 1.0];

        let planet_shape = ellipse::circle(0.0, 0.0, 10.0);
        let squad_shape = ellipse::circle(0.0, 0.0, 5.0);

        let (center_x, center_y) = ((args.width / 2) as f64, (args.height / 2) as f64);

        let planets = &self.planets;
        let players = &self.players;
        let glyph_cache = &mut self.glyph_cache;
        let gold = self.gold;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(SPACE_COLOR, gl);
            for (_, planet) in planets {
                let Position(planet_x, planet_y) = planet.position();

                let planet_transform = c.transform
                    .trans(center_x, center_y)
                    .trans(planet_x, -planet_y);

                ellipse(planet.color(), planet_shape, planet_transform, gl);
            }

            for (_, player) in players {
                for (_, squad) in player.squads() {
                    let Position(squad_x, squad_y) = squad.position();

                    let squad_transform = c.transform
                        .trans(center_x, center_y)
                        .trans(squad_x, -squad_y);

                    ellipse([1.0, 0.0, 0.0, 1.0], squad_shape, squad_transform, gl);
                    text(
                        [0.0, 0.0, 0.0, 1.0],
                        12,
                        &format!("{}", squad.count()),
                        glyph_cache,
                        squad_transform,
                        gl
                    );
                }
            }

            text(
                [0.870588235, 0.850980392, 0.529411765, 1.0],
                14,
                &format!("Gold: {}", gold.floor()),
                glyph_cache,
                c.transform.trans(5.0, 17.0),
                gl
            );
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if let Some(ref rx) = self.rx {
            while let Ok(command) = rx.try_recv() {
                match command {
                    Command::Connect { sender } => {
                        self.sender = Some(sender);
                    }
                    Command::Process { sender, planets_data, players, gold } => {
                        for planet_data in planets_data {
                            if let Some(planet) = self.planets.get_mut(&planet_data.id) {
                                planet.set_owner(planet_data.owner);
                                continue;
                            }

                            let PlanetData { id, position, owner } = planet_data;
                            let planet = Planet::new(id, position, owner);

                            self.planets.insert(id, planet);
                        }

                        self.players = players;
                        self.gold = gold;
                    },
                    _ => { }
                }
            };
        }
    }

    fn select_planet(&mut self, cursor: [f64; 2]) {
        let Size { width, height } = self.window.size();

        let x = cursor[0] - width as f64 / 2.0;
        let y = -cursor[1] + height as f64 / 2.0;

        self.current_selected_planet = None;

        let planets = &mut self.planets;
        for (_, planet) in planets {
            let Position(planet_x, planet_y) = planet.position();
            let distance = ((planet_x - x).powi(2) + (planet_y - y).powi(2)).sqrt();

            let mut color = [0.125490196, 0.752941176, 0.870588235, 1.0];

            if distance < 10.0 {
                color = [0.870588235, 0.850980392, 0.529411765, 1.0];
                self.current_selected_planet = Some(planet.id());
            }

            planet.set_color(color);
        }
    }

    fn select_squad(&mut self, cursor: [f64; 2]) {
        let Size { width, height } = self.window.size();

        let x = cursor[0] - width as f64 / 2.0;
        let y = -cursor[1] + height as f64 / 2.0;

        self.current_selected_squad = None;

        let players = &self.players;
        for (_, player) in players {
            for (_, squad) in player.squads() {
                let Position(squad_x, squad_y) = squad.position();
                let distance = ((squad_x - x).powi(2) + (squad_y - y).powi(2)).sqrt();

                if distance < 5.0 {
                    self.current_selected_squad = Some(squad.id());
                }
            }
        }
    }
}
