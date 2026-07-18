use common::front_can_do::snake::Snake;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! { static GAME: RefCell<Snake> = RefCell::new(Snake::new()); }

#[wasm_bindgen]
pub fn snake_init() { GAME.with(|g| *g.borrow_mut() = Snake::new()); }

#[wasm_bindgen]
pub fn snake_tick() { GAME.with(|g| g.borrow_mut().tick()); }

#[wasm_bindgen]
pub fn snake_set_dir(dir: &str) { GAME.with(|g| g.borrow_mut().set_dir(dir)); }

#[wasm_bindgen]
pub fn snake_board() -> Vec<u8> { GAME.with(|g| g.borrow().cells.iter().flat_map(|r| r.to_vec()).collect()) }

#[wasm_bindgen]
pub fn snake_score() -> u32 { GAME.with(|g| g.borrow().score) }

#[wasm_bindgen]
pub fn snake_over() -> bool { GAME.with(|g| g.borrow().over) }
