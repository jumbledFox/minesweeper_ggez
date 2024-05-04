use std::collections::VecDeque;

use macroquad::math::{vec2, Rect, Vec2};

use super::{hash_string, renderer::{DrawShape, Renderer}, spritesheet, state::{ButtonState, Id, SelectedItem, State}};

#[derive(Default)]
pub struct Menubar {
    height: f32,

    items: Vec<(Option<String>, Id, f32)>,
    current_item_index: usize,

    prev_id: Option<Id>,
    next_id: Option<Id>,

    item_current: Option<Id>,
    item_current_prev: Option<Id>,

    item_current_x: f32,
    item_next_x: f32,

    dropdown_width: f32,
    dropdown_id: Id,
    
    dropdown_start_pos: Vec2,
    dropdown_current_pos: Vec2,
    dropdown_next_pos: Vec2,
    
    dropdown_max_offset: Vec2,
    dropdown_rect: Rect,
}

impl Menubar {
    pub fn height(&self) -> f32 {
        self.height
    } 

    pub fn begin(&mut self, next_id: Option<Id>, prev_id: Option<Id>, items: &mut VecDeque<(String, f32)>) {
        self.prev_id = prev_id;
        self.next_id = next_id;

        // generate ids for each item
        self.items.clear();
        for _ in 0..items.len() {
            let (t, w) = match items.pop_front() {
                Some((t, w)) => (t, w),
                None => continue,
            };
            let id = hash_string(&t);
            self.items.push((Some(t), id, w));
        }

        self.current_item_index = 0;
        self.item_current_prev = self.item_current;
        self.item_next_x = 0.0;
    }

    pub fn finish(&mut self, state: &mut State, renderer: &mut Renderer) {
        renderer.draw(super::renderer::DrawShape::Rect {
            x: self.item_next_x,
            y: 0.0,
            w: state.screen_size().x - self.item_next_x,
            h: self.height,
            color: spritesheet::menubar_colors(false).0,
        });

        // If anywhere that's not the dropdown has been clicked, deselect the menubar
        if self.item_current.is_some() && self.item_current_prev.is_some()
        && state.mouse_pressed(macroquad::input::MouseButton::Left) && !state.mouse_in_rect(self.dropdown_rect) {
            state.active_item = SelectedItem::Unavailable;
            self.item_current = None;
        }
    }

    pub fn item(&mut self, state: &mut State, renderer: &mut Renderer) -> bool {
        let prev_id = match self.current_item_index.checked_sub(1).and_then(|i| self.items.get(i)) {
            Some((_, id, _)) => Some(*id),
            None => self.prev_id,
        };
        let next_id = match self.current_item_index.checked_add(1).and_then(|i| self.items.get(i)) {
            Some((_, id, _)) => Some(*id),
            None => self.next_id,
        };

        let (text, id, dropdown_width) = match self.items.get_mut(self.current_item_index) {
            Some((t, id, d)) => (t.take().unwrap(), *id, *d), // ????
            None => return false,
        };
        // allows for 100 dropdown items... maybe bad??! idgaf
        self.current_item_index = self.current_item_index.wrapping_add(1);
        self.dropdown_id = id.wrapping_add(1);

        self.item_current_x = self.item_next_x;
        
        let size = renderer.text_renderer.text_size(&text, None) + vec2(4.0, 2.0);
        self.item_next_x += size.x;
        self.height = self.height.max(size.y);
        
        self.dropdown_start_pos = vec2(self.item_current_x, self.height) + 1.0;
        self.dropdown_next_pos = self.dropdown_start_pos;
        self.dropdown_width = dropdown_width;
        self.dropdown_max_offset = vec2(dropdown_width, 0.0);
    
        let rect = Rect::new(self.item_current_x, 0.0, size.x, size.y);
        let hovered = state.mouse_in_rect(rect);
        let button_state = state.button_state(id, prev_id, next_id, hovered, false, false);
        // If a dropdown is open and the mouse has hovered this menu item, or if this menu item's been clicked, set THIS to be the current one.
        if (button_state == ButtonState::Hovered && self.item_current.is_some()) || button_state == ButtonState::Clicked {
            self.item_current = Some(id);
        }
        
        let colors = spritesheet::menubar_colors(self.item_current == Some(id) || state.hot_item == id);

        renderer.draw(super::renderer::DrawShape::text(rect.x + 2.0, rect.y + 1.0, text, colors.1));
        renderer.draw(super::renderer::DrawShape::rect(rect, colors.0));

        self.item_current == Some(id)
    }

    pub fn finish_item(&mut self, state: &mut State, renderer: &mut Renderer) {
        // If the dropdown doesn't go down at all, it has not dropdown items and therefore doesn't have a rect,
        // so we don't really care about doing anything
        if self.dropdown_max_offset.y == 0.0 {
            self.dropdown_rect = Rect::new(0.0, 0.0, 0.0, 0.0);
            return;
        }

        self.dropdown_rect =  Rect::new(
            self.item_current_x,
            self.height,
            self.dropdown_max_offset.x + 2.0,
            self.dropdown_max_offset.y + 2.0 - self.dropdown_start_pos.y,
        );

        // Draw the dropdown box and it's shadow
        renderer.draw(DrawShape::nineslice(self.dropdown_rect, spritesheet::DROPDOWN_BACKGROUND));
        renderer.draw(DrawShape::rect(self.dropdown_rect.offset(Vec2::splat(3.0)), spritesheet::SHADOW));

        // Make it so the box captures the hot item
        if state.hot_item.is_none() && state.mouse_in_rect(self.dropdown_rect) {
            state.hot_item = SelectedItem::Unavailable;
        }
    }

    fn dropdown_next_descend(&mut self, amount: f32) {
        self.dropdown_next_pos.y += amount;
        self.dropdown_max_offset.y = self.dropdown_max_offset.y.max(self.dropdown_next_pos.y);
    }

    fn dropdown_item(&mut self, text: String, icon: bool, state: &mut State, renderer: &mut Renderer) -> bool {
        self.dropdown_current_pos = self.dropdown_next_pos;
        let rect = Rect::new(
            self.dropdown_current_pos.x,
            self.dropdown_current_pos.y,
            self.dropdown_width,
            renderer.text_renderer.text_size(&text, None).y + 3.0,
        );
        self.dropdown_next_descend(rect.h);

        // I do different button logic here because they behave slightly differently than normal buttons
        let id = self.dropdown_id;
        self.dropdown_id = id.wrapping_add(1);

        let mouse_down = state.mouse_down(macroquad::input::MouseButton::Left);

        if state.hot_item.assign_if_none_and(id, state.mouse_in_rect(rect)) {
            if mouse_down {
                state.active_item.assign(id);
            }
        }

        let released = state.hot_item == id && state.active_item == id && !mouse_down;
        if released {
            self.item_current = None;
        }

        let colors = spritesheet::menubar_colors(state.hot_item == id);
        if icon {
            renderer.draw(super::renderer::DrawShape::Rect {
                x: rect.x + 2.0,
                y: rect.y + 3.0,
                w: 3.0,
                h: 3.0,
                color: colors.1,
            })
        }
        renderer.draw(super::renderer::DrawShape::text(rect.x + 7.0, rect.y + 2.0, text, colors.1 ));
        renderer.draw(super::renderer::DrawShape::rect(rect, colors.0));

        released
    }

    pub fn dropdown(&mut self, text: String, state: &mut State, renderer: &mut Renderer) -> bool {
        self.dropdown_item(text, false, state, renderer)
    }
    pub fn dropdown_radio(&mut self, text: String, qualifier: bool, state: &mut State, renderer: &mut Renderer) -> bool {
        self.dropdown_item(text, qualifier, state, renderer)
    }
    pub fn dropdown_toggle(&mut self, text: String, value: &mut bool, state: &mut State, renderer: &mut Renderer) -> bool {
        let pressed = self.dropdown_item(text, *value, state, renderer);
        if pressed { *value = !*value; }
        pressed
    }

    pub fn dropdown_separator(&mut self, renderer: &mut Renderer) {
        self.dropdown_current_pos = self.dropdown_next_pos;
        let source = spritesheet::DROPDOWN_SEPARATOR;
        let dest = Rect::new(
            self.dropdown_current_pos.x + 1.0,
            self.dropdown_current_pos.y,
            self.dropdown_width - 2.0,
            source.h,
        );
        self.dropdown_next_descend(dest.h);
        renderer.draw(super::renderer::DrawShape::image_rect(dest, source, None));
    }
}