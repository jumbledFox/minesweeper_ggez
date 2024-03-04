use std::time::Instant;

use rand::prelude::*;

#[derive(PartialEq)]
pub enum GameState {
    Prelude, Playing, Win, Lose
}
#[derive(PartialEq, Clone)]
pub enum TileType {
    Unopened, Dug, Flag,
}

impl rand::distributions::Distribution<TileType> for rand::distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileType {
        match rng.gen_range(0..=2) { // rand 0.8
            0 => TileType::Unopened,
            1 => TileType::Dug,
            _ => TileType::Dug,
        }
    }
}

// TODO: Maybe make this struct only hold game logic related things and be pure of rendering
pub struct Minesweeper {
    // Game logic
    pub width : usize,
    pub height: usize,
    pub bomb_count: usize,

    pub board: Vec<TileType>,
    pub bombs: Vec<usize>,
    pub neighbour_count: Vec<u8>,

    pub state: GameState,
    pub start_time: Instant,
    
    // Rendering
    // Used for the lose animation, holds the indexes of all of the bombs that should be drawn as an explosion rather than a bomb.
    // pub selected_tile: Option<usize>,
    pub exploded_bombs: Vec<usize>,
}

impl Minesweeper {
    pub fn new(width: usize, height: usize, bomb_count: usize) -> Minesweeper {
        let size = width*height;

        if bomb_count > size.saturating_sub(9) {
            println!("Bomb count is bigger than max! you silly goose :P");
        }
        let bomb_count = bomb_count.min(size - 9);

        let mut board = Vec::with_capacity(size);
        for _ in 0..size {
            // board.push(rand::random());
            board.push(TileType::Unopened);
        }
        // The bombs and neighbour_count values are only populated properly after the first move.
        // This is because we want the cell at the user's first dig, and all of it's neighbours, to never be a bomb.
        // So before that they're empty / filled with dummy values.

        let bombs = vec![0; bomb_count];

        let mut neighbour_count = Vec::with_capacity(size);
        for _ in 0..size {
            neighbour_count.push(thread_rng().gen_range(0..=3));
        }

        Minesweeper { width, height, bomb_count,
            board, bombs, neighbour_count,
            // selected_tile: Some(width + 1),
            state: GameState::Prelude, start_time: Instant::now(),
            exploded_bombs: vec![]
        }
    }

    pub fn dig(&mut self, index: usize) -> bool {
        // If the tile is valid and diggable...
        if let Some(tile) = self.board.get_mut(index) {
            if *tile == TileType::Unopened {
                if self.state == GameState::Prelude {
                    // Generate the board, bombs and stuff
        
                    // Start the timer
                    self.start_time = Instant::now();
                    self.state = GameState::Playing;
                }
                *tile = TileType::Dug;
                return true;
            }
        }
        false
    }

    // Toggle a flag at a position, checks if the index is valid, as well as if flagging that tile is a valid move
    // Returns if the operation was successful
    pub fn set_flag(&mut self, erasing_flags: bool, index: usize) -> bool {
        // If the index is valid
        if let Some(tile) = self.board.get_mut(index) {
            // Add or remove a flag, depending on 'erasing_flags'
            match erasing_flags {
                true  => if *tile == TileType::Flag { *tile = TileType::Unopened; return true; },
                false => if *tile == TileType::Unopened { *tile = TileType::Flag; return true; },
            }
        }
        false
    }
}