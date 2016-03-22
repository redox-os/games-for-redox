//! This crate is a simple implementation of minesweeper. It is carefully documented to encourage
//! newbies to add new games to the repository.

extern crate termion;
extern crate extra;

use termion::{IntoRawMode, TermWrite};

use std::env;
use std::io::{self, Read, Write};
use std::process;

use extra::rand::Randomizer;

/// A cell in the grid.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
struct Cell {
    /// Does it contain a mine?
    mine: bool,
    /// Is it revealed?
    ///
    /// That is, is it showed or chosen previously by the player?
    revealed: bool,
    /// Is this cell observed?
    ///
    /// That is, is the state of this cell determined, or is it pending for randomization.
    observed: bool,
}

/// The string printed for flagged cells.
#[cfg(target_os = "redox")]
const FLAGGED: &'static str = "F";
#[cfg(not(target_os = "redox"))]
const FLAGGED: &'static str = "▓";
/// The string printed for mines in the game over revealing.
#[cfg(target_os = "redox")]
const MINE: &'static str = "!";
#[cfg(not(target_os = "redox"))]
const MINE: &'static str = "█";
/// The string printed for concealed cells.
#[cfg(target_os = "redox")]
const CONCEALED: &'static str = "#";
#[cfg(not(target_os = "redox"))]
const CONCEALED: &'static str = "▒";

/// The game over screen.
#[cfg(target_os = "redox")]
const GAME_OVER: &'static str = "+-----------------+\n\r\
                                 |----Game over----|\n\r\
                                 | r | replay      |\n\r\
                                 | q | quit        |\n\r\
                                 +-----------------+";
#[cfg(not(target_os = "redox"))]
const GAME_OVER: &'static str = "╔═════════════════╗\n\r\
                                 ║───┬Game over────║\n\r\
                                 ║ r ┆ replay      ║\n\r\
                                 ║ q ┆ quit        ║\n\r\
                                 ╚═══╧═════════════╝";

/// The upper and lower boundary char.
#[cfg(target_os = "redox")]
const HORZ_BOUNDARY: &'static str = "-";
#[cfg(not(target_os = "redox"))]
const HORZ_BOUNDARY: &'static str = "─";
/// The left and right boundary char.
#[cfg(target_os = "redox")]
const VERT_BOUNDARY: &'static str = "|";
#[cfg(not(target_os = "redox"))]
const VERT_BOUNDARY: &'static str = "│";

/// The top-left corner
#[cfg(target_os = "redox")]
const TOP_LEFT_CORNER: &'static str = "+";
#[cfg(not(target_os = "redox"))]
const TOP_LEFT_CORNER: &'static str = "┌";
/// The top-right corner
#[cfg(target_os = "redox")]
const TOP_RIGHT_CORNER: &'static str = "+";
#[cfg(not(target_os = "redox"))]
const TOP_RIGHT_CORNER: &'static str = "┐";
/// The bottom-left corner
#[cfg(target_os = "redox")]
const BOTTOM_LEFT_CORNER: &'static str = "+";
#[cfg(not(target_os = "redox"))]
const BOTTOM_LEFT_CORNER: &'static str = "└";
/// The bottom-right corner
#[cfg(target_os = "redox")]
const BOTTOM_RIGHT_CORNER: &'static str = "+";
#[cfg(not(target_os = "redox"))]
const BOTTOM_RIGHT_CORNER: &'static str = "┘";

/// The help page.
const HELP: &'static str = r#"
minesweeper ~ a simple minesweeper implementation.

rules:
    Select a cell to reveal, printing the number of adjacent cells holding a mine.
    If no adjacent cells hold a mine, the cell is called free. Free cell will recursively
    reveal their neighboring cells. If a mine is revealed, you loose. The grid wraps.

flags:
    -r | --height N ~ set the height of the grid.
    -c | --width N  ~ set the width of the grid.
    -h | --help     ~ this help page.
    -b              ~ beginner mode.
    -i              ~ immediate mode.
    -a              ~ advanced mode.
    -g              ~ god mode.

controls:
    ---selection--------------------
    space ~ reveal the current cell.
    ---movement---------------------
    h     ~ move left.
    j     ~ move down.
    k     ~ move up.
    l     ~ move right.
    ---flags------------------------
    f     ~ set flag.
    F     ~ remove flag.
    ---control----------------------
    q     ~ quit game.
    r     ~ restart game.

author:
    ticki.
"#;

/// The game state.
struct Game<R, W: Write> {
    /// Width of the grid.
    width: u16,
    /// The grid.
    ///
    /// The cells are enumerated like you would read a book. Left to right, until you reach the
    /// line ending.
    grid: Box<[Cell]>,
    /// The difficulty of the game.
    ///
    /// The lower, the easier.
    difficulty: u8,
    /// The x coordinate.
    x: u16,
    /// The y coordinate.
    y: u16,
    /// The randomizer.
    rand: Randomizer,
    /// Points.
    ///
    /// That is, revealed fields.
    points: u16,
    /// Standard output.
    stdout: W,
    /// Standard input.
    stdin: R,
}

/// Initialize the game.
fn init<W: Write, R: Read>(mut stdout: W, stdin: R, difficulty: u8, w: u16, h: u16) {
    stdout.clear().unwrap();

    // Set the initial game state.
    let mut game = Game {
        x: 0,
        y: 0,
        rand: Randomizer::new(0),
        width: w,
        grid: vec![Cell {
            mine: false,
            revealed: false,
            observed: false,
        }; w as usize * h as usize].into_boxed_slice(),
        points: 0,
        stdin: stdin,
        stdout: stdout,
        difficulty: difficulty,
    };

    // Reset that game.
    game.reset();

    // Start the event loop.
    game.start();
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        // When done, restore the defaults to avoid messing with the terminal.
        self.stdout.restore().unwrap();
    }
}

impl<R: Read, W: Write> Game<R, W> {
    /// Get the grid position of a given coordinate.
    fn pos(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
    }

    /// Read cell, randomizing it if it is unobserved.
    fn read_cell(&mut self, c: usize) {
        if !self.grid[c].observed {
            self.grid[c].mine = self.rand.read_u8() % self.difficulty == 0;
            self.grid[c].observed = true;
        }
    }

    /// Get the cell at (x, y).
    fn get(&mut self, x: u16, y: u16) -> Cell {
        let pos = self.pos(x, y);

        self.read_cell(pos);
        self.grid[pos]
    }

    /// Get a mutable reference to the cell at (x, y).
    fn get_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        let pos = self.pos(x, y);

        self.read_cell(pos);
        &mut self.grid[pos]
    }

    /// Start the game loop.
    ///
    /// This will listen to events and do the appropriate actions.
    fn start(&mut self) {
        loop {
            // Read a single byte from stdin.
            let mut b = [0];
            self.stdin.read(&mut b).unwrap();

            // Collect it as entropy.
            self.rand.write_u8(b[0]);

            match b[0] {
                b'h' => self.x = self.left(self.x),
                b'j' => self.y = self.down(self.y),
                b'k' => self.y = self.up(self.y),
                b'l' => self.x = self.right(self.x),
                b' ' => {
                    // Check if it was a mine.
                    let (x, y) = (self.x, self.y);
                    if self.get(x, y).mine {
                        self.game_over();
                        return;
                    }

                    if !self.get(x, y).revealed {
                        self.points += 1;
                    }

                    // Reveal the cell.
                    self.reveal(x, y);

                    self.print_points();
                },
                b'f' => self.set_flag(),
                b'F' => self.remove_flag(),
                b'r' => {
                    self.restart();
                    return;
                }
                b'q' => return,
                _ => {},
            }

            // Make sure the cursor is placed on the current position.
            self.stdout.goto(self.x + 1, self.y + 1).unwrap();
            self.stdout.flush().unwrap();
        }
    }

    /// Set a flag on the current cell.
    fn set_flag(&mut self) {
        self.stdout.write(FLAGGED.as_bytes()).unwrap();
    }
    /// Remove a flag on the current cell.
    fn remove_flag(&mut self) {
        self.stdout.write(CONCEALED.as_bytes()).unwrap();
    }

    /// Reset the game.
    ///
    /// This will display the starting grid, and fill the old grid with random mines.
    fn reset(&mut self) {
        // Reset the cursor.
        self.stdout.goto(0, 0).unwrap();

        // Write the upper part of the frame.
        self.stdout.write(TOP_LEFT_CORNER.as_bytes()).unwrap();
        for _ in 0..self.width {
            self.stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
        }
        self.stdout.write(TOP_RIGHT_CORNER.as_bytes()).unwrap();
        self.stdout.write(b"\n\r").unwrap();

        // Conceal all the cells.
        for _ in 0..self.height() {
            // The left part of the frame
            self.stdout.write(VERT_BOUNDARY.as_bytes()).unwrap();

            for _ in 0..self.width {
                self.stdout.write_all(CONCEALED.as_bytes()).unwrap();
            }

            // The right part of the frame.
            self.stdout.write(VERT_BOUNDARY.as_bytes()).unwrap();
            self.stdout.write(b"\n\r").unwrap();
        }

        // Write the lower part of the frame.
        self.stdout.write(BOTTOM_LEFT_CORNER.as_bytes()).unwrap();
        for _ in 0..self.width {
            self.stdout.write(HORZ_BOUNDARY.as_bytes()).unwrap();
        }
        self.stdout.write(BOTTOM_RIGHT_CORNER.as_bytes()).unwrap();

        self.stdout.goto(self.x + 1, self.y + 1).unwrap();
        self.stdout.flush().unwrap();

        // Reset the grid.
        for i in 0..self.grid.len() {
            // Fill it with random, concealed fields.
            self.grid[i] = Cell {
                mine: false,
                revealed: false,
                observed: false,
            };

            self.points = 0;
        }
    }

    /// Get the value of a cell.
    ///
    /// The value represent the sum of adjacent cells containing mines. A cell of value, 0, is
    /// called "free".
    fn val(&mut self, x: u16, y: u16) -> u8 {
        // To avoid nightly version, we manually sum the adjacent mines.
        let mut res = 0;
        for &(x, y) in self.adjacent(x, y).iter() {
            res += self.get(x, y).mine as u8;
        }
        res
    }

    /// Reveal the cell, _c_.
    ///
    /// This will recursively reveal free cells, until non-free cell is reached, terminating the
    /// current recursion descendant.
    fn reveal(&mut self, x: u16, y: u16) {
        let v = self.val(x, y);

        self.get_mut(x, y).revealed = true;

        self.stdout.goto(x + 1, y + 1).unwrap();

        if v == 0 {
            // If the cell is free, simply put a space on the position.
            self.stdout.write(b" ").unwrap();

            // Recursively reveal adjacent cells until a non-free cel is reached.
            for &(x, y) in self.adjacent(x, y).iter() {
                if !self.get(x, y).revealed && !self.get(x, y).mine {
                    self.reveal(x, y);
                }
            }
        } else {
            // Aww. The cell was not free. Print the value instead.
            self.stdout.write(&[b'0' + v]).unwrap();
        }
    }

    /// Print the point count.
    fn print_points(&mut self) {
        let height = self.height();
        self.stdout.goto(2, height + 1).unwrap();
        self.stdout.write(self.points.to_string().as_bytes()).unwrap();
    }

    /// Reveal all the fields, printing where the mines were.
    fn reveal_all(&mut self) {
        self.stdout.goto(0, 0).unwrap();

        for y in 0..self.height() {
            for x in 0..self.width {
                self.stdout.goto(x + 1, y + 1).unwrap();
                if self.get(x, y).mine {
                    self.stdout.write(MINE.as_bytes()).unwrap();
                }
            }
        }
    }

    /// Game over!
    fn game_over(&mut self) {
        // Reveal all cells, showing the player where the mines were.
        self.reveal_all();

        self.stdout.goto(0, 0).unwrap();

        // Hide the cursor.
        self.stdout.hide_cursor().unwrap();

        self.stdout.write(GAME_OVER.as_bytes()).unwrap();
        self.stdout.flush().unwrap();

        loop {
            // Repeatedly read a single byte.
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();

            match buf[0] {
                b'r' => {
                    // Replay!
                    self.stdout.show_cursor().unwrap();
                    self.restart();
                    return;
                },
                b'q' => return,
                _ => {},
            }
        }
    }

    /// Restart (replay) the game.
    fn restart(&mut self) {
        self.reset();
        self.start();
    }

    /// Calculate the adjacent cells.
    fn adjacent(&self, x: u16, y: u16) -> [(u16, u16); 8] {
        let left = self.left(x);
        let right = self.right(x);
        let up = self.up(y);
        let down = self.down(y);

        [
            // Left-up
            (left, up),
            // Up
            (x, up),
            // Right-up
            (right, up),
            // Left
            (left, y),
            // Right
            (right, y),
            // Left-down
            (left, down),
            // Down
            (x, down),
            // Right-down
            (right, down)
        ]
    }

    /// Calculate the height (number of rows) of the grid.
    fn height(&self) -> u16 {
        (self.grid.len() / self.width as usize) as u16
    }

    /// Calculate the y coordinate of the cell "above" a given y coordinate.
    ///
    /// This wraps when _y = 0_.
    fn up(&self, y: u16) -> u16 {
        if y == 0 {
            // Upper bound reached. Wrap around.
            self.height() - 1
        } else {
            y - 1
        }
    }
    /// Calculate the y coordinate of the cell "below" a given y coordinate.
    ///
    /// This wraps when _y = h - 1_.
    fn down(&self, y: u16) -> u16 {
        if y + 1 == self.height() {
            // Lower bound reached. Wrap around.
            0
        } else {
            y + 1
        }
    }
    /// Calculate the x coordinate of the cell "left to" a given x coordinate.
    ///
    /// This wraps when _x = 0_.
    fn left(&self, x: u16) -> u16 {
        if x == 0 {
            // Lower bound reached. Wrap around.
            self.width - 1
        } else {
            x - 1
        }
    }
    /// Calculate the x coordinate of the cell "left to" a given x coordinate.
    ///
    /// This wraps when _x = w - 1_.
    fn right(&self, x: u16) -> u16 {
        if x + 1 == self.width {
            // Upper bound reached. Wrap around.
            0
        } else {
            x + 1
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let mut width = None;
    let mut height = None;
    let mut diff = 6;

    // Get and lock the stdios.
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stderr = io::stderr();
    let mut stderr = stderr.lock();

    loop {
        // Read the arguments.

        let arg = if let Some(x) = args.next() {
            x
        } else {
            break;
        };

        match arg.as_str() {
            "-r" | "--height" => if height.is_none() {
                height = Some(args.next().unwrap_or_else(|| {
                    stderr.write(b"no height given.\n").unwrap();
                    stderr.flush().unwrap();
                    process::exit(1);
                }).parse().unwrap_or_else(|_| {
                    stderr.write(b"invalid integer given.\n").unwrap();
                    stderr.flush().unwrap();
                    process::exit(1);
                }));
            } else {
                stderr.write(b"you may only input one height.\n").unwrap();
                stderr.flush().unwrap();
                process::exit(1);
            },
            "-c" | "--width" => if width.is_none() {
                width = Some(args.next().unwrap_or_else(|| {
                    stderr.write(b"no width given.\n").unwrap();
                    stderr.flush().unwrap();
                    process::exit(1);
                }).parse().unwrap_or_else(|_| {
                    stderr.write(b"invalid integer given.\n").unwrap();
                    stderr.flush().unwrap();
                    process::exit(1);
                }));
            } else {
                stderr.write(b"you may only input one width.\n").unwrap();
                stderr.flush().unwrap();
                process::exit(1);
            },
            "-h" | "--help" => {
                // Print the help page.
                stdout.write(HELP.as_bytes()).unwrap();
                stdout.flush().unwrap();
                process::exit(0);
            },
            "-g" => diff = 2,
            "-a" => diff = 4,
            "-i" => diff = 6,
            "-b" => diff = 10,
            _ => {
                stderr.write(b"Unknown argument.\n").unwrap();
                stderr.flush().unwrap();
                process::exit(1);
            }
        }
    }

    // We go to raw mode to make the control over the terminal more fine-grained.
    let stdout = stdout.into_raw_mode().unwrap();

    // Initialize the game!
    init(stdout, stdin, diff, width.unwrap_or(70), height.unwrap_or(40));
}
