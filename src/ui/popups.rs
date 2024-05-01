use macroquad::{input::MouseButton, math::{vec2, Rect, Vec2}, miniquad::window::order_quit};

use crate::minesweeper::Difficulty;

use super::{elements::{align_end, button, button_text}, hash_string, menubar::Menubar, renderer::{DrawShape, Renderer}, spritesheet, state::{ButtonState, Id, State}};

#[derive(Default)]
pub struct Popups {
    popups: Vec<Popup>,
    popup_drag_offset: Vec2,
    // TODO: 
    // i COULD make the PopupKind enum store this, although I like the idea of what you wrote being remembered.
    // custom_fields: (String, String, String),
}

impl Popups {
    pub fn add(&mut self, kind: PopupKind, state: &State) {
        self.popups.retain(|p| std::mem::discriminant(&p.kind) != std::mem::discriminant(&kind));
        self.popups.push(Popup::new(kind, state));
    }

    pub fn update(&mut self, state: &mut State, menubar: &Menubar, renderer: &mut Renderer) {
        let mut popup_returns: Vec<PopupReturn> = Vec::new();
        let mut close = None; 
        let mut front = None; 

        for (i, popup) in self.popups.iter_mut().enumerate().rev() {
            let (action, return_value) = popup.update(&mut self.popup_drag_offset, state, menubar, renderer);
            match action {
                PopupAction::Close => close = Some(i),
                PopupAction::Front => front = Some(i),
                _ => ()
            }
            if let Some(r) = return_value {
                popup_returns.push(r);
            }
        }

        if let Some(close) = close {
            self.popups.remove(close);
        }
        if let Some(front) = front {
            let popup = self.popups.remove(front);
            self.popups.push(popup);
        }

        // for p in popup_returns {
        //     match p {
        //         PopupReturn::NewGame { difficulty } => ()
        //     }
        // }
    }
}

pub enum PopupKind {
    NewGame { difficulty: Difficulty },
    Custom,
    About,
    Win,
    Exit,
}

pub enum PopupReturn {
    NewGame { difficulty: Difficulty },
}

pub enum PopupAction {
    None,
    Front,
    Close,
}

pub struct Popup {
    pos: Vec2,
    size: Vec2,
    title: String,
    kind: PopupKind,
    id: Id,
}

impl Popup {
    pub fn new(kind: PopupKind, state: &State) -> Popup {
        let (title, size) = match kind {
            PopupKind::NewGame{..} => ("New game", vec2( 90.0, 40.0)),
            PopupKind::Custom      => ("Custom",   vec2( 78.0, 58.0)),
            PopupKind::About       => ("About",    vec2(100.0, 80.0)),
            PopupKind::Win         => ("You win!", vec2( 70.0, 40.0)),
            PopupKind::Exit        => ("Exit",     vec2( 70.0, 40.0)),
        };
        let pos = (state.screen_size() - size) / 2.0;

        Popup { pos, size, title: title.to_owned(), kind, id: hash_string(&format!("popup!!{}", macroquad::miniquad::date::now())) }
    }

    pub fn update(&mut self, popup_drag_offset: &mut Vec2, state: &mut State, menubar: &Menubar, renderer: &mut Renderer) -> (PopupAction, Option<PopupReturn>) {
        let titlebar_height = renderer.text_renderer.text_size(&self.title, None).y + 3.0;
        self.pos = self.pos
            .min(state.screen_size() - self.size)
            .max(vec2(0.0, menubar.height()));

        let titlebar = Rect::new(self.pos.x, self.pos.y,              self.size.x, titlebar_height);
        let body     = Rect::new(self.pos.x, self.pos.y + titlebar.h, self.size.x, self.size.y - titlebar.h);

        let id = self.id;

        let active_before = state.active_item;
        let mut close = Popup::close_button(id.wrapping_add(1), titlebar, state, renderer);

        let mut return_value = None;

        // Elements
        match self.kind {
            PopupKind::NewGame { difficulty } => {
                // button clicked
                if false {
                    close = true;
                    return_value = Some(PopupReturn::NewGame { difficulty })
                }
            }
            PopupKind::Custom => {

            }
            PopupKind::About => {
                renderer.draw(DrawShape::text(body.x + 3.0, body.y + 3.0,
                    // TODO: Add text styling?
                    "Minesweeper\n\njumbledFox - 2024\n\nMade in Rust and the\nMacroquad framework.\n\nOpen source on Github!\njumbledFox.github.io".to_owned(),
                spritesheet::POPUP_BODY_TEXT));
            }
            PopupKind::Win => {
                renderer.draw(DrawShape::text(body.x + 3.0, body.y + 3.0, "You win,\ncongratulations!".to_owned(), spritesheet::POPUP_BODY_TEXT));
                if false {
                    close = true;
                }
            }
            PopupKind::Exit => {
                renderer.draw(DrawShape::text(body.x + 3.0, body.y + 3.0, "Are you sure you\nwant to exit?".to_owned(), spritesheet::POPUP_BODY_TEXT));
                if button_text(id.wrapping_add(2), "Exit".to_owned(), align_end(body.right()-3.0), align_end(body.bottom()-3.0), state, renderer) == ButtonState::Released {
                    order_quit();
                }
                close = close || button_text(id.wrapping_add(3), "Cancel".to_owned(), align_end(body.right()-25.0), align_end(body.bottom()-3.0), state, renderer) == ButtonState::Released;
            }
        }

        // Dragging the popup around
        let hovered = state.mouse_in_rect(titlebar) || state.mouse_in_rect(body);

        if state.hot_item.assign_if_none_and(id, hovered) {
            if state.active_item.assign_if_none_and(id, state.mouse_down(MouseButton::Left)) {
                *popup_drag_offset = state.mouse_pos() - self.pos;
            }
        }

        if state.active_item == id {
            state.hot_item = super::state::SelectedItem::Unavailable;
            self.pos = state.mouse_pos() - *popup_drag_offset;
        }
        
        renderer.draw(DrawShape::text(titlebar.x + 2.0, titlebar.y + 2.0, self.title.clone(), spritesheet::POPUP_TITLE_TEXT));
        renderer.draw(DrawShape::nineslice(titlebar, spritesheet::POPUP_TITLE));
        renderer.draw(DrawShape::nineslice(body,     spritesheet::POPUP_BODY));
        renderer.draw(DrawShape::rect(body.combine_with(titlebar).offset(vec2(3.0, 3.0)), spritesheet::SHADOW));
        
        let action = match (close, active_before.is_none() && !state.active_item.is_none()) {
            (true, _) => PopupAction::Close,
            (_, true) => PopupAction::Front,
            _ => PopupAction::None,
        };
        (action, return_value)
    }

    fn close_button(id: Id, titlebar: Rect, state: &mut State, renderer: &mut Renderer) -> bool {
        let pos = titlebar.point() + vec2(titlebar.w - 8.0, 1.0);
        let rect = Rect::new(pos.x, pos.y, 7.0, 7.0);
        let button_state = state.button_state(id, state.mouse_in_rect(rect), false, false);

        let colors = (
            spritesheet::popup_close_color(button_state != ButtonState::Idle),
            spritesheet::popup_close_color(button_state == ButtonState::Idle),
        );
        renderer.draw(DrawShape::image(pos.x + 2.0, pos.y + 2.0, spritesheet::POPUP_CLOSE, Some(colors.0)));
        renderer.draw(DrawShape::rect(rect, colors.1));

        button_state == ButtonState::Released
    }
}