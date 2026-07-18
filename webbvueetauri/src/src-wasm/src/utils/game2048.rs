use common::front_can_do::game2048::Game2048;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! { static GAME: RefCell<Game2048> = RefCell::new(Game2048::new()); }

#[wasm_bindgen]
pub fn game2048_init() { GAME.with(|g| *g.borrow_mut() = Game2048::new()); }

#[wasm_bindgen]
pub fn game2048_board() -> Vec<u32> { GAME.with(|g| g.borrow().board.iter().flat_map(|r| r.iter()).copied().collect()) }

#[wasm_bindgen]
pub fn game2048_score() -> u32 { GAME.with(|g| g.borrow().score as u32) }

#[wasm_bindgen]
pub fn game2048_max_tile() -> u32 { GAME.with(|g| g.borrow().max_tile()) }

#[wasm_bindgen]
pub fn game2048_won() -> bool { GAME.with(|g| g.borrow().won) }

#[wasm_bindgen]
pub fn game2048_over() -> bool { GAME.with(|g| g.borrow().over) }

#[wasm_bindgen]
pub fn game2048_move(dir: &str) -> bool { GAME.with(|g| g.borrow_mut().move_dir(dir)) }

#[wasm_bindgen]
pub fn game2048_undo() -> bool { GAME.with(|g| g.borrow_mut().undo()) }
