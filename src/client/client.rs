use std::thread;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use ws::{connect, Sender, Handler, Result, Message, Handshake, CloseCode};
use std::sync::mpsc::Sender as ChanelSender;
use std::sync::mpsc::Receiver as ChanelReceiver;
use std::sync::mpsc::channel;
use planet::PlanetClient;
use rustc_serialize::json::{Object, Json};
use std::collections::HashMap;

pub struct App {
    gl: GlGraphics,
    planets: HashMap<u64, PlanetClient>,
    rx: ChanelReceiver<ClientCommand>
}

enum ClientCommand {
    Process {
        planets: Vec<PlanetClient>
    }
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const SPACE_COLOR:[f32; 4] = [0.015686275, 0.129411765, 0.250980392, 1.0];

        let planet_shape = ellipse::circle(0.0, 0.0, 10.0);

        let (center_x, center_y) = ((args.width / 2) as f64,
                      (args.height / 2) as f64);

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
        while let Ok(client_command) = self.rx.try_recv() {
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

    fn select_planet(&mut self, cursor: [f64; 2]) {
        let x = cursor[0] - 640.0;
        let y = -cursor[1] + 400.0;

        let planets = &mut self.planets;
        for (_, planet) in planets {
            let distance = ((planet.x - x)*(planet.x - x) + (planet.y - y)*(planet.y - y)).sqrt();
            if distance <= 10.0 {
                planet.color = [0.870588235, 0.850980392, 0.529411765, 1.0];
            }
        }
    }
}

pub struct Client {
    pub cursor_position: [f64; 2]
}

impl Client {
    pub fn run(&mut self) {
        let (tx, rx) = channel::<ClientCommand>();

        thread::spawn(move || connect("ws://127.0.0.1:3012", |out| WebsocketClient { out: out, tx: tx.clone() }).unwrap());

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an Glutin window.
        let mut window: Window = WindowSettings::new(
                "spinning-square",
                [1280, 800]
            )
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        // Create a new game and run it.
        let mut app = App {
            gl: GlGraphics::new(opengl),
            planets: HashMap::new(),
            rx: rx
        };

        let mut events = window.events();
        while let Some(e) = events.next(&mut window) {
            if let Some(r) = e.render_args() {
                app.render(&r);
            }

            if let Some(u) = e.update_args() {
                app.update(&u);
            }

            if let Some(m) = e.mouse_cursor_args() {
                self.cursor_position = m;
            }

            match e {
                Event::Input(Input::Press(Button::Mouse(_))) => {
                    app.select_planet(self.cursor_position);
                }

                _ => {}
            }

        }
    }
}

struct WebsocketClient {
    out: Sender,
    tx: ChanelSender<ClientCommand>
}

impl Handler for WebsocketClient {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("New connection is opened from {}", shake.peer_addr.unwrap());

        Ok(())
    }

    fn on_message(&mut self, message: Message) -> Result<()> {
        let raw = message.into_text().unwrap_or("".to_string());
        let parsed = Json::from_str(&raw).unwrap_or(Json::Object(Object::new()));

        let empty_json_object = Object::new();
        let params = parsed.as_object().unwrap_or(&empty_json_object);

        if let Some(planets_json) = params.get("planets") {
            let planets = planets_json.as_array().unwrap().into_iter().map(|planet_json| {
                PlanetClient {
                    id: planet_json.as_object().unwrap().get("id").unwrap().as_u64().unwrap(),
                    x: planet_json.as_object().unwrap().get("x").unwrap().as_f64().unwrap(),
                    y: planet_json.as_object().unwrap().get("y").unwrap().as_f64().unwrap(),
                    color: [0.125490196, 0.752941176, 0.870588235, 1.0],
                    size: 10.0
                }
            }).collect();

            self.tx.send(ClientCommand::Process {
                planets: planets
            }).unwrap();
        }

        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("Connection closed code = {:?}, reason = {}", code, reason);
    }
}
