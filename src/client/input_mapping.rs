use piston_window::Button;
use piston_window::Input;
use piston_window::Key;
use piston_window::Motion;
use piston_window::MouseButton;

use client::game_event::GameEvent;

pub fn map_root_input(event: &Input) -> Option<GameEvent> {
    match *event {
        Input::Move(Motion::MouseRelative(x, y)) => Some(GameEvent::Cursor(x, y)),

        Input::Press(Button::Mouse(MouseButton::Left)) => Some(GameEvent::SelectStart),
        Input::Release(Button::Mouse(MouseButton::Left)) => Some(GameEvent::SelectEnd),

        Input::Press(Button::Keyboard(Key::Space)) => Some(GameEvent::ReadyToPlay),

        Input::Press(Button::Keyboard(Key::LCtrl)) => Some(GameEvent::Modifier1Start),
        Input::Release(Button::Keyboard(Key::LCtrl)) => Some(GameEvent::Modifier1End),

        Input::Press(Button::Keyboard(Key::LShift)) => Some(GameEvent::Modifier2Start),
        Input::Release(Button::Keyboard(Key::LShift)) => Some(GameEvent::Modifier2End),

        _ => None
    }
}

pub fn map_planet_input(event: &Input) -> Option<GameEvent> {
    match *event {
        Input::Press(Button::Keyboard(Key::S)) => Some(GameEvent::SquadSpawn),

        _ => None
    }
}

pub fn map_squad_input(event: &Input) -> Option<GameEvent> {
    match *event {
        Input::Press(Button::Mouse(MouseButton::Right)) => Some(GameEvent::SquadMove),

        _ => None
    }
}