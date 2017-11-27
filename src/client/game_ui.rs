use std::env::current_dir;
use std::time::Duration;

use conrod::{Colorable, Positionable, Theme, Ui, UiBuilder, Widget};
use conrod::color;
use conrod::widget::{Canvas, Text};
use conrod::backend::glium::Renderer;
use conrod::backend::winit;
use conrod::image::Map;
use conrod::position::{Align, Direction, Padding, Position, Relative};
use conrod::theme::StyleMap;
use glium::{Display, Surface};
use glium::glutin::WindowEvent;
use glium::texture::Texture2d;

widget_ids! {
    pub struct UiIds {
        master,

        header,
        header_items[],

        body,

        gold,
        planets,
        fps,
        players[]
    }
}

pub struct GameUi {
    ui: Ui,
    ui_ids: UiIds,
    ui_image_map: Map<Texture2d>,
    ui_renderer: Renderer,
}

impl GameUi {
    pub fn new(display: &Display, width: f64, height: f64) -> Self {
        let theme = Theme {
            name: "Vintergatan theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Relative(Relative::Align(Align::Start), None),
            y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
            background_color: color::TRANSPARENT,
            shape_color: color::LIGHT_CHARCOAL,
            border_color: color::BLACK,
            border_width: 0.0,
            label_color: color::WHITE,
            font_id: None,
            font_size_large: 18,
            font_size_medium: 14,
            font_size_small: 12,
            widget_styling: StyleMap::default(),
            mouse_drag_threshold: 0.0,
            double_click_threshold: Duration::from_millis(500),
        };

        let mut ui = UiBuilder::new([width, height])
            .theme(theme)
            .build();

        let font_path = current_dir().unwrap().join("assets/Exo2-Regular.ttf");
        ui.fonts.insert_from_file(&font_path).unwrap();

        let ui_ids = UiIds::new(ui.widget_id_generator());

        let ui_renderer = Renderer::new(display).unwrap();

        GameUi {
            ui,
            ui_ids,
            ui_image_map: Map::new(),
            ui_renderer
        }
    }

    pub fn draw<S>(&mut self, display: &Display, target: &mut S) where S: Surface {
        let primitives = self.ui.draw();

        self.ui_renderer.fill(display, primitives, &self.ui_image_map);
        self.ui_renderer.draw(display, target, &self.ui_image_map).unwrap();
    }

    pub fn update(&mut self, players_count: usize, gold: f64, planets_count: usize, fps: usize, players_states: Vec<String>) {
        const HEADER_ITEMS_COUNT: usize = 8;
        const HEADER_PADDING: f64 = 10.0;

        let mut ui = self.ui.set_widgets();

        self.ui_ids.header_items.resize(HEADER_ITEMS_COUNT, &mut ui.widget_id_generator());
        self.ui_ids.players.resize(players_count, &mut ui.widget_id_generator());

        let mut header_items = vec![];
        for i in 0..HEADER_ITEMS_COUNT {
            header_items.push(
                (
                    self.ui_ids.header_items[i],
                    Canvas::new()
                        .pad_left(HEADER_PADDING)
                        .pad_right(HEADER_PADDING)
                )
            )
        }

        Canvas::new()
            .flow_down(&[
                (
                    self.ui_ids.header,
                    Canvas::new()
                        .length(30.0)
                        .color(color::DARK_CHARCOAL)
                        .flow_right(&header_items)
                ),
                (self.ui_ids.body, Canvas::new())
            ])
            .set(self.ui_ids.master, &mut ui);

        Text::new(&format!("Gold: {}", gold.floor()))
            .color(color::LIGHT_BLUE)
            .mid_left_of(self.ui_ids.header_items[0])
            .set(self.ui_ids.gold, &mut ui);

        let planets_count = &format!("Planets: {}", planets_count);

        Text::new(planets_count)
            .color(color::LIGHT_BLUE)
            .mid_left_of(self.ui_ids.header_items[1])
            .set(self.ui_ids.planets, &mut ui);

        Text::new(&format!("FPS: {}", fps))
            .color(color::LIGHT_BLUE)
            .mid_left_of(self.ui_ids.header_items[2])
            .set(self.ui_ids.fps, &mut ui);

        let players_states_slice = &players_states[0..players_states.len().min(4)];
        for i in 0..players_states_slice.len() {
            Text::new(&players_states_slice[i])
                .color(color::LIGHT_BLUE)
                .mid_left_of(self.ui_ids.header_items[HEADER_ITEMS_COUNT - players_states_slice.len() + i])
                .set(self.ui_ids.players[i], &mut ui);
        }
    }

    pub fn process_event(&mut self, display: &Display, event: WindowEvent) {
        if let Some(input) = winit::convert_window_event(event, display) {
            self.ui.handle_event(input);
        }
    }
}
