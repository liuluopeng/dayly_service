// 小游戏逻辑绑定——状态由 Rust 侧持有（thread_local 单例），Dart 端负责渲染与交互。
// 游戏规则/棋盘逻辑全部复用 common::front_can_do（与 Vue/src-wasm 同一份实现）。

use common::front_can_do::game2048::Game2048;
use common::front_can_do::minesweeper::Minesweeper;
use common::front_can_do::snake::Snake;
use common::front_can_do::tetris::Tetris;
use std::cell::RefCell;

thread_local! {
    static GAME_2048: RefCell<Game2048> = RefCell::new(Game2048::new());
    static SNAKE: RefCell<Snake> = RefCell::new(Snake::new());
    static MINESWEEPER: RefCell<Minesweeper> = RefCell::new(Minesweeper::new());
    static TETRIS: RefCell<Tetris> = RefCell::new(Tetris::new());
}

#[flutter_rust_bridge::frb]
pub struct Game2048State {
    pub board: Vec<u32>,
    pub score: u64,
    pub over: bool,
}

#[flutter_rust_bridge::frb]
pub struct SnakeState {
    pub cells: Vec<u8>,
    pub score: u32,
    pub over: bool,
}

#[flutter_rust_bridge::frb]
pub struct MinesweeperState {
    pub cells: Vec<u8>,
    pub revealed: Vec<bool>,
    pub flagged: Vec<bool>,
    pub over: bool,
    pub won: bool,
}

#[flutter_rust_bridge::frb]
pub struct TetrisState {
    pub board: Vec<u8>,
    pub score: u32,
    pub over: bool,
}

// ─── 2048 ───

pub fn game2048_new() {
    GAME_2048.with(|g| *g.borrow_mut() = Game2048::new());
}

pub fn game2048_get() -> Game2048State {
    GAME_2048.with(|g| {
        let g = g.borrow();
        Game2048State {
            board: g.board.iter().flatten().copied().collect(),
            score: g.score,
            over: g.over,
        }
    })
}

pub fn game2048_move(dir: String) -> Game2048State {
    GAME_2048.with(|g| g.borrow_mut().move_dir(&dir));
    game2048_get()
}

pub fn game2048_undo() -> Game2048State {
    GAME_2048.with(|g| {
        g.borrow_mut().undo();
    });
    game2048_get()
}

pub fn game2048_can_move(dir: String) -> bool {
    GAME_2048.with(|g| g.borrow().can_move(&dir))
}

// ─── 贪吃蛇 ───

pub fn snake_new() {
    SNAKE.with(|s| *s.borrow_mut() = Snake::new());
}

pub fn snake_get() -> SnakeState {
    SNAKE.with(|s| {
        let s = s.borrow();
        SnakeState {
            cells: s.cells.iter().flatten().copied().collect(),
            score: s.score,
            over: s.over,
        }
    })
}

pub fn snake_set_dir(dir: String) {
    SNAKE.with(|s| s.borrow_mut().set_dir(&dir));
}

pub fn snake_tick() -> SnakeState {
    SNAKE.with(|s| s.borrow_mut().tick());
    snake_get()
}

// ─── 扫雷 ───

pub fn minesweeper_new() {
    MINESWEEPER.with(|m| *m.borrow_mut() = Minesweeper::new());
}

pub fn minesweeper_get() -> MinesweeperState {
    MINESWEEPER.with(|m| {
        let m = m.borrow();
        MinesweeperState {
            cells: m.cells.iter().flatten().copied().collect(),
            revealed: m.revealed.iter().flatten().copied().collect(),
            flagged: m.flagged.iter().flatten().copied().collect(),
            over: m.over,
            won: m.won,
        }
    })
}

pub fn minesweeper_click(x: usize, y: usize) -> MinesweeperState {
    MINESWEEPER.with(|m| m.borrow_mut().click(x, y));
    minesweeper_get()
}

pub fn minesweeper_toggle_flag(x: usize, y: usize) -> MinesweeperState {
    MINESWEEPER.with(|m| m.borrow_mut().toggle_flag(x, y));
    minesweeper_get()
}

// ─── 俄罗斯方块 ───

pub fn tetris_new() {
    TETRIS.with(|t| *t.borrow_mut() = Tetris::new());
}

pub fn tetris_get() -> TetrisState {
    TETRIS.with(|t| {
        let t = t.borrow();
        TetrisState {
            board: t.board.iter().flatten().copied().collect(),
            score: t.score,
            over: t.over,
        }
    })
}

pub fn tetris_move(dir: String) -> TetrisState {
    TETRIS.with(|t| t.borrow_mut().move_piece(&dir));
    tetris_get()
}

pub fn tetris_tick() -> TetrisState {
    TETRIS.with(|t| t.borrow_mut().tick());
    tetris_get()
}
