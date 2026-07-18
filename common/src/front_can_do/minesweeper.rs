use rand::Rng;

const W: usize = 9;
const H: usize = 9;
const MINES: usize = 10;

#[derive(Clone, Copy, PartialEq)]
enum Cell { Hidden(u8), Revealed(u8), Flagged(u8) }

pub struct Minesweeper {
    pub cells: [[u8; W]; H],
    pub revealed: [[bool; W]; H],
    pub flagged: [[bool; W]; H],
    pub over: bool,
    pub won: bool,
    first_click: bool,
}

impl Minesweeper {
    pub fn new() -> Self {
        Minesweeper {
            cells: [[0; W]; H],
            revealed: [[false; W]; H],
            flagged: [[false; W]; H],
            over: false, won: false,
            first_click: true,
        }
    }

    fn place_mines(&mut self, sx: usize, sy: usize) {
        let mut rng = rand::thread_rng();
        let mut placed = 0;
        while placed < MINES {
            let x = rng.gen_range(0..W);
            let y = rng.gen_range(0..H);
            if self.cells[y][x] == 9 || (x == sx && y == sy) { continue; }
            self.cells[y][x] = 9;
            placed += 1;
        }
        for y in 0..H {
            for x in 0..W {
                if self.cells[y][x] == 9 { continue; }
                let mut n = 0u8;
                for dy in 0..3 { for dx in 0..3 {
                    let nx = x.wrapping_add(dx).wrapping_sub(1);
                    let ny = y.wrapping_add(dy).wrapping_sub(1);
                    if nx < W && ny < H && self.cells[ny][nx] == 9 { n += 1; }
                }}
                self.cells[y][x] = n;
            }
        }
    }

    fn reveal(&mut self, x: usize, y: usize) {
        if x >= W || y >= H || self.revealed[y][x] || self.flagged[y][x] { return; }
        self.revealed[y][x] = true;
        if self.cells[y][x] == 0 {
            for dy in 0..3 { for dx in 0..3 {
                self.reveal(x.wrapping_add(dx).wrapping_sub(1), y.wrapping_add(dy).wrapping_sub(1));
            }}
        }
    }

    pub fn click(&mut self, x: usize, y: usize) {
        if self.over || self.won || x >= W || y >= H { return; }
        if self.first_click {
            self.first_click = false;
            self.place_mines(x, y);
        }
        if self.flagged[y][x] { return; }
        if self.cells[y][x] == 9 { self.over = true; return; }
        self.reveal(x, y);
        self.check_win();
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.over || self.won || x >= W || y >= H || self.revealed[y][x] { return; }
        self.flagged[y][x] = !self.flagged[y][x];
    }

    fn check_win(&mut self) {
        let mut safe = 0;
        for y in 0..H { for x in 0..W {
            if self.cells[y][x] != 9 && !self.revealed[y][x] { safe += 1; }
        }}
        if safe == 0 { self.won = true; }
    }

    pub fn flag_count(&self) -> usize {
        self.flagged.iter().flat_map(|r| r.iter()).filter(|&&f| f).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_game() { let m = Minesweeper::new(); assert!(!m.revealed[0][0]); }
    #[test]
    fn test_click() { let mut m = Minesweeper::new(); m.click(4, 4); assert!(!m.over); }
    #[test]
    fn test_flag() { let mut m = Minesweeper::new(); m.toggle_flag(3, 3); assert!(m.flagged[3][3]); }
}
