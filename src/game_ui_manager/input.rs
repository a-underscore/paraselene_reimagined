use hex::winit::{keyboard::KeyCode, event::MouseButton};

#[derive(Eq, PartialEq, Hash)]
pub enum Input {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}
