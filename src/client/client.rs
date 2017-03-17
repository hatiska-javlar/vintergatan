use std::thread;
use std::collections::HashMap;
use std::env::current_dir;
use std::time::Duration;
use std::cmp::min;

use piston_window;
use piston_window::{
    OpenGL,
    PistonWindow, Window, AdvancedWindow, WindowSettings, Size,
    clear, ellipse, text, image, Ellipse, Transformed,
    Input, RenderArgs,
    Key, MouseButton, Motion,
    G2d, G2dTexture, TextureSettings, Texture, Flip
};
use piston_window::texture::UpdateTexture;

use conrod;
use gfx_device_gl;

use ws::{connect, Sender};
use std::sync::mpsc::{channel, Receiver as ChannelReceiver};

use client::command::Command;
use client::game_event::GameEvent;
use client::input_mapping;
use client::json;
use client::planet::Planet;
use client::player::Player;
use client::squad::Squad;
use common::{Id, PlayerId, Position};
use common::websocket_handler::WebsocketHandler;

widget_ids! {
    pub struct UiIds {
        master,

        header,
        header_items[],

        body,

        gold,
        planets,
        players[]
    }
}

pub struct Client {
    window: PistonWindow,
    glyph_cache: piston_window::Glyphs,
    rx: Option<ChannelReceiver<Command>>,

    cursor_position: (f64, f64),
    cursor_icon: Texture<gfx_device_gl::Resources>,

    planets: HashMap<Id, Planet>,
    players: HashMap<PlayerId, Player>,
    squads: HashMap<Id, Squad>,
    gold: f64,
    me: PlayerId,

    current_selected_planet: Option<Id>,
    current_selected_squad: Option<Id>,
    is_modifier1: bool,
    is_modifier2: bool,
    sender: Option<Sender>,

    ui: conrod::Ui,
    ui_ids: UiIds,
    ui_image_map: conrod::image::Map<Texture<gfx_device_gl::Resources>>,
    ui_glyph_cache: conrod::text::GlyphCache,
    ui_text_texture_cache: Texture<gfx_device_gl::Resources>
}

impl Client {
    pub fn new() -> Self {
        use conrod::position::{Padding, Position, Relative, Align, Direction};

        const WIDTH: u32 = 1280;
        const HEIGHT: u32 = 800;

        let opengl = OpenGL::V3_2;

        let mut window: PistonWindow =
            WindowSettings::new("Vintergatan game", [WIDTH, HEIGHT])
                .opengl(opengl)
                .exit_on_esc(true)
                .build()
                .unwrap();

        window.set_capture_cursor(true);

        let window_factory = window.factory.clone();

        let theme = conrod::Theme {
            name: "Vintergatan theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Relative(Relative::Align(Align::Start), None),
            y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
            background_color: conrod::color::TRANSPARENT,
            shape_color: conrod::color::LIGHT_CHARCOAL,
            border_color: conrod::color::BLACK,
            border_width: 0.0,
            label_color: conrod::color::WHITE,
            font_id: None,
            font_size_large: 18,
            font_size_medium: 14,
            font_size_small: 12,
            widget_styling: HashMap::new(),
            mouse_drag_threshold: 0.0,
            double_click_threshold: Duration::from_millis(500),
        };

        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
            .theme(theme)
            .build();

        let font_path = current_dir().unwrap().join("assets/Exo2-Regular.ttf");
        ui.fonts.insert_from_file(&font_path).unwrap();

        let cursor_icon_path = current_dir().unwrap().join("assets/cursor.png");
        let cursor_icon = Texture::from_path(
            &mut window.factory,
            &cursor_icon_path,
            Flip::None,
            &TextureSettings::new()
        ).unwrap();

        let (ui_glyph_cache, ui_text_texture_cache) = {
            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;
            let cache = conrod::text::GlyphCache::new(WIDTH, HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE);
            let buffer_len = WIDTH as usize * HEIGHT as usize;
            let init = vec![128; buffer_len];
            let settings = TextureSettings::new();
            let factory = &mut window.factory.clone();
            let texture = G2dTexture::from_memory_alpha(factory, &init, WIDTH, HEIGHT, &settings).unwrap();
            (cache, texture)
        };

        let ui_ids = UiIds::new(ui.widget_id_generator());

        Client {
            window: window,
            glyph_cache: piston_window::Glyphs::new(&font_path, window_factory).unwrap(),
            rx: None,

            cursor_position: (0_f64, 0_f64),
            cursor_icon: cursor_icon,

            planets: HashMap::new(),
            players: HashMap::new(),
            squads: HashMap::new(),
            gold: 0.0,
            me: 0,

            current_selected_planet: None,
            current_selected_squad: None,
            is_modifier1: false,
            is_modifier2: false,
            sender: None,

            ui: ui,
            ui_ids: ui_ids,
            ui_image_map: conrod::image::Map::new(),
            ui_glyph_cache: ui_glyph_cache,
            ui_text_texture_cache: ui_text_texture_cache
        }
    }

    pub fn run(&mut self, address: String) {
        let (tx, rx) = channel::<Command>();

        self.rx = Some(rx);
        thread::spawn(move || connect(format!("ws://{}", address), |sender| WebsocketHandler::new(sender, tx.clone())).unwrap());

        while let Some(event) = self.window.next() {
            match event {
                Input::Render(args) => {
                    self.render(&args, &event);
                    self.render_ui(&event);
                }

                Input::Update(_) => {
                    self.update();
                    self.update_ui();
                }

                _ => {
                    self.process_input(&event);
                    self.process_ui(event);
                }
            }
        }
    }

    fn render(&mut self, args: &RenderArgs, event: &Input) {
        const SPACE_COLOR:[f32; 4] = [0.015686275, 0.129411765, 0.250980392, 1.0];
        const SELECTION_COLOR:[f32; 4] = [0.0, 1.0, 0.0, 0.2];
        const PLANET_COLOR:[f32; 4] = [0.125490196, 0.752941176, 0.870588235, 1.0];
        const MY_PLANET_COLOR: [f32; 4] = [0.87843137, 0.50588235, 0.35686275, 1.0];
        const ENEMY_PLANET_COLOR: [f32; 4] = [0.34901961, 0.08627451, 0.14117647, 1.0];
        const MY_SQUAD_COLOR:[f32; 4] = [0.870588235, 0.850980392, 0.529411765, 1.0];
        const ENEMY_SQUAD_COLOR: [f32; 4] = [0.87843137, 0.22352941, 0.35686275, 1.0];
        const MY_TEXT_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 0.2];
        const ENEMY_TEXT_COLOR: [f32; 4] = [0.87843137, 0.22352941, 0.35686275, 1.0];

        let planet_shape = ellipse::circle(0.0, 0.0, 10.0);
        let squad_shape = ellipse::circle(0.0, 0.0, 5.0);

        let (center_x, center_y) = ((args.width / 2) as f64, (args.height / 2) as f64);

        let planets = &self.planets;
        let squads = &self.squads;
        let glyph_cache = &mut self.glyph_cache;
        let me = self.me;

        let current_selected_planet = self.current_selected_planet;
        let current_selected_squad = self.current_selected_squad;

        self.window.draw_2d(event, |c, gl| {
            clear(SPACE_COLOR, gl);
            for (_, planet) in planets {
                let Position(planet_x, planet_y) = planet.position();

                let planet_transform = c.transform
                    .trans(center_x, center_y)
                    .trans(planet_x, -planet_y);

                let planet_color = planet.owner().map_or(PLANET_COLOR, |owner| {
                    if owner == me {
                        MY_PLANET_COLOR
                    } else {
                        ENEMY_PLANET_COLOR
                    }
                });

                ellipse(planet_color, planet_shape, planet_transform, gl);

                if let Some(current_selected_planet) = current_selected_planet {
                    if planet.id() == current_selected_planet {
                        Ellipse::new_border(SELECTION_COLOR, 1.0)
                            .draw([-18.0, -18.0, 36.0, 36.0], &c.draw_state, planet_transform, gl);
                    }
                }
            }

            for squad in squads.values() {
                let Position(squad_x, squad_y) = squad.position();

                let squad_transform = c.transform
                    .trans(center_x, center_y)
                    .trans(squad_x, -squad_y);

                let squad_color = if squad.owner() == me { MY_SQUAD_COLOR } else { ENEMY_SQUAD_COLOR };
                ellipse(squad_color, squad_shape, squad_transform, gl);

                if let Some(current_selected_squad) = current_selected_squad {
                    if squad.id() == current_selected_squad {
                        Ellipse::new_border(SELECTION_COLOR, 1.0)
                            .draw([-12.0, -12.0, 24.0, 24.0], &c.draw_state, squad_transform, gl);
                    }
                }

                let label_transform = if squad.owner() == me {
                    squad_transform.trans(16.0, 24.0)
                } else {
                    squad_transform.trans(16.0, -12.0)
                };

                let text_color = if squad.owner() == me { MY_TEXT_COLOR } else { ENEMY_TEXT_COLOR };
                text(
                    text_color,
                    12,
                    &format!("{}", squad.count()),
                    glyph_cache,
                    label_transform,
                    gl
                );
            }
        });
    }

    fn render_ui(&mut self, event: &Input) {
        let ref mut ui_text_texture_cache = &mut self.ui_text_texture_cache;
        let ref mut ui_glyph_cache = &mut self.ui_glyph_cache;
        let ref ui_image_map = &self.ui_image_map;
        let primitives = self.ui.draw();

        let ref cursor_icon = self.cursor_icon;
        let (cursor_x, cursor_y) = self.cursor_position;

        self.window.draw_2d(event, |c, gl| {
            let cache_queued_glyphs = |graphics: &mut G2d,
                                       cache: &mut G2dTexture,
                                       rect: conrod::text::rt::Rect<u32>,
                                       data: &[u8]|
                {
                    let offset = [rect.min.x, rect.min.y];
                    let size = [rect.width(), rect.height()];
                    let format = piston_window::texture::Format::Rgba8;
                    let encoder = &mut graphics.encoder;
                    let mut text_vertex_data = Vec::new();
                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                    UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
                        .expect("failed to update texture")
                };

            fn texture_from_image<T>(img: &T) -> &T { img }

            conrod::backend::piston::draw::primitives(
                primitives,
                c,
                gl,
                ui_text_texture_cache,
                ui_glyph_cache,
                ui_image_map,
                cache_queued_glyphs,
                texture_from_image
            );

            image(cursor_icon, c.transform.trans(cursor_x, cursor_y), gl);
        });
    }

    fn update(&mut self) {
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

    fn update_ui(&mut self) {
        use conrod::{Widget, widget, Colorable, color, Positionable};

        const HEADER_ITEMS_COUNT: usize = 8;
        const HEADER_PADDING: f64 = 10.0;

        let mut ui = self.ui.set_widgets();

        self.ui_ids.header_items.resize(HEADER_ITEMS_COUNT, &mut ui.widget_id_generator());
        self.ui_ids.players.resize(self.players.len(), &mut ui.widget_id_generator());

        let mut header_items = vec![];
        for i in 0..HEADER_ITEMS_COUNT {
            header_items.push(
                (
                    self.ui_ids.header_items[i],
                    widget::Canvas::new().pad_left(HEADER_PADDING).pad_right(HEADER_PADDING)
                )
            )
        }

        widget::Canvas::new().flow_down(&[
            (
                self.ui_ids.header,
                widget::Canvas::new().length(30.0).color(color::DARK_CHARCOAL).flow_right(&header_items)
            ),
            (self.ui_ids.body, widget::Canvas::new())
        ]).set(self.ui_ids.master, &mut ui);

        widget::Text::new(&format!("Gold: {}", self.gold.floor()))
            .color(color::LIGHT_BLUE)
            .mid_left_of(self.ui_ids.header_items[0])
            .set(self.ui_ids.gold, &mut ui);

        let planets = self.planets.values();
        let me = self.me;
        let planets_count = &format!(
            "Planets: {}",
            planets.filter(|&planet| planet.owner().map_or(false, |owner| owner == me)).count()
        );

        widget::Text::new(planets_count)
            .color(color::LIGHT_BLUE)
            .mid_left_of(self.ui_ids.header_items[1])
            .set(self.ui_ids.planets, &mut ui);

        let mut players_state = self.players.values()
            .map(|player| format!("{}: {}", player.name(), player.state()))
            .collect::<Vec<_>>();

        players_state.sort();
        let players_state_slice = &players_state[0..min(4, players_state.len())];
        for i in 0..players_state_slice.len() {
            widget::Text::new(&players_state_slice[i])
                .color(color::LIGHT_BLUE)
                .mid_left_of(self.ui_ids.header_items[HEADER_ITEMS_COUNT - players_state_slice.len() + i])
                .set(self.ui_ids.players[i], &mut ui);
        }
    }

    fn process_input(&mut self, event: &Input) {
        for mapping in self.get_input_mapping() {
            if let Some(game_event) = mapping(&event) {
                match game_event {
                    GameEvent::ReadyToPlay => {
                        let command_json = json::format_ready_command();

                        if let Some(ref sender) = self.sender {
                            sender.send(command_json);
                        }
                    },

                    GameEvent::Cursor(dx, dy) => {
                        let (dx, dy) = self.normalize_mouse_cursor(dx, dy);
                        let (cursor_x, cursor_y) = self.cursor_position;

                        let x = cursor_x + dx;
                        let y = cursor_y + dy;

                        let Size { width, height } = self.window.size();

                        self.cursor_position = (
                            x.max(0_f64).min(width as f64),
                            y.max(0_f64).min(height as f64)
                        );
                    },

                    GameEvent::SelectStart => {
                        self.select_planet();
                        self.select_squad();
                    },

                    GameEvent::SquadSpawn => {
                        if let Some(planet_id) = self.current_selected_planet {
                            if let Some(ref sender) = self.sender {
                                let command_json = json::format_squad_spawn_command(planet_id);
                                sender.send(command_json);
                            }
                        }
                    },

                    GameEvent::SquadMove => {
                        if let Some(squad_id) = self.current_selected_squad {
                            let (cursor_x, cursor_y) = self.cursor_position;

                            let Size { width, height } = self.window.size();

                            let x = cursor_x - width as f64 / 2.0;
                            let y = -cursor_y + height as f64 / 2.0;

                            if let Some(ref sender) = self.sender {
                                let cut_count = self.get_cut_count();
                                let command_json = json::format_squad_move_command(squad_id, x, y, cut_count);
                                sender.send(command_json);
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

                    _ => { }
                }

                break;
            }
        }
    }

    fn process_ui(&mut self, event: Input) {
        let Size { width, height } = self.window.size();
        let (cursor_x, cursor_y) = self.cursor_position;

        match event {
            Input::Move(Motion::MouseRelative(_, _)) => {
                let mouse_cursor_motion = Motion::MouseCursor(
                    cursor_x - width as f64 / 2_f64,
                    -cursor_y + height as f64 / 2_f64
                );
                let conrod_event = conrod::event::Input::Move(mouse_cursor_motion);
                self.ui.handle_event(conrod_event);
            },

            _ => {
                if let Some(e) = conrod::backend::piston::event::convert(event, width as f64, height as f64) {
                    self.ui.handle_event(e);
                }
            }
        }
    }

    fn get_input_mapping(&self) -> Vec<fn(&Input) -> Option<GameEvent>> {
        vec![
            input_mapping::map_squad_input,
            input_mapping::map_planet_input,
            input_mapping::map_root_input
        ]
    }

    fn select_planet(&mut self) {
        let Size { width, height } = self.window.size();
        let (cursor_x, cursor_y) = self.cursor_position;

        let x = cursor_x - width as f64 / 2.0;
        let y = -cursor_y + height as f64 / 2.0;

        let cursor_position = Position(x, y);

        self.current_selected_planet = self.planets
            .values()
            .find(|planet| cursor_position.distance_to(planet.position()) < 20_f64)
            .map(|planet| planet.id());
    }

    fn select_squad(&mut self) {
        let Size { width, height } = self.window.size();
        let (cursor_x, cursor_y) = self.cursor_position;

        let x = cursor_x - width as f64 / 2.0;
        let y = -cursor_y + height as f64 / 2.0;

        let cursor_position = Position(x, y);

        self.current_selected_squad = self.squads
            .values()
            .find(|squad| cursor_position.distance_to(squad.position()) < 10_f64)
            .map(|squad| squad.id());
    }

    fn get_cut_count(&self) -> Option<u64> {
        if self.is_modifier1 && self.is_modifier2 {
            return Some(1);
        }

        if self.is_modifier1 {
            return Some(10);
        }

        if self.is_modifier2 {
            return Some(50);
        }

        None
    }

    fn normalize_mouse_cursor(&self, dx: f64, dy: f64) -> (f64, f64){
        if cfg!(target_os = "macos") {
            let Size { width, height } = self.window.size();
            return (
                dx - width as f64 / 2_f64,
                dy - height as f64 / 2_f64
            );
        }

        (dx, dy)
    }
}