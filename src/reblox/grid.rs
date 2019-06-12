use std::time::Duration;
use extra::rand::Randomizer;

pub const GRID_WIDTH: u8 = 10;
pub const GRID_HEIGHT: u8 = 20;
const START_BLOCK_SPEED_NANOSEC: u32 = 2000000000;
const END_BLOCK_SPEED_NANOSEC: u32 = 50000000;
const BLOCK_SPEED_STEP: u32 =
    (START_BLOCK_SPEED_NANOSEC - END_BLOCK_SPEED_NANOSEC) / 255;
const LINES_TO_CLEAR_TO_LVL_UP: u8 = 10;
const BLOCK_COLLECTION_SIZE: u8 = 3;

#[derive(PartialEq,Eq,Clone,Copy)]
pub enum BlockType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
    Garbage,
    None,
}

impl BlockType {
    pub fn num_to_block(n: u8) -> BlockType {
        match n {
            0 => BlockType::I,
            1 => BlockType::J,
            2 => BlockType::L,
            3 => BlockType::O,
            4 => BlockType::S,
            5 => BlockType::T,
            6 => BlockType::Z,
            7 => BlockType::Garbage,
            _ => BlockType::None,
        }
    }
}

struct NextBlockCollection {
    collection: Vec<(BlockType, u8)>,
}

impl NextBlockCollection {
    fn new() -> Self {
        let mut collection = Self {
            collection: Vec::new(),
        };
        collection.generate_collection();
        collection
    }

    fn reset(&mut self) {
        self.collection.clear();
        self.generate_collection();
    }

    fn generate_collection(&mut self) {
        for i in 0..=6 {
            self.collection.push((BlockType::num_to_block(i), BLOCK_COLLECTION_SIZE));
        }
    }

    fn next(&mut self, rng: &mut Randomizer) -> BlockType {
        let index = rng.read_u8() as usize % self.collection.len();
        let (next_type, ref mut count) = self.collection[index];
        *count -= 1;
        if *count == 0 {
            self.collection.swap_remove(index);
        }
        if self.collection.is_empty() {
            self.generate_collection();
        }
        next_type
    }
}

pub struct Grid1D {
    pub x: u8,
    pub width: u8,
}

pub struct Grid2D {
    pub x: u8,
    pub y: u8,
    pub width: u8,
}

impl From<Grid2D> for Grid1D {
    fn from(g2d: Grid2D) -> Grid1D {
        Grid1D {
            x: g2d.x + g2d.y * g2d.width,
            width: g2d.width,
        }
    }
}

impl From<Grid1D> for Grid2D {
    fn from(g1d: Grid1D) -> Grid2D {
        Grid2D {
            x: g1d.x % g1d.width,
            y: g1d.x / g1d.width,
            width: g1d.width,
        }
    }
}

pub struct BlockPos {
    pub positions: [u8; 4],
}

impl BlockPos {
    pub fn new(pos: u8, rotation:u8, block_type: BlockType, grid_width: u8) -> BlockPos {
        let br_type = block_type as u8 + rotation * 7;
        match br_type {
            // type I rot 0
            0 => BlockPos{positions: [pos - grid_width, pos, pos + grid_width, pos - grid_width * 2]},
            // type J rot 0
            1 => BlockPos{positions: [pos - grid_width, pos, pos + grid_width, pos - grid_width - 1]},
            // type L rot 0
            2 => BlockPos{positions: [pos - grid_width, pos, pos + grid_width, pos - grid_width + 1]},
            // type O rot 0
            3 => BlockPos{positions: [pos, pos + 1, pos - grid_width, pos - grid_width + 1]},
            // type S rot 0
            4 => BlockPos{positions: [pos, pos + 1, pos - grid_width, pos - grid_width - 1]},
            // type T rot 0
            5 => BlockPos{positions: [pos - 1, pos, pos + 1, pos - grid_width]},
            // type Z rot 0
            6 => BlockPos{positions: [pos - 1, pos, pos - grid_width, pos - grid_width + 1]},
            // type I rot 1
            7 => BlockPos{positions: [pos + 1, pos, pos - 1, pos - 2]},
            // type J rot 1
            8 => BlockPos{positions: [pos + 1, pos, pos - 1, pos + grid_width - 1]},
            // type L rot 1
            9 => BlockPos{positions: [pos + 1, pos, pos - 1, pos - grid_width - 1]},
            // type O rot 1
            10 => BlockPos{positions: [pos, pos - 1, pos - grid_width, pos - grid_width - 1]},
            // type S rot 1
            11 => BlockPos{positions: [pos - grid_width, pos, pos - 1, pos + grid_width - 1]},
            // type T rot 1
            12 => BlockPos{positions: [pos + grid_width, pos, pos - grid_width, pos - 1]},
            // type Z rot 1
            13 => BlockPos{positions: [pos + grid_width, pos, pos - 1, pos - grid_width - 1]},
            // type I rot 2
            14 => BlockPos{positions: [pos - grid_width, pos, pos + grid_width, pos - grid_width * 2]},
            // type J rot 2
            15 => BlockPos{positions: [pos - grid_width, pos, pos + grid_width, pos + grid_width + 1]},
            // type L rot 2
            16 => BlockPos{positions: [pos - grid_width, pos, pos + grid_width, pos + grid_width - 1]},
            // type O rot 2
            17 => BlockPos{positions: [pos, pos - 1, pos + grid_width, pos + grid_width - 1]},
            // type S rot 2
            18 => BlockPos{positions: [pos - 1, pos, pos + grid_width, pos + grid_width + 1]},
            // type T rot 2
            19 => BlockPos{positions: [pos - 1, pos, pos + 1, pos + grid_width]},
            // type Z rot 2
            20 => BlockPos{positions: [pos + 1, pos, pos + grid_width, pos + grid_width - 1]},
            // type I rot 3
            21 => BlockPos{positions: [pos + 1, pos, pos - 1, pos - 2]},
            // type J rot 3
            22 => BlockPos{positions: [pos - 1, pos, pos + 1, pos - grid_width + 1]},
            // type L rot 3
            23 => BlockPos{positions: [pos - 1, pos, pos + 1, pos + grid_width + 1]},
            // type O rot 3
            24 => BlockPos{positions: [pos, pos + 1, pos + grid_width, pos + grid_width + 1]},
            // type S rot 3
            25 => BlockPos{positions: [pos + grid_width, pos, pos + 1, pos - grid_width + 1]},
            // type T rot 3
            26 => BlockPos{positions: [pos + 1, pos + grid_width, pos, pos - grid_width]},
            // type Z rot 3
            27 => BlockPos{positions: [pos - grid_width, pos, pos + 1, pos + grid_width + 1]},
            _ => BlockPos{positions: [0, 0, 0, 0]},
        }
    }
}

/// Grid implements the game logic (backend).
pub struct Grid {
    /// grid is updated such that almost everything that needs to be drawn is
    /// in this grid, including falling blocks. What isn't in this grid is
    /// the next falling block.
    /// Block data is stored in a 1D array. To convert to 2D, use Grid1D::to_2D.
    /// (0, 0) is bottom left.
    pub grid: [BlockType; (GRID_WIDTH * (GRID_HEIGHT + 1)) as usize],

    /// Is the BlockType of the currently falling block.
    falling_type: BlockType,

    /// Is the position of the currently falling block.
    /// Note that the falling block is stored in the grid. The position of each
    /// part of the block can be found using BlockPos.
    falling_pos: u8,

    /// Is the rotation of the currently falling block (0-3).
    /// Also used in BlockPos::new to get the position of the block in the grid.
    falling_rot: u8,

    /// The type of the next block to fall.
    next_falling_type: BlockType,

    /// The rotation of the next block to fall.
    next_falling_rot: u8,

    /// The random number generator (rng) for the game.
    pub rng: Randomizer,

    /// The current level. Higher level correlates to harder difficulty.
    /// (Faster fall speed.)
    level: u8,

    /// When lines_cleared reaches the LINES_TO_CLEAR_TO_LVL_UP interval,
    /// the level increments by 1 and the falling block speed increases.
    lines_cleared: u32,

    /// Duration that keeps track of elapsed time. Is incremented in update with
    /// the "delta" parameter. When elapsed_time is greater than or equal to
    /// cached_fall_rate, elapsed_time is decremented by cached_fall_rate and
    /// the currently falling block will descend by one, or if it has landed,
    /// the next falling block becomes the currently falling block and the
    /// next falling block info is randomly decided via rng.
    elapsed_time: Duration,

    /// Determines how long it takes for the falling block to descend by 1.
    /// On level up, this value decreases by BLOCK_SPEED_STEP.
    cached_fall_rate: Duration,

    /// If true, the game has ended.
    /// If the next falling block's spawn area is obstructed by another block
    /// when spawning the falling block, the game ends.
    pub dead: bool,

    next_block_collection: NextBlockCollection,

    held_block_type: BlockType,
    held_block_rot: u8,
    held_changed: bool,
}

impl Grid {
    pub fn new() -> Grid {
        let mut grid = Grid {
            grid: [BlockType::None; (GRID_WIDTH * (GRID_HEIGHT + 1)) as usize],
            falling_type: BlockType::None,
            falling_pos: 0,
            falling_rot: 0,
            next_falling_type: BlockType::L,
            next_falling_rot: 0,
            rng: Randomizer::new(0),
            level: 0,
            lines_cleared: 0,
            elapsed_time: Duration::new(0, 0),
            cached_fall_rate: Duration::new(0, START_BLOCK_SPEED_NANOSEC),
            dead: false,
            next_block_collection: NextBlockCollection::new(),
            held_block_type: BlockType::None,
            held_block_rot: 0,
            held_changed: false,
        };
        grid.next_falling_type = grid.next_block_collection.next(&mut grid.rng);
        grid
    }

    pub fn reset(&mut self) {
        for i in 0..(GRID_WIDTH * (GRID_HEIGHT + 1)) {
            self.grid[i as usize] = BlockType::None;
        }
        self.next_block_collection.reset();
        self.falling_type = BlockType::None;
        self.next_falling_type = self.next_block_collection.next(&mut self.rng);
        self.next_falling_rot = self.rng.read_u8() % 4;
        self.level = 0;
        self.lines_cleared = 0;
        self.elapsed_time = Duration::new(0, 0);
        self.cached_fall_rate = Duration::new(0, START_BLOCK_SPEED_NANOSEC);
        self.dead = false;
        self.held_block_type = BlockType::None;
        self.held_changed = false;
    }

    pub fn update(&mut self, delta: Duration) {
        if self.dead {
            return;
        }
        self.elapsed_time = self.elapsed_time + delta;
        if self.elapsed_time >= self.cached_fall_rate {
            self.elapsed_time = self.elapsed_time - self.cached_fall_rate;
            match self.falling_type {
                BlockType::None => self.generate_falling(),
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
        self.falling_pos = Grid1D::from(Grid2D { x: GRID_WIDTH / 2, y: GRID_HEIGHT - 1, width: GRID_WIDTH }).x;
        self.falling_type = self.next_falling_type;
        self.falling_rot = self.next_falling_rot;
        self.next_falling_type = self.next_block_collection.next(&mut self.rng);
        self.next_falling_rot = self.rng.read_u8() % 4;
        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        for i in 0..4 {
            if self.grid[piece_pos[i] as usize] != BlockType::None {
                self.dead = true;
                return;
            }
            else {
                self.grid[piece_pos[i] as usize] = self.falling_type;
            }
        }
    }

    pub fn simulate_falling(&mut self) {
        if self.falling_type == BlockType::None || self.dead {
            return;
        }
        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::None;
        }
        let mut can_fall = true;
        for i in 0..4 {
            if piece_pos[i] < GRID_WIDTH || self.grid[(piece_pos[i] - GRID_WIDTH) as usize] != BlockType::None {
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
            self.falling_type = BlockType::None;
            self.elapsed_time = Duration::new(0, 0);
            self.held_changed = false;
            self.generate_falling();
        }
    }

    pub fn fall(&mut self) {
        if self.falling_type == BlockType::None || self.dead {
            return;
        }
        self.held_changed = false;
        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::None;
        }
        let mut drop_amount: u8 = 0;
        'outer: loop {
            for i in 0..4 {
                if self.grid[(piece_pos[i] - drop_amount) as usize] != BlockType::None {
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
        self.falling_type = BlockType::None;
        self.elapsed_time = Duration::new(0, 0);
        self.generate_falling();
    }

    pub fn move_left(&mut self) {
        if self.falling_type == BlockType::None || self.dead {
            return;
        }
        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::None;
        }
        let mut can_move = true;
        for i in 0..4 {
            if piece_pos[i] % GRID_WIDTH == 0 || self.grid[(piece_pos[i] - 1) as usize] != BlockType::None {
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
        if self.falling_type == BlockType::None || self.dead {
            return;
        }
        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::None;
        }
        let mut can_move = true;
        for i in 0..4 {
            if piece_pos[i] % GRID_WIDTH == GRID_WIDTH - 1 || self.grid[(piece_pos[i] + 1) as usize] != BlockType::None {
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
        if self.falling_type == BlockType::None || self.dead {
            return;
        }
        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        let mut leftmost = GRID_WIDTH - 1;
        let mut rightmost = 0;
        let mut lowest = GRID_HEIGHT * GRID_WIDTH;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::None;
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
            let rotated_pos = BlockPos::new(self.falling_pos, (self.falling_rot + 1) % 4, self.falling_type, GRID_WIDTH).positions;
            for i in 0..4 {
                if self.grid[rotated_pos[i] as usize] != BlockType::None {
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
        if self.falling_type == BlockType::None || self.dead {
            return;
        }
        let mut next_rot = self.falling_rot;
        if next_rot == 0 {
            next_rot = 3;
        }
        else {
            next_rot -= 1;
        }
        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        let mut leftmost = GRID_WIDTH - 1;
        let mut rightmost = 0;
        let mut lowest = GRID_HEIGHT * GRID_WIDTH;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::None;
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
            let rotated_pos = BlockPos::new(self.falling_pos, next_rot, self.falling_type, GRID_WIDTH).positions;
            for i in 0..4 {
                if self.grid[rotated_pos[i] as usize] != BlockType::None {
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
                if self.grid[Grid1D::from(Grid2D{x: x, y: y, width: GRID_WIDTH}).x as usize] == BlockType::None {
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
                        self.grid[Grid1D::from(Grid2D{x: i, y: j, width: GRID_WIDTH}).x as usize] = self.grid[Grid1D::from(Grid2D{x: i, y: j + 1, width: GRID_WIDTH}).x as usize];
                    }
                }
                for i in 0..GRID_WIDTH {
                    self.grid[Grid1D::from(Grid2D{x: i, y: GRID_HEIGHT, width: GRID_WIDTH}).x as usize] = BlockType::None;
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

    pub fn activate_hold(&mut self) {
        if self.held_changed {
            return;
        }
        self.held_changed = true;

        let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
        for i in 0..4 {
            self.grid[piece_pos[i] as usize] = BlockType::None;
        }

        let held_type = self.held_block_type;
        let held_rot = self.held_block_rot;
        self.held_block_type = self.falling_type;
        self.held_block_rot = self.falling_rot;

        if held_type != BlockType::None {
            self.falling_type = held_type;
            self.falling_rot = held_rot;
            self.falling_pos = Grid1D::from(Grid2D {
                x: GRID_WIDTH / 2,
                y: GRID_HEIGHT - 1,
                width: GRID_WIDTH,
            }).x;
            let piece_pos = BlockPos::new(self.falling_pos, self.falling_rot, self.falling_type, GRID_WIDTH).positions;
            for i in 0..4 {
                if self.grid[piece_pos[i] as usize] != BlockType::None {
                    self.dead = true;
                    return;
                }
                else {
                    self.grid[piece_pos[i] as usize] = self.falling_type;
                }
            }
        } else {
            self.falling_type = BlockType::None;
            self.generate_falling();
        }
        self.elapsed_time = Duration::new(0, 0);
    }

    pub fn get_held_type(&self) -> BlockType {
        self.held_block_type
    }

    pub fn get_held_rot(&self) -> u8 {
        self.held_block_rot
    }
}
