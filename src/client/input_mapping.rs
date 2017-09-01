use glium::glutin::{WindowEvent, ElementState, MouseButton, KeyboardInput, VirtualKeyCode};

use client::game_event::GameEvent;

pub fn map_root_input(event: &WindowEvent) -> Option<GameEvent> {
    match *event {
        WindowEvent::MouseMoved { position, .. } => Some(GameEvent::Cursor(position.0, position.1)),

        WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Pressed, .. } => Some(GameEvent::SelectStart),
            WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Released, .. } => Some(GameEvent::SelectEnd),

        WindowEvent::KeyboardInput {
            input: KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(VirtualKeyCode::Space),
                ..
            },
            ..
        } => Some(GameEvent::ReadyToPlay),

        WindowEvent::KeyboardInput {
            input: KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(VirtualKeyCode::Z),
                ..
            },
            ..
        } => Some(GameEvent::ZoomIn),

        WindowEvent::KeyboardInput {
            input: KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(VirtualKeyCode::X),
                ..
            },
            ..
        } => Some(GameEvent::ZoomOut),

        WindowEvent::Resized(width, height) => Some(GameEvent::Resize(width as f64, height as f64)),

        _ => None
    }
}

pub fn map_planet_input(event: &WindowEvent) -> Option<GameEvent> {
    match *event {
        WindowEvent::KeyboardInput {
            input: KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(VirtualKeyCode::S),
                ..
            },
            ..
        } => Some(GameEvent::SquadSpawn),

        _ => None
    }
}

pub fn map_squad_input(event: &WindowEvent) -> Option<GameEvent> {
    match *event {
        WindowEvent::MouseInput {
            button: MouseButton::Right,
            state: ElementState::Released,
            ..
        } => Some(GameEvent::SquadMove),

        _ => None
    }
}