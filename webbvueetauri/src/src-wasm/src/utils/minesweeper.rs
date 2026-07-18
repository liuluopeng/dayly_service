use common::front_can_do::minesweeper::Minesweeper;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! { static GAME: RefCell<Minesweeper> = RefCell::new(Minesweeper::new()); }

#[wasm_bindgen]
pub fn ms_init() { GAME.with(|g| *g.borrow_mut() = Minesweeper::new()); }

#[wasm_bindgen]
pub fn ms_click(x: usize, y: usize) { GAME.with(|g| g.borrow_mut().click(x, y)); }

#[wasm_bindgen]
pub fn ms_toggle_flag(x: usize, y: usize) { GAME.with(|g| g.borrow_mut().toggle_flag(x, y)); }

#[wasm_bindgen]
pub fn ms_cells() -> Vec<u8> { GAME.with(|g| g.borrow().cells.iter().flat_map(|r| r.to_vec()).collect()) }

#[wasm_bindgen]
pub fn ms_revealed() -> Vec<u8> { GAME.with(|g| g.borrow().revealed.iter().flat_map(|r| r.iter().map(|&b| b as u8)).collect()) }

#[wasm_bindgen]
pub fn ms_flagged() -> Vec<u8> { GAME.with(|g| g.borrow().flagged.iter().flat_map(|r| r.iter().map(|&b| b as u8)).collect()) }

#[wasm_bindgen]
pub fn ms_over() -> bool { GAME.with(|g| g.borrow().over) }

#[wasm_bindgen]
pub fn ms_won() -> bool { GAME.with(|g| g.borrow().won) }

#[wasm_bindgen]
pub fn ms_flag_count() -> usize { GAME.with(|g| g.borrow().flag_count()) }
