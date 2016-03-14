#![feature(iter_arith)]

extern crate libterm;

use libterm::{IntoRawMode, TermWrite};

use std::io::{stdout, stdin, Read, Write};

/// A cell in the grid.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
struct Cell {
    /// Does it contain a mine?
    mine: bool,
    /// Is it revealed?
    ///
    /// That is, is it showed or chosen previously by the player?
    revealed: bool,
}

const FLAGGED: &'static str = "▓";
const MINE: &'static str = "█";
const EMPTY: &'static str = "▒";

/// The game state.
struct Game<R, W> {
    /// Width of the grid.
    width: usize,
    /// The grid.
    ///
    /// The cells are enumerated like you would read a book. Left to right, until you reach the
    /// line ending.
    grid: Box<[Cell]>,
    /// The current position.
    ///
    /// That is, what cell is the cursor on at this moment. Note that _c = y * w + x_.
    c: usize,
    /// The randomizer state.
    ///
    /// This will be modified when a random value is read or written.
    seed: usize,
    /// Standard output.
    stdout: W,
    /// Standard input.
    stdin: R,
}

/// Initialize the game.
fn init(w: usize, h: usize) {
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    stdout.write(b"type 10 random characters: ").unwrap();
    stdout.flush().unwrap();

    let mut buf = [0; 10];
    stdin.read_exact(&mut buf).unwrap();
    stdout.clear().unwrap();
    stdout.goto(0, 0).unwrap();
    stdout.flush().unwrap();

    let mut game = Game {
        c: 0,
        seed: 0,
        width: w,
        grid: vec![Cell {
            mine: false,
            revealed: false,
        }; w * h].into_boxed_slice(),
        stdin: stdin,
        stdout: stdout,
    };

    for &i in buf.iter() {
        game.write_rand(i);
    }

    game.reset();

    game.start();
    game.stdout.restore().unwrap();
}

impl<R: Read, W: Write> Game<R, W> {
    /// Start the game loop.
    ///
    /// This will listen to events and do the appropriate actions.
    fn start(&mut self) {
        loop {
            let mut b = [0];
            self.stdin.read(&mut b).unwrap();

            match b[0] {
                b'h' => self.c = self.left(self.c),
                b'j' => self.c = self.down(self.c),
                b'k' => self.c = self.up(self.c),
                b'l' => self.c = self.right(self.c),
                b' ' => {
                    let c = self.c;

                    if self.grid[c].mine {
                        self.game_over();
                        return;
                    }

                    self.reveal(c);
                },
                b'f' => self.set_flag(),
                b'F' => self.remove_flag(),
                b'q' => return,
                _ => {},
            }

            let c = self.c;
            self.update(c);
            self.stdout.flush().unwrap();
        }
    }

    /// Read a number from the randomizer.
    fn read_rand(&mut self) -> usize {
        self.seed ^= self.seed.rotate_right(4).wrapping_add(0x25A45B35C4FD3DF2);
        self.seed ^= self.seed >> 7;
        self.seed
    }

    /// Write a number into the randomizer.
    ///
    /// This is used for collecting entropy to the randomizer.
    fn write_rand(&mut self, b: u8) {
        self.seed ^= b as usize;
        self.read_rand();
    }

    fn set_flag(&mut self) {
        self.stdout.write(FLAGGED.as_bytes());
    }
    fn remove_flag(&mut self) {
        self.stdout.write(EMPTY.as_bytes());
    }

    /// Reset the game.
    ///
    /// This will display the starting grid, and fill the old grid with random mines.
    fn reset(&mut self) {
        self.stdout.goto(0, 0).unwrap();

        self.stdout.write("┌".as_bytes()).unwrap();
        for _ in 0..self.width {
            self.stdout.write("─".as_bytes()).unwrap();
        }
        self.stdout.write("┐\n\r".as_bytes()).unwrap();

        for _ in 0..self.grid.len() / self.width {
            self.stdout.write("│".as_bytes()).unwrap();
            for _ in 0..self.width {
                self.stdout.write_all(EMPTY.as_bytes()).unwrap();
            }
            self.stdout.write("│".as_bytes()).unwrap();
            self.stdout.write(b"\n\r").unwrap();
        }

        self.stdout.write("└".as_bytes()).unwrap();
        for _ in 0..self.width {
            self.stdout.write("─".as_bytes()).unwrap();
        }
        self.stdout.write("┘".as_bytes()).unwrap();

        for i in 0..self.grid.len() {
            self.grid[i] = Cell {
                mine: self.read_rand() % 3 == 0,
                revealed: false,
            };
        }
    }

    /// Get the value of a cell.
    ///
    /// The value represent the sum of adjacent cells containing mines. A cell of value, 0, is
    /// called "free".
    fn val(&self, c: usize) -> u8 {
        self.adjacent(c).iter().map(|&c| self.grid[c].mine as u8).sum()
    }


    /// Update the cursor to reflect the current position.
    fn update(&mut self, c: usize) {
        self.stdout.goto((c % self.width) as u16 + 1, (c / self.width) as u16 + 1).unwrap();
    }

    /// Reveal the cell, _c_.
    ///
    /// This will recursively reveal free cells, until non-free cell is reached, terminating the
    /// current recursion descendant.
    fn reveal(&mut self, c: usize) {
        let v = self.val(c);

        self.grid[c].revealed = true;

        self.update(c);

        if v == 0 {
            self.stdout.write(b" ").unwrap();

            for &adj in self.adjacent(c).iter() {
                if !self.grid[adj].revealed && !self.grid[adj].mine {
                    self.reveal(adj);
                }
            }
        } else {
            self.stdout.write(&[b'0' + v]).unwrap();
        }
    }

    /// Reveal all the fields, printing where the mines were.
    fn reveal_all(&mut self) {
        self.stdout.goto(0, 0).unwrap();

        for y in 0..self.grid.len() / self.width {
            for x in 0..self.width {
                self.stdout.goto(x as u16 + 1, y as u16 + 1).unwrap();
                if self.grid[self.width * y + x].mine {
                    self.stdout.write(MINE.as_bytes()).unwrap();
                }
            }
        }
    }

    /// Game over!
    fn game_over(&mut self) {
        self.reveal_all();
        self.stdout.goto(0, 0).unwrap();
        self.stdout.hide_cursor();
        self.stdout.write("╔═════════════════╗\n\r\
                           ║───┬Game over────║\n\r\
                           ║ r ┆ replay      ║\n\r\
                           ║ q ┆ quit        ║\n\r\
                           ╚═══╧═════════════╝\
                          ".as_bytes()).unwrap();
        self.stdout.flush().unwrap();

        loop {
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();

            match buf[0] {
                b'r' => {
                    self.stdout.show_cursor();
                    self.restart();
                    break;
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
    fn adjacent(&self, c: usize) -> [usize; 8] {
        [
            // Left-up
            self.up(self.left(c)),
            // Up
            self.up(c),
            // Right-up
            self.up(self.right(c)),
            // Left
            self.left(c),
            // Right
            self.right(c),
            // Left-down
            self.down(self.left(c)),
            // Down
            self.down(c),
            // Down-right
            self.down(self.right(c)),
        ]
    }

    fn up(&self, c: usize) -> usize {
        if c < self.width {
            self.grid.len() - self.width + c % self.width
        } else {
            c - self.width
        }
    }
    fn down(&self, c: usize) -> usize {
        if self.grid.len() - c <= self.width {
            c % self.width
        } else {
            c + self.width
        }
    }
    fn left(&self, c: usize) -> usize {
        // Wrap around.
        if c % self.width == 0 {
            c + self.width - 1
        } else {
            c - 1
        }
    }
    fn right(&self, c: usize) -> usize {
        if (c + 1) % self.width == 0 {
            c + 1 - self.width
        } else {
            c + 1
        }
    }
}

fn main() {
    init(70, 40);
}
