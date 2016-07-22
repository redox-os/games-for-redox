
use std::time::Duration;

use pseudo_random::PseudoRandom;

#[derive(PartialEq,Eq,Clone,Copy)]
pub enum BlockType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
    GARBAGE,
    NONE,
}

const GRID_WIDTH: u8 = 10;
const GRID_HEIGHT: u8 = 20;
const START_BLOCK_SPEED_NANOSEC: u32 = 2000000000;
const END_BLOCK_SPEED_NANOSEC: u32 = 50000000;
const BLOCK_SPEED_STEP: u32 =
    (START_BLOCK_SPEED_NANOSEC - END_BLOCK_SPEED_NANOSEC) / 255;
const LINES_TO_CLEAR_TO_LVL_UP: u8 = 10;

pub fn p2xy(pos: u8) -> (u8, u8) {
    (pos % GRID_WIDTH, pos / GRID_WIDTH)
}

pub fn xy2p(x: u8, y: u8) -> u8 {
    x + y * GRID_WIDTH
}

pub fn pr2piece(pos: u8, rotation: u8, block_type: BlockType, grid_width: u8) -> [u8;4] {
    match rotation {
        0 =>
            match block_type {
                BlockType::I => [pos - grid_width, pos, pos + grid_width, pos - grid_width * 2],
                BlockType::J => [pos - grid_width, pos, pos + grid_width, pos - grid_width - 1],
                BlockType::L => [pos - grid_width, pos, pos + grid_width, pos - grid_width + 1],
                BlockType::O => [pos, pos + 1, pos - grid_width, pos - grid_width + 1],
                BlockType::S => [pos, pos + 1, pos - grid_width, pos - grid_width - 1],
                BlockType::T => [pos - 1, pos, pos + 1, pos - grid_width],
                BlockType::Z => [pos - 1, pos, pos - grid_width, pos - grid_width + 1],
                _ => [0, 0, 0, 0],
            },
        1 =>
            match block_type {
                BlockType::I => [pos + 1, pos, pos - 1, pos - 2],
                BlockType::J => [pos + 1, pos, pos - 1, pos + grid_width - 1],
                BlockType::L => [pos + 1, pos, pos - 1, pos - grid_width - 1],
                BlockType::O => [pos, pos - 1, pos - grid_width, pos - grid_width - 1],
                BlockType::S => [pos - grid_width, pos, pos - 1, pos + grid_width - 1],
                BlockType::T => [pos + grid_width, pos, pos - grid_width, pos - 1],
                BlockType::Z => [pos + grid_width, pos, pos - 1, pos - grid_width - 1],
                _ => [0, 0, 0, 0],
            },
        2 =>
            match block_type {
                BlockType::I => [pos - grid_width, pos, pos + grid_width, pos - grid_width * 2],
                BlockType::J => [pos - grid_width, pos, pos + grid_width, pos + grid_width + 1],
                BlockType::L => [pos - grid_width, pos, pos + grid_width, pos + grid_width - 1],
                BlockType::O => [pos, pos - 1, pos + grid_width, pos + grid_width - 1],
                BlockType::S => [pos - 1, pos, pos + grid_width, pos + grid_width + 1],
                BlockType::T => [pos - 1, pos, pos + 1, pos + grid_width],
                BlockType::Z => [pos + 1, pos, pos + grid_width, pos + grid_width - 1],
                _ => [0, 0, 0, 0],
            },
        3 =>
            match block_type {
                BlockType::I => [pos + 1, pos, pos - 1, pos - 2],
                BlockType::J => [pos - 1, pos, pos + 1, pos - grid_width + 1],
                BlockType::L => [pos - 1, pos, pos + 1, pos + grid_width + 1],
                BlockType::O => [pos, pos + 1, pos + grid_width, pos + grid_width + 1],
                BlockType::S => [pos + grid_width, pos, pos + 1, pos - grid_width + 1],
                BlockType::T => [pos + 1, pos + grid_width, pos, pos - grid_width],
                BlockType::Z => [pos - grid_width, pos, pos + 1, pos + grid_width + 1],
                _ => [0, 0, 0, 0],
            },
        _ => panic!("fn pr2piece: Invalid rotation value!"),
    }
}

/// (0, 0) is bottom left
pub struct Grid {
    pub grid: [BlockType; (GRID_WIDTH * (GRID_HEIGHT + 1)) as usize],
    pub grid_size: u8,
    falling_type: BlockType,
    falling_pos: u8,
    falling_rot: u8,
    next_falling_type: BlockType,
    next_falling_rot: u8,
    pub rng: PseudoRandom,
    level: u8,
    lines_cleared: u32,
    elapsed_time: Duration,
    cached_fall_rate: Duration,
    pub dead: bool,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            grid: [BlockType::NONE; (GRID_WIDTH * (GRID_HEIGHT + 1)) as usize],
            grid_size: GRID_WIDTH * GRID_HEIGHT,
            falling_type: BlockType::NONE,
            falling_pos: 0,
            falling_rot: 0,
            next_falling_type: BlockType::L,
            next_falling_rot: 0,
            rng: PseudoRandom::new(),
            level: 0,
            lines_cleared: 0,
            elapsed_time: Duration::new(0, 0),
            cached_fall_rate: Duration::new(0, START_BLOCK_SPEED_NANOSEC),
            dead: false,
        }
    }

    pub fn reset(&mut self) {
        for i in 0..(GRID_WIDTH * (GRID_HEIGHT + 1)) {
            self.grid[i as usize] = BlockType::NONE;
        }
        self.falling_type = BlockType::NONE;
        match self.rng.next() % 7 {
            0 => self.next_falling_type = BlockType::I,
            1 => self.next_falling_type = BlockType::J,
            2 => self.next_falling_type = BlockType::L,
            3 => self.next_falling_type = BlockType::O,
            4 => self.next_falling_type = BlockType::S,
            5 => self.next_falling_type = BlockType::T,
            6 => self.next_falling_type = BlockType::Z,
            _ => panic!("fn Grid::reset: rng returned out of range number!"),
        }
        self.next_falling_rot = (self.rng.next() % 4) as u8;
        self.level = 0;
        self.lines_cleared = 0;
        self.elapsed_time = Duration::new(0, 0);
        self.cached_fall_rate = Duration::new(0, START_BLOCK_SPEED_NANOSEC);
        self.dead = false;
    }

    pub fn update(&mut self, delta: Duration) {
        if self.dead {
            return;
        }
        self.elapsed_time = self.elapsed_time + delta;
        if self.elapsed_time >= self.cached_fall_rate {
            self.elapsed_time = self.elapsed_time - self.cached_fall_rate;
            match self.falling_type {
                BlockType::NONE => self.generate_falling(),
                BlockType::I | BlockType::J | BlockType::L | BlockType::O |
                BlockType::S | BlockType::T | BlockType::Z => self.simulate_falling(),
                _ => panic!("fn Grid::update: falling_type is invalid type!"),
            }
        }
    }

    pub fn reset_elapsed_time(&mut self) {
        self.elapsed_time = Duration::new(0, 0);
    }

    /// At level 0, fall at START_BLOCK_SPEED_NANOSEC
    /// At level 255, fall at END_BLOCK_SPEED_NANOSEC
    fn fall_rate(&self) -> Duration {
        Duration::new(0, START_BLOCK_SPEED_NANOSEC - BLOCK_SPEED_STEP * self.level as u32)
    }

    fn generate_falling(&mut self) {
        self.check_lines();
        self.falling_pos = xy2p(GRID_WIDTH / 2, GRID_HEIGHT - 1);
        self.falling_type = self.next_falling_type;
        self.falling_rot = self.next_falling_rot;
        match self.rng.next() % 7 {
            0 => self.next_falling_type = BlockType::I,
            1 => self.next_falling_type = BlockType::J,
            2 => self.next_falling_type = BlockType::L,
            3 => self.next_falling_type = BlockType::O,
            4 => self.next_falling_type = BlockType::S,
            5 => self.next_falling_type = BlockType::T,
            6 => self.next_falling_type = BlockType::Z,
            _ => panic!("fn Grid::generate_falling: rng returned out of range number!"),
        }
        self.next_falling_rot = (self.rng.next() % 4) as u8;
        let piece_pos = pr2piece(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH);
        for i in 0..4 {
            if self.grid[piece_pos[i] as usize] != BlockType::NONE {
                self.dead = true;
                return;
            }
            else {
                self.grid[piece_pos[i] as usize] = self.falling_type;
            }
        }
    }

    pub fn simulate_falling(&mut self) {
        if self.falling_type == BlockType::NONE || self.dead {
            return;
        }
        let piece_pos = pr2piece(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH);
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::NONE;
        }
        let mut can_fall = true;
        for i in 0..4 {
            if piece_pos[i] < GRID_WIDTH || self.grid[(piece_pos[i] - GRID_WIDTH) as usize] != BlockType::NONE {
                can_fall = false;
                break;
            }
        }
        if can_fall {
            for i in 0..4 {
                self.grid[(piece_pos[i] - GRID_WIDTH) as usize] = self.falling_type;
            }
            self.falling_pos -= GRID_WIDTH;
        }
        else {
            for i in 0..4 {
                self.grid[piece_pos[i] as usize] = self.falling_type;
            }
            self.falling_type = BlockType::NONE;
            self.elapsed_time = Duration::new(0, 0);
            self.generate_falling();
        }
    }

    pub fn drop(&mut self) {
        if self.falling_type == BlockType::NONE || self.dead {
            return;
        }
        let piece_pos = pr2piece(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH);
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::NONE;
        }
        let mut drop_amount: u8 = 0;
        'outer: loop {
            for i in 0..4 {
                if self.grid[(piece_pos[i] - drop_amount) as usize] != BlockType::NONE {
                    drop_amount -= GRID_WIDTH;
                    break 'outer;
                }
            }
            for i in 0..4 {
                if piece_pos[i] - drop_amount < GRID_WIDTH {
                    break 'outer;
                }
            }
            drop_amount += GRID_WIDTH;
        }
        for i in 0..4 {
            self.grid[(piece_pos[i] - drop_amount) as usize] = self.falling_type;
        }
        self.falling_type = BlockType::NONE;
        self.elapsed_time = Duration::new(0, 0);
        self.generate_falling();
    }

    pub fn move_left(&mut self) {
        if self.falling_type == BlockType::NONE || self.dead {
            return;
        }
        let piece_pos = pr2piece(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH);
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::NONE;
        }
        let mut can_move = true;
        for i in 0..4 {
            if piece_pos[i] % GRID_WIDTH == 0 || self.grid[(piece_pos[i] - 1) as usize] != BlockType::NONE {
                can_move = false;
                break;
            }
        }
        if can_move {
            for i in 0..4 {
                self.grid[(piece_pos[i] - 1) as usize] = self.falling_type;
            }
            self.falling_pos -= 1;
        }
        else {
            for i in 0..4 {
                self.grid[piece_pos[i] as usize] = self.falling_type;
            }
        }
    }

    pub fn move_right(&mut self) {
        if self.falling_type == BlockType::NONE || self.dead {
            return;
        }
        let piece_pos = pr2piece(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH);
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::NONE;
        }
        let mut can_move = true;
        for i in 0..4 {
            if piece_pos[i] % GRID_WIDTH == GRID_WIDTH - 1 || self.grid[(piece_pos[i] + 1) as usize] != BlockType::NONE {
                can_move = false;
                break;
            }
        }
        if can_move {
            for i in 0..4 {
                self.grid[(piece_pos[i] + 1) as usize] = self.falling_type;
            }
            self.falling_pos += 1;
        }
        else {
            for i in 0..4 {
                self.grid[piece_pos[i] as usize] = self.falling_type;
            }
        }
    }

    pub fn rotate_clockwise(&mut self) {
        if self.falling_type == BlockType::NONE || self.dead {
            return;
        }
        let piece_pos = pr2piece(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH);
        let mut leftmost = GRID_WIDTH - 1;
        let mut rightmost = 0;
        let mut lowest = GRID_HEIGHT * GRID_WIDTH;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::NONE;
            if piece_pos[i] % GRID_WIDTH < leftmost {
                leftmost = piece_pos[i] % GRID_WIDTH;
            }
            if piece_pos[i] % GRID_WIDTH > rightmost {
                rightmost = piece_pos[i] % GRID_WIDTH;
            }
            if piece_pos[i] / GRID_WIDTH < lowest {
                lowest = piece_pos[i] / GRID_WIDTH;
            }
        }
        let mut can_rotate = true;
        match self.falling_type {
            BlockType::I => {
                if ((leftmost == 0 || leftmost == 1) && (self.falling_rot == 0 || self.falling_rot == 2)) ||
                    (rightmost == GRID_WIDTH - 1 && (self.falling_rot == 0 || self.falling_rot == 2)) ||
                    ((lowest == 0 || lowest == 1) && (self.falling_rot == 1 || self.falling_rot == 3)) {
                    can_rotate = false;
                }
            },
            BlockType::J => {
                if (leftmost == 0 && self.falling_rot == 2) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 0) ||
                    (lowest == 0 && self.falling_rot == 1) {
                    can_rotate = false;
                }
            },
            BlockType::L => {
                if (leftmost == 0 && self.falling_rot == 0) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 2) ||
                    (lowest == 0 && self.falling_rot == 3) {
                    can_rotate = false;
                }
            },
            BlockType::O => {
                if (leftmost == 0 && self.falling_rot == 0) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 2) ||
                    (lowest == 0 && self.falling_rot == 3) {
                    can_rotate = false;
                }
            },
            BlockType::S => {
                if (leftmost == 0 && self.falling_rot == 3) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 1) ||
                    (lowest == 0 && self.falling_rot == 2) {
                    can_rotate = false;
                }
            },
            BlockType::T => {
                if (leftmost == 0 && self.falling_rot == 3) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 1) ||
                    (lowest == 0 && self.falling_rot == 2) {
                    can_rotate = false;
                }
            },
            BlockType::Z => {
                if (leftmost == 0 && self.falling_rot == 3) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 1) ||
                    (lowest == 0 && self.falling_rot == 2) {
                    can_rotate = false;
                }
            },
            _ => panic!("fn Grid::rotate_clockwise: falling_type is invalid type!"),
        }
        if can_rotate {
            let rotated_pos = pr2piece(self.falling_pos, (self.falling_rot + 1) % 4, self.falling_type, GRID_WIDTH);
            for i in 0..4 {
                if self.grid[rotated_pos[i] as usize] != BlockType::NONE {
                    can_rotate = false;
                    break;
                }
            }
            if can_rotate {
                for i in 0..4 {
                    self.grid[rotated_pos[i] as usize] = self.falling_type;
                }
                self.falling_rot = (self.falling_rot + 1) % 4;
            }
            else {
                for i in 0..4 {
                    self.grid[piece_pos[i] as usize] = self.falling_type;
                }
            }
        }
        else {
            for i in 0..4 {
                self.grid[piece_pos[i] as usize] = self.falling_type;
            }
        }
    }

    pub fn rotate_counter_clockwise(&mut self) {
        if self.falling_type == BlockType::NONE || self.dead {
            return;
        }
        let mut next_rot = self.falling_rot;
        if next_rot == 0 {
            next_rot = 3;
        }
        else {
            next_rot -= 1;
        }
        let piece_pos = pr2piece(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH);
        let mut leftmost = GRID_WIDTH - 1;
        let mut rightmost = 0;
        let mut lowest = GRID_HEIGHT * GRID_WIDTH;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::NONE;
            if piece_pos[i] % GRID_WIDTH < leftmost {
                leftmost = piece_pos[i] % GRID_WIDTH;
            }
            if piece_pos[i] % GRID_WIDTH > rightmost {
                rightmost = piece_pos[i] % GRID_WIDTH;
            }
            if piece_pos[i] / GRID_WIDTH < lowest {
                lowest = piece_pos[i] / GRID_WIDTH;
            }
        }
        let mut can_rotate = true;
        match self.falling_type {
            BlockType::I => {
                if ((leftmost == 0 || leftmost == 1) && (self.falling_rot == 0 || self.falling_rot == 2)) ||
                    (rightmost == GRID_WIDTH - 1 && (self.falling_rot == 0 || self.falling_rot == 2)) ||
                    ((lowest == 0 || lowest == 1) && (self.falling_rot == 1 || self.falling_rot == 3)) {
                    can_rotate = false;
                }
            },
            BlockType::J => {
                if (leftmost == 0 && self.falling_rot == 2) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 0) ||
                    (lowest == 0 && self.falling_rot == 1) {
                    can_rotate = false;
                }
            },
            BlockType::L => {
                if (leftmost == 0 && self.falling_rot == 0) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 2) ||
                    (lowest == 0 && self.falling_rot == 3) {
                    can_rotate = false;
                }
            },
            BlockType::O => {
                if (leftmost == 0 && self.falling_rot == 3) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 1) ||
                    (lowest == 0 && self.falling_rot == 2) {
                    can_rotate = false;
                }
            },
            BlockType::S => {
                if (leftmost == 0 && self.falling_rot == 3) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 1) ||
                    (lowest == 0 && self.falling_rot == 2) {
                    can_rotate = false;
                }
            },
            BlockType::T => {
                if (leftmost == 0 && self.falling_rot == 3) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 1) ||
                    (lowest == 0 && self.falling_rot == 2) {
                    can_rotate = false;
                }
            },
            BlockType::Z => {
                if (leftmost == 0 && self.falling_rot == 3) ||
                    (rightmost == GRID_WIDTH - 1 && self.falling_rot == 1) ||
                    (lowest == 0 && self.falling_rot == 2) {
                    can_rotate = false;
                }
            },
            _ => panic!("fn Grid::rotate_clockwise: falling_type is invalid type!"),
        }
        if can_rotate {
            let rotated_pos = pr2piece(self.falling_pos, next_rot, self.falling_type, GRID_WIDTH);
            for i in 0..4 {
                if self.grid[rotated_pos[i] as usize] != BlockType::NONE {
                    can_rotate = false;
                    break;
                }
            }
            if can_rotate {
                for i in 0..4 {
                    self.grid[rotated_pos[i] as usize] = self.falling_type;
                }
                if self.falling_rot == 0 {
                    self.falling_rot = 3;
                }
                else {
                    self.falling_rot -= 1;
                }
            }
            else {
                for i in 0..4 {
                    self.grid[piece_pos[i] as usize] = self.falling_type;
                }
            }
        }
        else {
            for i in 0..4 {
                self.grid[piece_pos[i] as usize] = self.falling_type;
            }
        }
    }

    fn check_lines(&mut self) {
        let mut to_clear: [bool; GRID_HEIGHT as usize] = [false; GRID_HEIGHT as usize];
        for y in 0..GRID_HEIGHT {
            let mut is_clearing = true;
            for x in 0..GRID_WIDTH {
                if self.grid[xy2p(x, y) as usize] == BlockType::NONE {
                    is_clearing = false;
                    break;
                }
            }
            to_clear[y as usize] = is_clearing;
        }

        let mut y = GRID_HEIGHT - 1;
        loop {
            if to_clear[y as usize] {
                for j in y..GRID_HEIGHT {
                    for i in 0..GRID_WIDTH {
                        self.grid[xy2p(i, j) as usize] = self.grid[xy2p(i, j + 1) as usize];
                    }
                }
                for i in 0..GRID_WIDTH {
                    self.grid[xy2p(i, GRID_HEIGHT) as usize] = BlockType::NONE;
                }
            }
            if y == 0 {
                break;
            }
            else
            {
                y -= 1;
            }
        }

        let mut clear_count = 0;
        for i in 0..GRID_HEIGHT {
            if to_clear[i as usize] {
                clear_count += 1;
            }
        }

        while clear_count > 0 {
            self.lines_cleared += 1;
            clear_count -= 1;
            if self.lines_cleared % LINES_TO_CLEAR_TO_LVL_UP as u32 == 0 {
                self.level += 1;
                self.cached_fall_rate = self.fall_rate();
            }
        }
    }

    pub fn get_level(&self) -> u8 {
        self.level
    }

    pub fn get_lines_cleared(&self) -> u32 {
        self.lines_cleared
    }

    pub fn get_next_type(&self) -> BlockType {
        self.next_falling_type
    }

    pub fn get_next_rot(&self) -> u8 {
        self.next_falling_rot
    }
}

