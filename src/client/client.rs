use std::thread;
use std::collections::HashMap;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver as ChannelReceiver};

use fps_counter::FPSCounter;
use glium;
use glium::Surface;
use glium_text_rusttype as glium_text;
use vecmath;
use ws::{connect, Sender};

use client::camera::Camera;
use client::command::Command;
use client::game_cursor::GameCursor;
use client::game_event::GameEvent;
use client::game_ui::GameUi;
use client::input_mapping;
use client::json;
use client::player::Player;
use client::squad::Squad;
use client::waypoint::{Waypoint, WaypointType};
use common::{Id, PlayerId, Position};
use common::websocket_handler::WebsocketHandler;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

fn color_from_rgb(r: u8, g: u8, b: u8, alpha: f32) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, alpha]
}

pub struct Client {
    events_loop: glium::glutin::EventsLoop,
    display: glium::Display,

    text_system: glium_text::TextSystem,
    font: glium_text::FontTexture,

    shape: Vec<Vertex>,
    program: glium::Program,

    rx: Option<ChannelReceiver<Command>>,

    game_cursor: GameCursor,

    waypoints: HashMap<Id, Waypoint>,
    players: HashMap<PlayerId, Player>,
    squads: HashMap<Id, Squad>,
    gold: f64,
    me: PlayerId,

    current_selected_waypoint: Option<Id>,
    current_selected_squad: Option<Id>,
    is_modifier1: bool,
    is_modifier2: bool,
    sender: Option<Sender>,

    game_ui: GameUi,

    viewport: [f64; 2],
    camera: Camera,

    fps: usize,
    fps_counter: FPSCounter
}

impl Client {
    pub fn new() -> Self {
        const WIDTH: u32 = 1280;
        const HEIGHT: u32 = 800;

        let events_loop = glium::glutin::EventsLoop::new();

        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title("Vintergatan game on glium");

        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);

        let display = glium::Display::new(window, context, &events_loop).unwrap();

        display.gl_window().set_cursor_state(glium::glutin::CursorState::Grab);
        display.gl_window().set_cursor(glium::glutin::MouseCursor::NoneCursor);
        display.gl_window().set_cursor_position(WIDTH as i32 / 2, HEIGHT as i32 / 2);

        let text_system = glium_text::TextSystem::new(&display);
        let font = glium_text::FontTexture::new(
            &display,
            &include_bytes!("../../assets/Exo2-Regular.ttf")[..],
            32,
            glium_text::FontTexture::ascii_character_list()
        ).unwrap();

        let sectors_count = 128_u32;
        let mut shape = (0..(sectors_count + 1))
            .map(|sector| (sector as f32) * 2_f32 * ::std::f32::consts::PI / (sectors_count as f32))
            .map(|angle| Vertex { position: [angle.cos(), angle.sin()] })
            .collect::<Vec<_>>();

        shape.insert(0, Vertex { position: [0_f32, 0_f32] });

        let program = program!(&display,
            140 => {
                vertex: r#"
                    #version 140

                    in vec2 position;
                    out vec4 v_color;

                    uniform mat4 matrix;
                    uniform mat4 view;
                    uniform vec4 color;

                    void main() {
                        v_color = color;
                        gl_Position = view * matrix * vec4(position, 0.0, 1.0);
                    }
                "#,
                outputs_srgb: true,
                fragment: r#"
                    #version 140

                    in vec4 v_color;
                    out vec4 color;

                    void main() {
                        color = v_color;
                    }
                "#,
            },
        ).unwrap();

        let game_ui = GameUi::new(&display, WIDTH as f64, HEIGHT as f64);

        let game_cursor = GameCursor::new(&display);

        Client {
            events_loop,
            display,

            text_system,
            font,

            shape,
            program,

            rx: None,

            game_cursor,

            waypoints: HashMap::new(),
            players: HashMap::new(),
            squads: HashMap::new(),
            gold: 0.0,
            me: 0,

            current_selected_waypoint: None,
            current_selected_squad: None,
            is_modifier1: false,
            is_modifier2: false,
            sender: None,

            game_ui,

            viewport: [WIDTH as f64, HEIGHT as f64],
            camera: Camera::new(),

            fps: 0,
            fps_counter: FPSCounter::new()
        }
    }

    pub fn run(&mut self, address: String) {
        let (tx, rx) = channel::<Command>();

        self.rx = Some(rx);
        thread::spawn(move || connect(format!("ws://{}", address), |sender| WebsocketHandler::new(sender, tx.clone())).unwrap());

        'main: loop {
            let mut events = Vec::new();
            self.events_loop.poll_events(|ev| events.push(ev));

            for ev in events {
                match ev {
                    glium::glutin::Event::WindowEvent { event, .. } => {
                        match event {
                            glium::glutin::WindowEvent::Closed => break 'main,
                            _ => {
                                self.process_input(&event);
                                self.game_ui.process_event(&self.display, event);
                            }
                        }
                    }
                    _ => ()
                }
            }

            self.render();

            self.update();
            self.update_game_ui();

            self.camera.update(self.game_cursor.position(), &self.viewport, 0.05);

            thread::sleep(Duration::from_millis(40));
        }
    }

    fn render(&mut self) {
        const SPACE_COLOR: [f32; 4] = [0.015686275, 0.129411765, 0.250980392, 1.0];

        const SELECTION_COLOR:[f32; 4] = [0.0, 1.0, 0.0, 0.2];
        const PLANET_COLOR:[f32; 4] = [0.125490196, 0.752941176, 0.870588235, 1.0];
        const MY_PLANET_COLOR: [f32; 4] = [0.87843137, 0.50588235, 0.35686275, 1.0];
        const ENEMY_PLANET_COLOR: [f32; 4] = [0.34901961, 0.08627451, 0.14117647, 1.0];
        const MY_SQUAD_COLOR:[f32; 4] = [0.870588235, 0.850980392, 0.529411765, 1.0];
        const ENEMY_SQUAD_COLOR: [f32; 4] = [0.87843137, 0.22352941, 0.35686275, 1.0];
        const MY_TEXT_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 0.2];
        const ENEMY_TEXT_COLOR: [f32; 4] = [0.87843137, 0.22352941, 0.35686275, 1.0];

        let mut frame = self.display.draw();
        frame.clear_color_srgb(SPACE_COLOR[0], SPACE_COLOR[1], SPACE_COLOR[2], SPACE_COLOR[3]);

        let vertex_buffer = glium::VertexBuffer::new(&self.display, &self.shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

        let view = self.camera.view_matrix(&self.viewport);

        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            .. Default::default()
        };

        let me = self.me;

        let current_selected_waypoint = self.current_selected_waypoint;
        let current_selected_squad = self.current_selected_squad;

        for waypoint in self.waypoints.values() {
            let Position(waypoint_x, waypoint_y) = waypoint.position();

            let waypoint_size = Self::get_waypoint_size(waypoint) as f32;

            if let Some(current_selected_waypoint) = current_selected_waypoint {
                if waypoint.id() == current_selected_waypoint {
                    let selected_waypoint_size = (waypoint_size * 1.5).max(15.0);

                    let uniforms = uniform! {
                        matrix: [
                            [selected_waypoint_size, 0.0, 0.0, 0.0],
                            [0.0, selected_waypoint_size, 0.0, 0.0],
                            [0.0, 0.0, selected_waypoint_size, 0.0],
                            [waypoint_x as f32, waypoint_y as f32, 0.0, 1.0f32],
                        ],
                        view: view,
                        color: SELECTION_COLOR
                    };

                    frame.draw(&vertex_buffer, &indices, &self.program, &uniforms, &params).unwrap();
                }
            }

            let uniforms = uniform! {
                matrix: [
                    [waypoint_size, 0.0, 0.0, 0.0],
                    [0.0, waypoint_size, 0.0, 0.0],
                    [0.0, 0.0, waypoint_size, 0.0],
                    [waypoint_x as f32, waypoint_y as f32, 0.0, 1.0f32],
                ],
                view: view,
                color: self.get_waypoint_color(waypoint)
            };

            frame.draw(&vertex_buffer, &indices, &self.program, &uniforms, &params).unwrap();
        }

        for squad in self.squads.values() {
            let Position(squad_x, squad_y) = squad.position();

            let squad_color = if squad.owner() == me { MY_SQUAD_COLOR } else { ENEMY_SQUAD_COLOR };

            let uniforms = uniform! {
                matrix: [
                    [5.0, 0.0, 0.0, 0.0],
                    [0.0, 5.0, 0.0, 0.0],
                    [0.0, 0.0, 5.0, 0.0],
                    [squad_x as f32, squad_y as f32, 0.0, 1.0f32],
                ],
                view: view,
                color: squad_color
            };

            frame.draw(&vertex_buffer, &indices, &self.program, &uniforms, &params).unwrap();


            if let Some(current_selected_squad) = current_selected_squad {
                if squad.id() == current_selected_squad {
                    let uniforms = uniform! {
                        matrix: [
                            [8.0, 0.0, 0.0, 0.0],
                            [0.0, 8.0, 0.0, 0.0],
                            [0.0, 0.0, 8.0, 0.0],
                            [squad_x as f32, squad_y as f32, 0.0, 1.0f32],
                        ],
                        view: view,
                        color: SELECTION_COLOR
                    };

                    frame.draw(&vertex_buffer, &indices, &self.program, &uniforms, &params).unwrap();

                }
            }

            let mut text_x = squad_x;
            let mut text_y = squad_y;

            if squad.owner() == me {
                text_x += 16.0;
                text_y += 12.0;
            } else {
                text_x += 16.0;
                text_y -= 12.0;
            };

            let text_color = if squad.owner() == me { MY_TEXT_COLOR } else { ENEMY_TEXT_COLOR };
            let text = glium_text::TextDisplay::new(&self.text_system, &self.font, &format!("{}", squad.count()));

            let matrix = [
                [20.0, 0.0, 0.0, 0.0],
                [0.0, 20.0, 0.0, 0.0],
                [0.0, 0.0, 20.0, 0.0],
                [text_x as f32, text_y as f32, 0.0, 1.0]
            ];

            glium_text::draw(&text, &self.text_system, &mut frame, vecmath::col_mat4_mul(view, matrix), (text_color[0], text_color[1], text_color[2], text_color[3]));
        }

        self.game_ui.draw(&self.display, &mut frame);

        self.game_cursor.draw(&mut frame);

        frame.finish().unwrap();

        self.fps = self.fps_counter.tick();
    }

    fn update(&mut self) {
        if let Some(ref rx) = self.rx {
            while let Ok(command) = rx.try_recv() {
                match command {
                    Command::Connect { sender } => {
                        self.sender = Some(sender);
                    }

                    Command::Process { sender, waypoints, players, squads, gold, me } => {
                        self.waypoints = waypoints;
                        self.players = players;
                        self.squads = squads;
                        self.gold = gold;
                        self.me = me;
                    }

                    _ => ()
                }
            };
        }
    }

    fn update_game_ui(&mut self) {
        let players_count = self.players.len();

        let planets_count = self.waypoints
            .values()
            .filter(|waypoint| waypoint.waypoint_type() == WaypointType::Planet)
            .filter(|&planet| planet.owner().map_or(false, |owner| owner == self.me))
            .count();

        let mut players_states = self.players
            .values()
            .map(|player| format!("{}: {}", player.name(), player.state()))
            .collect::<Vec<_>>();

        players_states.sort();

        self.game_ui.update(players_count, self.gold, planets_count, self.fps, players_states);
    }

    fn process_input(&mut self, event: &glium::glutin::WindowEvent) {
        for mapping in self.get_input_mapping() {
            if let Some(game_event) = mapping(event) {
                match game_event {
                    GameEvent::ReadyToPlay => {
                        let command_json = json::format_ready_command();

                        if let Some(ref sender) = self.sender {
                            sender.send(command_json);
                        }
                    },

                    GameEvent::Cursor(x, y) => {
                        self.game_cursor.set_position((x, y));
                    },

                    GameEvent::SelectStart => {
                        self.select_waypoint();
                        self.select_squad();
                    },

                    GameEvent::SquadSpawn => {
                        if let Some(waypoint_id) = self.current_selected_waypoint {
                            if let Some(ref sender) = self.sender {
                                let command_json = json::format_squad_spawn_command(waypoint_id);
                                sender.send(command_json);
                            }
                        }
                    },

                    GameEvent::SquadMove => {
                        if let Some(squad_id) = self.current_selected_squad {
                            if let Some(waypoint) = self.find_waypoint_under_cursor() {
                                if let Some(ref sender) = self.sender {
                                    let command_json = json::format_squad_move_command(squad_id, waypoint.id());
                                    sender.send(command_json);
                                }
                            }
                        }
                    },

                    GameEvent::Modifier1Start => {
                        self.is_modifier1 = true;
                    },

                    GameEvent::Modifier1End => {
                        self.is_modifier1 = false;
                    },

                    GameEvent::Modifier2Start => {
                        self.is_modifier2 = true;
                    },

                    GameEvent::Modifier2End => {
                        self.is_modifier2 = false;
                    },

                    GameEvent::ZoomIn => {
                        self.camera.zoom_in();
                    }

                    GameEvent::ZoomOut => {
                        self.camera.zoom_out();
                    }

                    GameEvent::Resize(width, height) => {
                        self.viewport = [width as f64, height as f64];
                    }

                    _ => { }
                }

                break;
            }
        }
    }

    fn get_input_mapping(&self) -> Vec<fn(&glium::glutin::WindowEvent) -> Option<GameEvent>> {
        vec![
            input_mapping::map_squad_input,
            input_mapping::map_planet_input,
            input_mapping::map_root_input
        ]
    }

    fn find_waypoint_under_cursor(&self) -> Option<&Waypoint> {
        let (x, y) = self.cursor_world_coordinates();
        let cursor_position = Position(x as f64, y as f64);

        self.waypoints
            .values()
            .find(|waypoint| cursor_position.distance_to(waypoint.position()) < Self::get_waypoint_size(waypoint).max(20.0))
    }

    fn select_waypoint(&mut self) {
        self.current_selected_waypoint = self.find_waypoint_under_cursor().map(|waypoint| waypoint.id());
    }

    fn select_squad(&mut self) {
        let (x, y) = self.cursor_world_coordinates();
        let cursor_position = Position(x as f64, y as f64);

        self.current_selected_squad = self.squads
            .values()
            .find(|squad| cursor_position.distance_to(squad.position()) < 10_f64)
            .map(|squad| squad.id());
    }

    fn get_waypoint_size(waypoint: &Waypoint) -> f64 {
        match waypoint.waypoint_type() {
            WaypointType::Planet => 50.0,
            WaypointType::Planetoid => 20.0,
            WaypointType::BlackHole => 15.0,
            WaypointType::Asteroid => 5.0
        }
    }

    fn get_waypoint_color(&self, waypoint: &Waypoint) -> [f32; 4] {
        if waypoint.waypoint_type() == WaypointType::BlackHole {
            return color_from_rgb(0, 0, 0, 1.0);
        }

        waypoint.owner().map_or(
            color_from_rgb(255, 255, 255, 1.0),
            |owner| if owner == self.me { color_from_rgb(164, 196, 0, 1.0) } else { color_from_rgb(229, 20, 0, 1.0) }
        )
    }

    fn cursor_world_coordinates(&self) -> (f32, f32) {
        self.camera.unproject(self.game_cursor.position(), &self.viewport)
    }
}
