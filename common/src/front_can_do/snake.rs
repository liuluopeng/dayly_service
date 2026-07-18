use rand::Rng;

const W: usize = 20;
const H: usize = 20;

#[derive(Clone, Copy, PartialEq)]
enum Dir { Up, Down, Left, Right }

pub struct Snake {
    pub cells: [[u8; W]; H],
    pub score: u32,
    pub over: bool,
    body: Vec<(usize, usize)>,
    dir: Dir,
    next_dir: Dir,
    food: (usize, usize),
}

impl Snake {
    pub fn new() -> Self {
        let mut cells = [[0u8; W]; H];
        let body = vec![(W / 2, H / 2), (W / 2 - 1, H / 2)];
        for &(x, y) in &body { cells[y][x] = 1; }
        let mut s = Snake { cells, score: 0, over: false, body, dir: Dir::Right, next_dir: Dir::Right, food: (0, 0) };
        s.place_food();
        s
    }

    fn place_food(&mut self) {
        let mut rng = rand::thread_rng();
        let empty: Vec<_> = (0..H).flat_map(|y| (0..W).map(move |x| (x, y)))
            .filter(|&(x, y)| self.cells[y][x] == 0).collect();
        if let Some(&pos) = empty.get(rng.gen_range(0..empty.len())) {
            self.food = pos;
            self.cells[pos.1][pos.0] = 2;
        }
    }

    pub fn set_dir(&mut self, dir: &str) {
        self.next_dir = match dir {
            "up" if self.dir != Dir::Down => Dir::Up,
            "down" if self.dir != Dir::Up => Dir::Down,
            "left" if self.dir != Dir::Right => Dir::Left,
            "right" if self.dir != Dir::Left => Dir::Right,
            _ => return,
        };
    }

    pub fn tick(&mut self) {
        if self.over { return; }
        self.dir = self.next_dir;
        let head = self.body[0];
        let new = match self.dir {
            Dir::Up => (head.0, head.1.wrapping_sub(1)),
            Dir::Down => (head.0, head.1 + 1),
            Dir::Left => (head.0.wrapping_sub(1), head.1),
            Dir::Right => (head.0 + 1, head.1),
        };
        if new.0 >= W || new.1 >= H { self.over = true; return; }
        if self.cells[new.1][new.0] == 1 && new != self.body.last().copied().unwrap_or((0, 0)) {
            self.over = true; return;
        }
        self.body.insert(0, new);
        let ate = new == self.food;
        if ate {
            self.score += 1;
            self.place_food();
        } else {
            let tail = self.body.pop().unwrap();
            self.cells[tail.1][tail.0] = 0;
        }
        for &(x, y) in &self.body { self.cells[y][x] = 1; }
        if self.food.1 < H { self.cells[self.food.1][self.food.0] = 2; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_init() {
        let s = Snake::new();
        let count: usize = s.cells.iter().flat_map(|r| r.iter()).filter(|&&c| c > 0).count();
        assert_eq!(count, 3); // body(2) + food(1)
    }

    #[test]
    fn test_snake_move() {
        let mut s = Snake::new();
        let old = s.body[0];
        s.tick();
        assert!(s.body[0] != old || s.over);
    }
}
