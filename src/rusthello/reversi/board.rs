//! Implementation of a 2D board (and of its constituing elements) with coordinates and iterators.

use reversi;
use reversi::Result;

/// The number of cells per side of the board.
pub const BOARD_SIZE: usize = 8;

/// The total number of cells of the board. Derived from `BOARD_SIZE` for ease of use.
pub const NUM_CELLS: usize = BOARD_SIZE * BOARD_SIZE;

/// Enums all the cardinal directions.
/// #Examples
/// If I am in cell `(4, 5)` and move `NE`, I go to cell `(3, 6)`.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    NE,
    East,
    SE,
    South,
    SW,
    West,
    NW,
}

/// Lists all cardinal directions from `Direction`.
pub const DIRECTIONS: [Direction; 8] = [
    Direction::North,
    Direction::NE,
    Direction::East,
    Direction::SE,
    Direction::South,
    Direction::SW,
    Direction::West,
    Direction::NW
];

/// Coordinates of a cell, given by a row and a column.
/// Follows matrices conventions (see <https://en.wikipedia.org/wiki/Matrix_(mathematics)>) but for starting indexes at 0.
#[derive(Debug, Clone, Copy)]
pub struct Coord {
    row: usize,
    col: usize,
}

impl Coord {

    pub fn new(row: usize, col: usize) -> Coord {
        Coord {
            row: row,
            col: col,
        }
    }

    /// Returns coordinate's components.
    pub fn get_row_col(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    /// Returns coordinate's first component.
    pub fn get_row(&self) -> usize {
        self.row
    }

    /// Returns coordinate's second component.
    pub fn get_col(&self) -> usize {
        self.col
    }

    /// Checks both upper and lower bounds.
    pub fn check_bounds(&self) -> Result<()> {
        if self.row >= BOARD_SIZE || self.col >= BOARD_SIZE {
            Err(reversi::ReversiError::OutOfBoundCoord(*self))
        } else {
            Ok(())
        }
    }

    /// Produces new indexes moving along a direction and checking for out-of-bound errors (meaning sub-zero indexes).
    pub fn step(&mut self, dir: Direction) -> Result<()> {
        match dir {
            Direction::North    if self.row > 0                     => Ok({self.row -= 1;}),
            Direction::NE       if self.row > 0                     => Ok({self.row -= 1; self.col += 1;}),
            Direction::East                                         => Ok({self.col += 1;}),
            Direction::SE                                           => Ok({self.row += 1; self.col += 1;}),
            Direction::South                                        => Ok({self.row += 1;}),
            Direction::SW       if self.col > 0                     => Ok({self.row += 1; self.col -= 1;}),
            Direction::West     if self.col > 0                     => Ok({self.col -= 1;}),
            Direction::NW       if self.row > 0 && self.col > 0     => Ok({self.row -= 1; self.col -= 1;}),
            _ => Err(reversi::ReversiError::OutOfBoundStep(*self, dir)),
        }
    }
}

/// A disk is characterized by its two sides, one Dark and one Light.
#[derive(Debug, Clone, Copy)]
pub struct Disk(reversi::Side);

impl Disk {
    /// Creates a new disk with given side.
    pub fn new(side: reversi::Side) -> Disk {
        Disk(side)
    }

    /// Return's a disk's side.
    pub fn get_side(&self) -> reversi::Side {
        self.0
    }

    /// Turns the disk on the other side.
    pub fn flip(&mut self) {
        self.0 = self.0.opposite();
    }
}

/// Each cell in the board can either be empty or taken by one of the players.
pub type Cell = Option<Disk>;

#[derive(Debug, Clone)]
pub struct Board([[Cell; BOARD_SIZE]; BOARD_SIZE]);

/// `Board` is the type of boards, which are made by a `Frame`.
impl Board {
    /// Creates a new board, given its cells.
    pub fn new(cells: &[[Cell; BOARD_SIZE]; BOARD_SIZE]) -> Board {
        Board(*cells)
    }

    /// Returns a non-mutable reference to the array of cells.
    pub fn get_all_cells(&self) -> &[[Cell; BOARD_SIZE]; BOARD_SIZE] {
        &self.0
    }

    /// Returns a non-mutable cell.
    pub fn get_cell(&self, coord: Coord) -> Result<Cell> {
        try!(self.0.get(coord.get_row()).ok_or(reversi::ReversiError::OutOfBoundCoord(coord)))
            .get(coord.get_col()).ok_or(reversi::ReversiError::OutOfBoundCoord(coord)).map(|&cell| cell)
    }

    /// Returns a non-mutable disk.
    pub fn get_disk(&self, coord: Coord) -> Result<Disk> {
        match try!(self.get_cell(coord)) {
            None => Err(reversi::ReversiError::EmptyCell(coord)),
            Some(disk) => Ok(disk),
        }
    }

    /// Returns a mutable reference to a cell (which is why it's private).
    fn get_mut_cell(&mut self, coord: Coord) -> Result<&mut Cell> {
        try!(self.0.get_mut(coord.get_row()).ok_or(reversi::ReversiError::OutOfBoundCoord(coord)))
            .get_mut(coord.get_col()).ok_or(reversi::ReversiError::OutOfBoundCoord(coord))
    }

    /// Flips the disk on a non-empty cell.
    pub fn flip_disk(&mut self, coord: Coord) -> Result<()> {
        let cell = try!(self.get_mut_cell(coord));
        match cell {
            &mut Some(mut disk) => Ok({
                disk.flip();
                *cell = Some(disk);
            }),
            &mut None => Err(reversi::ReversiError::EmptyCell(coord)),
        }
    }

    /// Place a disk on an empty cell.
    pub fn place_disk(&mut self, side: reversi::Side, coord: Coord) -> Result<()> {
        let cell = try!(self.get_mut_cell(coord));
        match cell {
            &mut Some(_) => Err(reversi::ReversiError::CellAlreadyTaken(coord)),
            &mut None => Ok( *cell = Some(Disk::new(side)) ),
        }
    }
}
