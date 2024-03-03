use ggez::{glam::Vec2, graphics::Rect};

use super::{MousePressMode, TextRenderer};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Idle, Hovered, Depressed, Disabled,
}
#[derive(PartialEq)]
pub enum PressMode {
    Immediate, Release
}

pub struct Button {
    pub pressed: bool,
    pub state: State,
    pub rect: Rect,
    press_mode: PressMode,
}

impl Button {
    pub fn new(rect: Rect, press_mode: PressMode, disabled: bool) -> Button {
        Button { rect, press_mode, pressed: false, state: if disabled { State::Disabled } else { State::Idle } }
    }

    pub fn update(&mut self, mouse_pos: Vec2, mouse_mode: MousePressMode) {
        if self.state == State::Disabled { return; }
        self.state = match (mouse_mode, self.state, self.rect.contains(mouse_pos)) {
            // If the mouse isn't over the button, idle
            (.., false) => State::Idle,
            // If the mouse is over the button and the mouse has been pressed or released
            (MousePressMode::Down, ..) => {
                if self.press_mode == PressMode::Immediate { self.pressed = true; } State::Depressed }
            (MousePressMode::Up,   ..) => {
                if self.press_mode == PressMode::Release   { self.pressed = true; } State::Hovered }
            // If the mouse hasn't been pressed or released, if we're depressed stay depressed, otherwise we're hovering
            (_, State::Depressed, _) => State::Depressed,
            _ => State::Hovered,
        };
    }

    pub fn pos(&self) -> Vec2 { Vec2 { x: self.rect.x, y: self.rect.y } }

    pub fn pressed(&mut self) -> bool {
        if self.pressed {
            self.pressed = false;
            true
        } else { false }
    }
}


pub struct LabeledButton {
    pub b: Button,
    pub label: String,
    pub padding: (f32, f32, f32, f32)
}
impl LabeledButton {
    pub fn new(tr: &TextRenderer, label: String, padding: Option<(f32, f32, f32, f32)>, pos: Vec2, press_mode: PressMode, disabled: bool) -> LabeledButton {
        let padding = padding.unwrap_or((2.0, 1.0, 2.0, 1.0));
        let dimensions = tr.text_size_padded(&label, padding);
        LabeledButton { b: Button::new(Rect::new(pos.x, pos.y, dimensions.x, dimensions.y), press_mode, disabled), label, padding }
    }

    pub fn update(&mut self, mouse_pos: Vec2, mouse_mode: MousePressMode) {
        self.b.update(mouse_pos, mouse_mode);
    }

    pub fn pressed(&mut self) -> bool {
        self.b.pressed()
    }

    pub fn pos(&self) -> Vec2 { Vec2 { x: self.b.rect.x, y: self.b.rect.y } }
    pub fn text_pos(&self) -> Vec2 {
        Vec2::new(self.b.rect.x + self.padding.2, self.b.rect.y + self.padding.0)
    }
}