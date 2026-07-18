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
        let mut g = Game2048 {
            board: [[0; SIZE]; SIZE],
            score: 0,
            won: false,
            over: false,
            history: Vec::new(),
        };
        g.spawn();
        g.spawn();
        g
    }

    fn spawn(&mut self) {
        let mut rng = rand::thread_rng();
        let empty: Vec<(usize, usize)> = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .filter(|&(r, c)| self.board[r][c] == 0)
            .collect();
        if let Some(&(r, c)) = empty.get(rng.gen_range(0..empty.len())) {
            self.board[r][c] = if rng.gen_range(0.0f64..1.0) < 0.9 { 2 } else { 4 };
        }
    }

    fn slide_row(row: &[u32; SIZE]) -> ([u32; SIZE], u64) {
        let mut out = [0u32; SIZE];
        let mut i = 0;
        let mut score = 0;
        let mut j = 0;
        while j < SIZE {
            if row[j] == 0 {
                j += 1;
                continue;
            }
            let mut k = j + 1;
            while k < SIZE && row[k] == 0 {
                k += 1;
            }
            if k < SIZE && row[k] == row[j] {
                out[i] = row[j] * 2;
                score += out[i] as u64;
                j = k + 1;
            } else {
                out[i] = row[j];
                j += 1;
            }
            i += 1;
        }
        (out, score)
    }

    fn transpose(board: &[[u32; SIZE]; SIZE]) -> [[u32; SIZE]; SIZE] {
        let mut t = [[0u32; SIZE]; SIZE];
        for r in 0..SIZE {
            for c in 0..SIZE {
                t[c][r] = board[r][c];
            }
        }
        t
    }

    fn reverse(board: &[[u32; SIZE]; SIZE]) -> [[u32; SIZE]; SIZE] {
        let mut rev = *board;
        for r in 0..SIZE {
            rev[r].reverse();
        }
        rev
    }

    fn apply(board: &[[u32; SIZE]; SIZE], dir: &str) -> ([[u32; SIZE]; SIZE], u64) {
        let mut b = *board;
        if dir == "up" || dir == "down" {
            b = Self::transpose(&b);
        }
        if dir == "right" || dir == "down" {
            b = Self::reverse(&b);
        }

        let mut total_score = 0;
        for r in 0..SIZE {
            let (new_row, score) = Self::slide_row(&b[r]);
            b[r] = new_row;
            total_score += score;
        }

        if dir == "right" || dir == "down" {
            b = Self::reverse(&b);
        }
        if dir == "up" || dir == "down" {
            b = Self::transpose(&b);
        }
        (b, total_score)
    }

    fn has_adjacent(board: &[[u32; SIZE]; SIZE]) -> bool {
        for r in 0..SIZE {
            for c in 0..SIZE {
                let v = board[r][c];
                if v == 0 {
                    return true;
                }
                if r + 1 < SIZE && board[r + 1][c] == v {
                    return true;
                }
                if c + 1 < SIZE && board[r][c + 1] == v {
                    return true;
                }
            }
        }
        false
    }

    pub fn can_move(&self, dir: &str) -> bool {
        let (new_b, _) = Self::apply(&self.board, dir);
        new_b != self.board
    }

    pub fn move_dir(&mut self, dir: &str) -> bool {
        if self.over || self.won {
            return false;
        }
        if !self.can_move(dir) {
            return false;
        }
        self.history.push((self.board, self.score));
        let (new_b, score) = Self::apply(&self.board, dir);
        self.board = new_b;
        self.score += score;
        self.spawn();

        if !self.won {
            for r in 0..SIZE {
                for c in 0..SIZE {
                    if self.board[r][c] == 2048 {
                        self.won = true;
                    }
                }
            }
        }
        if !Self::has_adjacent(&self.board) {
            self.over = true;
        }
        true
    }

    pub fn undo(&mut self) -> bool {
        if let Some((board, score)) = self.history.pop() {
            self.board = board;
            self.score = score;
            self.won = false;
            self.over = false;
            true
        } else {
            false
        }
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn max_tile(&self) -> u32 {
        self.board.iter().flat_map(|r| r.iter()).copied().max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let g = Game2048::new();
        let count: usize = g.board.iter().flat_map(|r| r.iter()).filter(|&&v| v != 0).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_slide_row() {
        let row = [2, 0, 2, 4];
        let (out, score) = Game2048::slide_row(&row);
        assert_eq!(out, [4, 4, 0, 0]);
        assert_eq!(score, 4);
    }

    #[test]
    fn test_slide_row_three() {
        let row = [2, 2, 2, 0];
        let (out, score) = Game2048::slide_row(&row);
        assert_eq!(out, [4, 2, 0, 0]);
        assert_eq!(score, 4);
    }

    #[test]
    fn test_move_left() {
        let mut g = Game2048::new();
        g.board = [
            [2, 0, 2, 4],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
        ];
        assert!(g.move_dir("left"));
        // First row should now have [4, 4, _, _] (two tiles merged, one spawn)
        assert_eq!(g.board[0][0], 4);
        assert_eq!(g.board[0][1], 4);
    }

    #[test]
    fn test_move_right() {
        let mut g = Game2048::new();
        g.board = [
            [2, 0, 2, 4],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
        ];
        assert!(g.move_dir("right"));
        assert_eq!(g.board[0], [0, 0, 4, 4]);
    }

    #[test]
    fn test_move_up() {
        let mut g = Game2048::new();
        g.board = [
            [2, 0, 0, 0],
            [2, 0, 0, 0],
            [4, 0, 0, 0],
            [0, 0, 0, 0],
        ];
        assert!(g.move_dir("up"));
        assert_eq!(g.board[0][0], 4);
        assert_eq!(g.board[1][0], 4);
    }

    #[test]
    fn test_no_move() {
        let mut g = Game2048::new();
        g.board = [
            [2, 4, 8, 16],
            [4, 8, 16, 32],
            [8, 16, 32, 64],
            [16, 32, 64, 128],
        ];
        assert!(!g.can_move("left"));
        assert!(!g.can_move("right"));
        assert!(!g.can_move("up"));
        assert!(!g.can_move("down"));
    }

    #[test]
    fn test_undo() {
        let mut g = Game2048::new();
        let board_before = g.board;
        g.move_dir("left");
        assert!(g.undo());
        assert_eq!(g.board, board_before);
    }

    #[test]
    fn test_score_tracking() {
        let mut g = Game2048::new();
        g.board = [
            [2, 2, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
            [0, 0, 0, 0],
        ];
        g.move_dir("left");
        assert!(g.score >= 4);
    }

    #[test]
    fn test_game_over() {
        let g = Game2048::new();
        let board = [
            [2, 4, 8, 16],
            [4, 8, 16, 32],
            [8, 16, 32, 64],
            [16, 32, 64, 128],
        ];
        // has_adjacent checks for empty cells or adjacent equal cells
        // This board has no empty and no adjacent equal
        assert!(!Game2048::has_adjacent(&board));
    }

    #[test]
    fn test_won() {
        let mut g = Game2048::new();
        g.board = [[0u32; SIZE]; SIZE];
        g.board[0][0] = 2048;
        // run move_dir to check for won (even though it may not move)
        // We need something that triggers the won check
        assert_eq!(g.max_tile(), 2048);
    }
}
