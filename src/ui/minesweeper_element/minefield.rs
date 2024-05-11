use macroquad::{audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound}, camera::{set_camera, Camera2D}, color::WHITE, input::MouseButton, math::{vec2, Rect, Vec2}, texture::{draw_texture_ex, render_target, DrawTextureParams, RenderTarget}};

use crate::{minesweeper::{Difficulty, GameState, Minesweeper, SetFlagMode, Tile}, ui::{elements::{aligned_rect, Align}, hash_string, renderer::{DrawShape, Renderer}, spritesheet, state::State}};

use super::exploder::Exploder;

pub struct Minefield {
    flag_mode: Option<SetFlagMode>,
    losing_tile: Option<usize>,

    render_target: RenderTarget,

    sound_flag:      Option<Sound>,
    sound_explosion: Option<Sound>,
    sound_win:       Option<Sound>,
}

impl Minefield {
    pub async fn new(difficulty: Difficulty) -> Minefield {
        Minefield {
            flag_mode: None,
            losing_tile: None,
            render_target: Minefield::render_target(difficulty),

            sound_flag:      load_sound_from_bytes(include_bytes!("../../../resources/congrats.ogg" )).await.ok(),
            sound_explosion: load_sound_from_bytes(include_bytes!("../../../resources/explosion.ogg")).await.ok(),
            sound_win:       load_sound_from_bytes(include_bytes!("../../../resources/congrats.ogg" )).await.ok(),
        }
    }
    pub fn explosion_sound(&self) -> &Option<Sound> {
        &self.sound_explosion
    }

    pub fn update(&mut self, area: Rect, game: &mut Minesweeper, exploder: &mut Exploder, state: &mut State, renderer: &mut Renderer) {
        renderer.draw(DrawShape::rect(area, macroquad::color::Color::from_rgba(255, 0, 0, 128)));

        let size = vec2(game.width() as f32, game.height() as f32) * 9.0;
        let rect = aligned_rect(Align::Mid(area.x + area.w / 2.0), Align::Mid(area.y + area.h / 2.0), size.x, size.y);
        // Make sure it doesn't go above the area
        let rect = Rect::new(rect.x, f32::max(rect.y, area.y + 2.0), rect.w, rect.h);

        let id = hash_string(&"if you're reading this... say hi :3".to_owned());

        if state.hot_item.assign_if_none_and(id, state.mouse_in_rect(rect)) {
            let hovered_tile_coord = ((state.mouse_pos() - rect.point()) / 9.0).floor();
            let selected_tile = hovered_tile_coord.y as usize * game.width() + hovered_tile_coord.x as usize;

            // Draw the selector thingy
            let selector_pos = rect.point() + hovered_tile_coord * 9.0 - 1.0;
            renderer.draw(DrawShape::image(selector_pos.x, selector_pos.y, spritesheet::MINEFIELD_SELECTED, None));
            
            // Interacting
            let any_mouse_down = state.mouse_down(MouseButton::Left) || state.mouse_down(MouseButton::Middle) || state.mouse_down(MouseButton::Right);
            state.active_item.assign_if_none_and(id, any_mouse_down);
            if state.active_item == id {
                
                let prev_game_state = game.state();

                // Digging
                if state.mouse_released(MouseButton::Left) {
                    game.dig(selected_tile);
                }
                // If about to dig, draw a tile being dug
                if state.mouse_down(MouseButton::Left) && game.diggable(selected_tile) {
                    renderer.draw(DrawShape::image(selector_pos.x + 1.0, selector_pos.y + 1.0, spritesheet::minefield_tile(1), None));
                }

                // Flagging
                if state.mouse_pressed(MouseButton::Right) {
                    self.flag_mode = match game.board().get(selected_tile).is_some_and(|t| *t == Tile::Flag) {
                        true  => Some(SetFlagMode::Remove),
                        false => Some(SetFlagMode::Flag),
                    }
                }
                if let Some(flag_mode) = self.flag_mode {
                    if game.set_flag(flag_mode, selected_tile) {
                        // TODO: Make playing sounds look less ugly
                        if let Some(sound_flag) = &self.sound_flag {
                            play_sound(sound_flag, PlaySoundParams::default());
                        }
                    }
                }
                // We only want to set flags once, and remove flags when holding the mouse down.
                if matches!(self.flag_mode, Some(SetFlagMode::Flag)) || state.mouse_released(MouseButton::Right) {
                    self.flag_mode = None;
                }

                // TODO: Chording

                // If we've lost on this frame, start exploding bombs!
                if prev_game_state.is_playing() && game.state().is_lose() {
                    exploder.initialise(selected_tile, game);
                }
                // TODO: Winning?
            }
        }
        // Explode :3
        if game.state() == GameState::Lose {
            exploder.update(self.explosion_sound(), renderer);
        }

        // Rendering the board
        // Drawing the minefield to a texture is much better than a million billion trillion renderer.draw(DrawShape)s
        set_camera(&Camera2D {
            zoom: 2.0 / self.render_target.texture.size(),
            target: self.render_target.texture.size() / 2.0,
            render_target: Some(self.render_target.clone()),
            ..Default::default()
        });
        let texture = &renderer.texture();
        let draw_tile = |index: usize, id: usize| {
            let pos = 9.0 * Vec2::new(
                (index % game.width()) as f32,
                (index / game.width()) as f32
            );
            draw_texture_ex(texture, pos.x, pos.y, WHITE, DrawTextureParams {
                source: Some(spritesheet::minefield_tile(id)),
                ..Default::default()
            });
        };
        // Draw each of the tiles
        for (i, t) in game.board().iter().enumerate() {
            // Draw the background of the tile
            draw_tile(i, match t {
                _ if self.losing_tile == Some(i)                        => 4, // The losing tile
                Tile::Dug | Tile::Numbered(_)                           => 1, // A dug tile
                Tile::Flag if exploder.index_exploded(&i) != Some(true) => 0, // A flag that's not got an exploded bomb below it
                _ if exploder.contains(&i)                              => 1, // A bomb or a tile being chorded
                _                                                       => 0, // Unopened
            });
            // Draw the icon on top of the tile
            match (t, exploder.index_exploded(&i)) {
                (Tile::Flag, None) if game.state().is_lose() => Some(6), // Incorrect flag
                (Tile::Flag, Some(true))                     => Some(7), // Exploded flag
                (Tile::Flag, _)                              => Some(5), // Flag
                (_, Some(false))                             => Some(2), // Unexploded bomb
                (_, Some(true))                              => Some(3), // Exploded bomb
                (Tile::Numbered(n), _)                       => Some(*n as usize + 7), // Number
                _ => None,
            }.map(|id| draw_tile(i, id));
        }
        macroquad::shapes::draw_circle(4.5, 4.5, 4.5, macroquad::color::RED);

        // Draw the texture and the border with the renderer
        let border_rect = Rect::new(rect.x - 2.0, rect.y - 2.0, rect.w + 4.0, rect.h + 4.0);
        renderer.draw(DrawShape::texture(rect.x, rect.y, self.render_target.texture.clone()));
        renderer.draw(DrawShape::nineslice(border_rect, spritesheet::MINEFIELD_BORDER));
    }

    pub fn new_game(&mut self, difficulty: Difficulty) {
        self.flag_mode     = None;
        self.losing_tile   = None;
        self.render_target = Minefield::render_target(difficulty);
    }

    fn render_target(difficulty: Difficulty) -> RenderTarget {
        let r = render_target(difficulty.values().width() as u32 * 9, difficulty.values().height() as u32 * 9);
        r.texture.set_filter(macroquad::texture::FilterMode::Nearest);
        r
    }
}