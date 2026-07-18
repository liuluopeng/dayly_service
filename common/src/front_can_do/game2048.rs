use rand::Rng;

const SIZE: usize = 4;

#[derive(Clone)]
pub struct Game2048 {
    pub board: [[u32; SIZE]; SIZE],
    pub score: u64,
    pub won: bool,
    pub over: bool,
    history: Vec<([[u32; SIZE]; SIZE], u64)>,
}

impl Game2048 {
    pub fn new() -> Self {
        let mut g = Game2048 { board: [[0; SIZE]; SIZE], score: 0, won: false, over: false, history: Vec::new() };
        g.spawn(); g.spawn(); g
    }

    fn spawn(&mut self) {
        let mut rng = rand::thread_rng();
        let empty: Vec<(usize, usize)> = (0..SIZE).flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[r][c] == 0).collect();
        if let Some(&(r, c)) = empty.get(rng.gen_range(0..empty.len())) {
            self.board[r][c] = if rng.gen_range(0.0f64..1.0) < 0.9 { 2 } else { 4 };
        }
    }

    fn slide_row(row: &[u32; SIZE]) -> ([u32; SIZE], u64) {
        let mut out = [0u32; SIZE]; let mut i = 0; let mut score = 0; let mut j = 0;
        while j < SIZE {
            if row[j] == 0 { j += 1; continue; }
            let mut k = j + 1;
            while k < SIZE && row[k] == 0 { k += 1; }
            if k < SIZE && row[k] == row[j] { out[i] = row[j] * 2; score += out[i] as u64; j = k + 1; }
            else { out[i] = row[j]; j += 1; }
            i += 1;
        }
        (out, score)
    }

    fn transpose(b: &[[u32; SIZE]; SIZE]) -> [[u32; SIZE]; SIZE] {
        let mut t = [[0u32; SIZE]; SIZE];
        for r in 0..SIZE { for c in 0..SIZE { t[c][r] = b[r][c]; }}
        t
    }

    fn reverse(b: &[[u32; SIZE]; SIZE]) -> [[u32; SIZE]; SIZE] {
        let mut rev = *b; for r in 0..SIZE { rev[r].reverse(); } rev
    }

    fn apply(board: &[[u32; SIZE]; SIZE], dir: &str) -> ([[u32; SIZE]; SIZE], u64) {
        let mut b = *board;
        if dir == "up" || dir == "down" { b = Self::transpose(&b); }
        if dir == "right" || dir == "down" { b = Self::reverse(&b); }
        let mut total = 0;
        for r in 0..SIZE { let (nr, s) = Self::slide_row(&b[r]); b[r] = nr; total += s; }
        if dir == "right" || dir == "down" { b = Self::reverse(&b); }
        if dir == "up" || dir == "down" { b = Self::transpose(&b); }
        (b, total)
    }

    fn has_adjacent(board: &[[u32; SIZE]; SIZE]) -> bool {
        for r in 0..SIZE { for c in 0..SIZE {
            let v = board[r][c];
            if v == 0 { return true; }
            if r + 1 < SIZE && board[r + 1][c] == v { return true; }
            if c + 1 < SIZE && board[r][c + 1] == v { return true; }
        }}
        false
    }

    pub fn can_move(&self, dir: &str) -> bool { let (nb, _) = Self::apply(&self.board, dir); nb != self.board }

    pub fn move_dir(&mut self, dir: &str) -> bool {
        if self.over || self.won || !self.can_move(dir) { return false; }
        self.history.push((self.board, self.score));
        let (nb, s) = Self::apply(&self.board, dir);
        self.board = nb; self.score += s;
        self.spawn();
        if !self.won { for r in 0..SIZE { for c in 0..SIZE { if self.board[r][c] == 2048 { self.won = true; }}}}
        if !Self::has_adjacent(&self.board) { self.over = true; }
        true
    }

    pub fn undo(&mut self) -> bool {
        if let Some((board, score)) = self.history.pop() { self.board = board; self.score = score; self.won = false; self.over = false; true }
        else { false }
    }

    pub fn max_tile(&self) -> u32 { self.board.iter().flat_map(|r| r.iter()).copied().max().unwrap_or(0) }
}
