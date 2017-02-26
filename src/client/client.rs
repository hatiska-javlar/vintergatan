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
use client::json;
use client::planet::Planet;
use client::player::Player;
use client::squad::Squad;
use common::{Id, PlayerId, Position};
use common::websocket_handler::WebsocketHandler;

pub struct Client {
    window: GlutinWindow,
    gl: GlGraphics,
    glyph_cache: GlyphCache<'static>,
    rx: Option<ChannelReceiver<Command>>,

    cursor_position: [f64; 2],
    planets: HashMap<Id, Planet>,
    players: HashMap<PlayerId, Player>,
    squads: HashMap<Id, Squad>,
    gold: f64,
    me: PlayerId,

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
            squads: HashMap::new(),
            gold: 0.0,
            me: 0,

            current_selected_planet: None,
            current_selected_squad: None,
            sender: None
        }
    }

    pub fn run(&mut self, address: String) {
        let (tx, rx) = channel::<Command>();

        self.rx = Some(rx);
        thread::spawn(move || connect(format!("ws://{}", address), |sender| WebsocketHandler::new(sender, tx.clone())).unwrap());

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
                            let command_json = json::format_squad_move_command(squad_id, x, y);
                            sender.send(command_json);
                        }
                    }
                }

                Event::Input(Input::Press(Button::Keyboard(Key::B))) => {
                    if let Some(planet_id) = self.current_selected_planet {
                        if let Some(ref sender) = self.sender {
                            let command_json = json::format_squad_spawn_command(planet_id);
                            sender.send(command_json);
                        }
                    }
                }

                _ => { }
            }

        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const SPACE_COLOR:[f32; 4] = [0.015686275, 0.129411765, 0.250980392, 1.0];
        const GOLD_COLOR:[f32; 4] = [0.870588235, 0.850980392, 0.529411765, 1.0];
        const TEXT_COLOR:[f32; 4] = [0.878431373, 0.223529412, 0.356862745, 1.0];
        const SELECTION_COLOR:[f32; 4] = [0.878431373, 0.223529412, 0.356862745, 0.2];
        const PLANET_COLOR:[f32; 4] = [0.125490196, 0.752941176, 0.870588235, 1.0];
        const MY_PLANET_COLOR: [f32; 4] = [0.87843137, 0.50588235, 0.35686275, 1.0];
        const SQUAD_COLOR:[f32; 4] = [0.870588235, 0.850980392, 0.529411765, 1.0];
        const HIGHLIGHT_COLOR:[f32; 4] = [1.0, 1.0, 1.0, 0.5];

        let planet_shape = ellipse::circle(0.0, 0.0, 10.0);
        let squad_shape = ellipse::circle(0.0, 0.0, 5.0);

        let (center_x, center_y) = ((args.width / 2) as f64, (args.height / 2) as f64);

        let planets = &self.planets;
        let players = &self.players;
        let squads = &self.squads;
        let glyph_cache = &mut self.glyph_cache;
        let gold = self.gold;
        let me = self.me;

        let current_selected_planet = self.current_selected_planet;
        let current_selected_squad = self.current_selected_squad;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(SPACE_COLOR, gl);
            for (_, planet) in planets {
                let Position(planet_x, planet_y) = planet.position();

                let planet_transform = c.transform
                    .trans(center_x, center_y)
                    .trans(planet_x, -planet_y);

                let is_my_planet = planet.owner().map_or(false, |owner| owner == me);
                let planet_color = if is_my_planet {
                    MY_PLANET_COLOR
                } else {
                    PLANET_COLOR
                };

                ellipse(planet_color, planet_shape, planet_transform, gl);

                if let Some(current_selected_planet) = current_selected_planet {
                    if planet.id() == current_selected_planet {
                        ellipse(HIGHLIGHT_COLOR, planet_shape, planet_transform, gl);
                        Rectangle::new_border(SELECTION_COLOR, 1.0)
                            .draw([-15.0, -15.0, 30.0, 30.0], &c.draw_state, planet_transform, gl);
                    }
                }
            }

            for squad in squads.values() {
                let Position(squad_x, squad_y) = squad.position();

                let squad_transform = c.transform
                    .trans(center_x, center_y)
                    .trans(squad_x, -squad_y);

                ellipse(SQUAD_COLOR, squad_shape, squad_transform, gl);

                if let Some(current_selected_squad) = current_selected_squad {
                    if squad.id() == current_selected_squad {
                        ellipse(HIGHLIGHT_COLOR, squad_shape, squad_transform, gl);
                        Rectangle::new_border(SELECTION_COLOR, 1.0)
                            .draw([-10.0, -10.0, 20.0, 20.0], &c.draw_state, squad_transform, gl);
                    }
                }

                text(
                    TEXT_COLOR,
                    12,
                    &format!("{}", squad.count()),
                    glyph_cache,
                    squad_transform,
                    gl
                );
            }

            text(
                GOLD_COLOR,
                14,
                &format!("Gold: {}", gold.floor()),
                glyph_cache,
                c.transform.trans(5.0, 17.0),
                gl
            );

            text(
                GOLD_COLOR,
                14,
                &format!("Planets: {}", planets.values().filter(|&planet|
                    planet.owner().map_or(false, |owner| owner == me)).count()),
                glyph_cache,
                c.transform.trans(100.0, 17.0),
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
                    Command::Process { sender, planets, players, squads, gold, me } => {
                        self.planets = planets;
                        self.players = players;
                        self.squads = squads;
                        self.gold = gold;
                        self.me = me;
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

            if distance < 20.0 {
                self.current_selected_planet = Some(planet.id());
            }
        }
    }

    fn select_squad(&mut self, cursor: [f64; 2]) {
        let Size { width, height } = self.window.size();

        let x = cursor[0] - width as f64 / 2.0;
        let y = -cursor[1] + height as f64 / 2.0;

        self.current_selected_squad = self.squads
            .values()
            .find(|squad| {
                let Position(squad_x, squad_y) = squad.position();
                let distance = ((squad_x - x).powi(2) + (squad_y - y).powi(2)).sqrt();

                distance < 10_f64
            })
            .map(|squad| squad.id());
    }
}