use std::thread;
use std::collections::HashMap;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::window::{Window, Size};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use ws::connect;
use std::sync::mpsc::{channel, Receiver as ChannelReceiver};

use client::client_command::ClientCommand;
use client::websocket_client::WebsocketClient;
use planet::PlanetClient;

pub struct Client {
    window: GlutinWindow,
    gl: GlGraphics,
    rx: Option<ChannelReceiver<ClientCommand>>,

    cursor_position: [f64; 2],
    planets: HashMap<u64, PlanetClient>
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
            rx: None,

            cursor_position: [0f64, 0f64],
            planets: HashMap::new()
        }
    }

    pub fn run(&mut self) {
        let (tx, rx) = channel::<ClientCommand>();

        self.rx = Some(rx);
        thread::spawn(move || connect("ws://127.0.0.1:3012", |out| WebsocketClient::new(out, tx.clone())).unwrap());

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
                Event::Input(Input::Press(Button::Mouse(_))) => {
                    let cursor_position = self.cursor_position;
                    self.select_planet(cursor_position);
                }

                _ => {}
            }

        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const SPACE_COLOR:[f32; 4] = [0.015686275, 0.129411765, 0.250980392, 1.0];

        let planet_shape = ellipse::circle(0.0, 0.0, 10.0);

        let (center_x, center_y) = ((args.width / 2) as f64, (args.height / 2) as f64);

        let planets = &self.planets;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(SPACE_COLOR, gl);
            for (_, planet) in planets {

                let planet_transform = c.transform
                    .trans(center_x, center_y)
                    .trans(planet.x, -planet.y);

                ellipse(planet.color, planet_shape, planet_transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if let Some(ref rx) = self.rx {
            while let Ok(client_command) = rx.try_recv() {
                match client_command {
                    ClientCommand::Process { planets } => {
                        let mut client_planets = &mut self.planets;

                        for planet in planets {
                            let id = planet.id;
                            if let Some(p) = client_planets.get_mut(&id) {
                                p.x = planet.x;
                                p.y = planet.y;

                                continue;
                            }

                            client_planets.insert(id, planet);
                        }
                    }
                }
            };
        }
    }

    fn select_planet(&mut self, cursor: [f64; 2]) {
        let Size { width, height } = self.window.size();

        let x = cursor[0] - width as f64 / 2.0;
        let y = -cursor[1] + height as f64 / 2.0;

        let planets = &mut self.planets;
        for (_, planet) in planets {
            let distance = ((planet.x - x).powi(2) + (planet.y - y).powi(2)).sqrt();

            planet.color = if distance < 10.0 {
                [0.870588235, 0.850980392, 0.529411765, 1.0]
            } else {
                [0.125490196, 0.752941176, 0.870588235, 1.0]
            }
        }
    }
}