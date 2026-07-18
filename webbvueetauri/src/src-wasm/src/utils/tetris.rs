use common::front_can_do::tetris::Tetris;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! { static GAME: RefCell<Tetris> = RefCell::new(Tetris::new()); }

#[wasm_bindgen]
pub fn tetris_init() { GAME.with(|g| *g.borrow_mut() = Tetris::new()); }

#[wasm_bindgen]
pub fn tetris_tick() { GAME.with(|g| g.borrow_mut().tick()); }

#[wasm_bindgen]
pub fn tetris_move(dir: &str) { GAME.with(|g| g.borrow_mut().move_piece(dir)); }

#[wasm_bindgen]
pub fn tetris_board() -> Vec<u8> { GAME.with(|g| g.borrow().board.iter().flat_map(|r| r.to_vec()).collect()) }

#[wasm_bindgen]
pub fn tetris_score() -> u32 { GAME.with(|g| g.borrow().score) }

#[wasm_bindgen]
pub fn tetris_over() -> bool { GAME.with(|g| g.borrow().over) }
