use common::front_can_do::game2048::Game2048;
use wasm_bindgen::prelude::*;

static mut GAME: Option<Game2048> = None;

fn with_game<F, R>(f: F) -> R
where
    F: FnOnce(&mut Game2048) -> R,
{
    unsafe {
        let g = GAME.get_or_insert_with(Game2048::new);
        f(g)
    }
}

#[wasm_bindgen]
pub fn game2048_init() {
    unsafe { GAME = Some(Game2048::new()); }
}

#[wasm_bindgen]
pub fn game2048_board() -> Vec<u32> {
    with_game(|g| g.board.iter().flat_map(|r| r.iter()).copied().collect())
}

#[wasm_bindgen]
pub fn game2048_score() -> u32 {
    with_game(|g| g.score as u32)
}

#[wasm_bindgen]
pub fn game2048_max_tile() -> u32 {
    with_game(|g| g.max_tile())
}

#[wasm_bindgen]
pub fn game2048_won() -> bool {
    with_game(|g| g.won)
}

#[wasm_bindgen]
pub fn game2048_over() -> bool {
    with_game(|g| g.over)
}

#[wasm_bindgen]
pub fn game2048_move(dir: &str) -> bool {
    with_game(|g| g.move_dir(dir))
}

#[wasm_bindgen]
pub fn game2048_undo() -> bool {
    with_game(|g| g.undo())
}
