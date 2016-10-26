use std::thread;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use ws::{connect, Sender, Handler, Result, Message, Handshake, CloseCode};

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64   // Rotation for the square.
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        // rgb(4, 33, 63)
        const SPACE_COLOR:[f32; 4] = [0.015686275, 0.129411765, 0.250980392, 1.0];
        // rgb(32, 192, 222)
        const PLANET_COLOR:[f32; 4] = [0.125490196, 0.752941176, 0.870588235, 1.0];
        // rgb(222, 217, 135)
        const TADPOLE_COLOR:[f32; 4] = [0.870588235, 0.850980392, 0.529411765, 1.0];

        let planet = ellipse::circle(0.0, 0.0, 50.0);
        let tadpole = ellipse::circle(0.0, 0.0, 5.0);

        let rotation = self.rotation;
        let (x, y) = ((args.width / 2) as f64,
                      (args.height / 2) as f64);


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(SPACE_COLOR, gl);

            let planet_transform = c.transform.trans(x, y);
            let tadpole_transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-50.0, -50.0);

            // Draw a box rotating around the middle of the screen.
            ellipse(PLANET_COLOR, planet, planet_transform, gl);
            ellipse(TADPOLE_COLOR, tadpole, tadpole_transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

pub struct Client {}

impl Client {
    pub fn run(&self) {
        thread::spawn(move || connect("ws://127.0.0.1:3012", |out| WebsocketClient { out: out }).unwrap());

        // Change this to OpenGL::V2_1 if not working.
        let opengl = OpenGL::V3_2;

        // Create an Glutin window.
        let mut window: Window = WindowSettings::new(
                "spinning-square",
                [200, 200]
            )
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        // Create a new game and run it.
        let mut app = App {
            gl: GlGraphics::new(opengl),
            rotation: 0.0
        };

        let mut events = window.events();
        while let Some(e) = events.next(&mut window) {
            if let Some(r) = e.render_args() {
                app.render(&r);
            }

            if let Some(u) = e.update_args() {
                app.update(&u);
            }
        }
    }
}

struct WebsocketClient {
    out: Sender
}

impl Handler for WebsocketClient {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("New connection is opened from {}", shake.peer_addr.unwrap());

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