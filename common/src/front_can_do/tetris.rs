use rand::Rng;

const W: usize = 10;
const H: usize = 20;

type Piece = (&'static [u8], usize, usize);

const PIECES: &[Piece] = &[
    (&[1,1,1,1], 4, 1),          // I
    (&[1,1,1,1], 2, 2),          // O
    (&[0,1,0,1,1,1], 3, 2),      // T
    (&[1,0,0,1,1,1], 3, 2),      // L
    (&[0,0,1,1,1,1], 3, 2),      // J
    (&[0,1,1,1,1,0], 3, 2),      // S
    (&[1,1,0,0,1,1], 3, 2),      // Z
];

fn rotated(data: &[u8], w: usize, h: usize, rot: usize) -> Vec<u8> {
    let mut r = data.to_vec();
    let (mut cw, mut ch) = (w, h);
    for _ in 0..rot {
        let mut nr = vec![0u8; cw * ch];
        for y in 0..ch { for x in 0..cw {
            nr[x * h + (ch - 1 - y)] = r[y * cw + x];
        }}
        std::mem::swap(&mut cw, &mut ch);
        r = nr;
    }
    r
}

pub struct Tetris {
    pub board: [[u8; W]; H],
    pub score: u32,
    pub over: bool,
    piece_idx: usize,
    px: i32, py: i32,
    rot: usize,
}

impl Tetris {
    pub fn new() -> Self {
        let mut t = Tetris { board: [[0; W]; H], score: 0, over: false, piece_idx: 0, px: 0, py: 0, rot: 0 };
        t.spawn();
        t
    }

    fn piece(&self) -> Piece { PIECES[self.piece_idx] }
    fn cells(&self) -> Vec<u8> { let (d, w, h) = self.piece(); rotated(d, w, h, self.rot) }

    fn collides(&self, dx: i32, dy: i32, dr: usize) -> bool {
        let (d, w, h) = PIECES[self.piece_idx];
        let r = rotated(d, w, h, (self.rot + dr) % 4);
        let (cw, ch) = if (self.rot + dr) % 2 == 0 { (w, h) } else { (h, w) };
        for y in 0..ch { for x in 0..cw {
            if r[y * cw + x] == 0 { continue; }
            let nx = self.px + x as i32;
            let ny = self.py + y as i32;
            if nx < 0 || nx >= W as i32 || ny >= H as i32 { return true; }
            if ny >= 0 && self.board[ny as usize][nx as usize] != 0 { return true; }
        }}
        false
    }

    fn spawn(&mut self) {
        self.piece_idx = rand::thread_rng().gen_range(0..PIECES.len());
        self.px = (W / 2) as i32 - 1;
        self.py = 0;
        self.rot = 0;
        if self.collides(0, 0, 0) { self.over = true; }
    }

    fn lock(&mut self) {
        let r = self.cells();
        let (_, w, h) = self.piece();
        let (cw, ch) = if self.rot % 2 == 0 { (w, h) } else { (h, w) };
        for y in 0..ch { for x in 0..cw {
            if r[y * cw + x] == 0 { continue; }
            let ny = self.py + y as i32;
            let nx = self.px + x as i32;
            if ny >= 0 && ny < H as i32 && nx >= 0 && nx < W as i32 {
                self.board[ny as usize][nx as usize] = (self.piece_idx + 1) as u8;
            }
        }}
        self.clear_lines();
        self.spawn();
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0;
        for y in (0..H).rev() {
            if self.board[y].iter().all(|&c| c != 0) {
                for yy in (1..=y).rev() { self.board[yy] = self.board[yy - 1]; }
                self.board[0] = [0; W];
                cleared += 1;
            }
        }
        self.score += match cleared { 1 => 100, 2 => 300, 3 => 500, 4 => 800, _ => 0 };
    }

    pub fn tick(&mut self) {
        if self.over { return; }
        if !self.collides(0, 1, 0) { self.py += 1; } else { self.lock(); }
    }

    pub fn move_piece(&mut self, dir: &str) {
        if self.over { return; }
        match dir {
            "left" if !self.collides(-1, 0, 0) => self.px -= 1,
            "right" if !self.collides(1, 0, 0) => self.px += 1,
            "down" if !self.collides(0, 1, 0) => self.py += 1,
            "drop" => { while !self.collides(0, 1, 0) { self.py += 1; } self.lock(); }
            "rotate" => { if !self.collides(0, 0, 1) { self.rot = (self.rot + 1) % 4; }}
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() { let t = Tetris::new(); assert!(!t.over); }
    #[test]
    fn test_move() { let mut t = Tetris::new(); let px = t.px; t.move_piece("left"); assert!(t.px < px || t.over); }
}
