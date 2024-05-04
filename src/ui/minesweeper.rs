// A minesweeper ui element

use macroquad::math::Rect;

use crate::minesweeper::{Difficulty, DifficultyValues, GameState, Minesweeper};

use super::{elements::{aligned_rect, button, Align}, hash_string, renderer::{DrawShape, Renderer}, spritesheet, state::{ButtonState, State}};

pub struct MinesweeperElement {
    game: Minesweeper,
    difficulty: Difficulty,
    timer: Option<f32>,
    
    // Stores what the last custom input was for the popup
    custom_values: Option<DifficultyValues>,
    requesting_new_game: bool,
}

impl MinesweeperElement {
    pub fn new() -> MinesweeperElement {
        let difficulty = Difficulty::Easy;

        MinesweeperElement {
            game: Minesweeper::new(difficulty),
            difficulty,
            timer: None,

            custom_values: None,
            requesting_new_game: false,
        }
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }
    pub fn custom_values(&self) -> Option<DifficultyValues> {
        self.custom_values
    }
    pub fn game_in_progress(&self) -> bool {
        self.game.state() == GameState::Playing && self.game.turns() != 0
    }

    pub fn requesting_new_game(&mut self) -> bool {
        // If somethings checking that we're requesting a new game, reset our flag, as they'll deal with it
        match self.requesting_new_game {
            true => { self.requesting_new_game = false; true }
            false => false
        }
    }

    pub fn new_game(&mut self, difficulty: Difficulty) {
        self.difficulty = difficulty;
        self.game = Minesweeper::new(difficulty);
        match difficulty {
            Difficulty::Custom(values) => self.custom_values = Some(values),
            _ => (),
        }
    }

    // Area is the area where the minefield and the ui bar at the top will be drawn, it's NOT CLIPPED!
    pub fn update(&mut self, area: Rect, state: &mut State, renderer: &mut Renderer) {
        // Update the timer
        self.timer = match (self.game.turns(), self.game.state()) {
            // If we haven't made a move yet, the time should be None
            (0, _) => None,
            // If the game is being played, increment the timer
            (_, GameState::Playing) => Some(self.timer.unwrap_or(0.0) + macroquad::time::get_frame_time()),
            // Otherwise (meaning we've won or lost) keep the timer frozen, on a valid value
            _ => Some(self.timer.unwrap_or(0.0)),
        };

        // Draw the elements along the top
        // The button makes a new game :P
        if button(
            hash_string(&"hello if ur reading this :3".to_owned()),
            Align::Mid(area.x + area.w / 2.0), Align::Beg(area.y + 3.0),
            19.0, 19.0, false, state, renderer
        ) == ButtonState::Released {
            self.requesting_new_game = true;
        }

        let lower_x = area.x + area.w * (1.0 / 6.0);
        let upper_x = area.x + area.w * (5.0 / 6.0);
        self.bomb_counter(Align::Mid(lower_x), Align::Beg(area.y + 4.0), renderer);
        self.timer(       Align::Mid(upper_x), Align::Beg(area.y + 8.0), renderer);
    }

    fn bomb_counter(&self, x: Align, y: Align, renderer: &mut Renderer) {
        let value = self.game.flags_left();
        // Calculate the minimum number of digits needed to display the bomb count (always being 2 or larger for style purposes: 3)
        let digits = ((f32::log10(self.game.bomb_count() as f32) + 1.0).floor() as usize).max(2);

        let size = spritesheet::counter_size(digits);
        let rect = aligned_rect(x, y, size.x, size.y);

        let draw_shapes = (0..digits)
            // Work out the place value of the current digit
            .map(|i| (i, 10usize.saturating_pow(i as u32)))
            .map(|(i, power_of_ten)| (i,
                match value {
                    // If value is none, draw dashes for every character
                    None => spritesheet::CounterDigit::Dash,
                    // Otherwise draw a number! This doesn't draw leading zeros, however always renders the first digit, even if it's a zero!
                    Some(v) if power_of_ten <= v || i == 0 => spritesheet::CounterDigit::Digit((v / power_of_ten) % 10),
                    _ => spritesheet::CounterDigit::Empty,
                }
            ))
            .map(|(i, digit)| (i, spritesheet::counter_digit(digit)))
            // Render the digits in reverse order so they appear the right way around
            .map(|(i, digit_rect)| DrawShape::image(rect.x + 3.0 + (digit_rect.w + 2.0) * (digits - i - 1) as f32, rect.y + 2.0, digit_rect, None))
            // Last but not least draw the background
            .chain(std::iter::once(DrawShape::nineslice(rect, spritesheet::TIMER_BACKGROUND)));
        renderer.draw_iter(draw_shapes);
    }
    
    fn timer(&self, x: Align, y: Align, renderer: &mut Renderer) {
        let size = spritesheet::TIMER_SIZE;
        let rect = aligned_rect(x, y, size.x, size.y);

        let (digits, colon_lit): ([Option<usize>; 4], bool) = if let Some(time) = self.timer {
            let seconds = (time as usize).min(60*100-1).try_into().unwrap_or(usize::MAX);
            let digits = [
                // Don't display the last digit as a 0 
                if seconds < 60*10 { None } else { Some((seconds / 60) / 10) }, // Tens of minutes
                Some((seconds / 60) % 10),                                      // Minutes
                Some((seconds % 60) / 10),                                      // Tens
                Some(seconds % 10),                                             // Units 
            ];
            // Originally, the colon flashed every half-second, but I found it a bit distracting :/
            (digits, true)
        } else {
            ([None; 4], false)
        };

        let draw_shapes = digits.iter()
            .zip([2.0, 6.0, 12.0, 16.0])
            .map(|(&digit, along)| DrawShape::image(rect.x + along, rect.y + 2.0, spritesheet::timer_digit(digit), None))
            // Draw the colon and the background
            .chain(std::iter::once(DrawShape::image(rect.x + 10.0, rect.y + 2.0, spritesheet::timer_colon(colon_lit), None)))
            .chain(std::iter::once(DrawShape::nineslice(rect, spritesheet::TIMER_BACKGROUND)));
        renderer.draw_iter(draw_shapes);
    }
}