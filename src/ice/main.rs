extern crate termion;

use termion::{clear, cursor, style};
use termion::raw::{IntoRawMode, RawTerminal};
use std::env::args;
use std::io::{self, Write, Read};
use std::thread;
use std::time;

static MAN_PAGE: &'static str = /* @MANSTART{ice} */ r#"
NAME
    ice - slide over ice into the goal.

SYNOPSIS
    ice [-h | --help]

DESCRIPTION
    A puzzle game of sliding over ice into a goal.

    Slide an icon over ice by moving until a wall or the goal is reached.
    Multiple levels are stored in ASIC-art format text files.

    Controls:
        q - quit the game
        w - slide up
        a - slide left
        s - side down
        d - slide right

OPTIONS
    (none)
        Run the program.
    -h
    --help
        Print this manual page.

AUTHOR
    This program was written by Ticki for Redox OS. Bugs, issues, or feature requests
    should be reported in the Gitlab repository, 'redox-os/games'.

COPYRIGHT
    Copyright (c) 2016 Ticki

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software
    and associated documentation files (the "Software"), to deal in the Software without
    restriction, including without limitation the rights to use, copy, modify, merge, publish,
    distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all copies or
    substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
    BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
    DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#; /* @MANEND */

static ARGS_ERR: &'static str = r#"
unknown argument, try `ice --help`
"#;

const MAP_1: &'static [u8] = include_bytes!("map1.txt");
const MAP_2: &'static [u8] = include_bytes!("map2.txt");
const MAP_3: &'static [u8] = include_bytes!("map3.txt");
const MAP_4: &'static [u8] = include_bytes!("map4.txt");
const MAP_5: &'static [u8] = include_bytes!("map5.txt");
const DONE: &'static [u8] = include_bytes!("done.txt");

/// The game state.
struct Game<R, W: Write> {
    /// The x coordinate.
    x: u16,
    /// The y coordinate.
    y: u16,
    /// Standard output.
    stdout: W,
    /// Standard input.
    stdin: R,
    /// The width of the map.
    width: usize,
    /// The map.
    map: &'static [u8],
    /// Current level.
    level: u8,
}

/// A direction.
#[derive(Copy, Clone)]
enum Direction {
    /// Up.
    Up,
    /// Down.
    Down,
    /// Left.
    Left,
    /// Right.
    Right,
}

impl<R: Read, W: Write> Game<R, W> {
    /// Construct a new game state.
    fn new(stdin: R, stdout: W) -> Game<R, RawTerminal<W>> {
        Game {
            x: 1,
            y: 1,
            stdout: stdout.into_raw_mode().unwrap(),
            stdin: stdin,
            width: 1,
            map: MAP_1,
            level: 0,
        }
    }

    /// Start the game loop.
    ///
    /// This will listen to events and do the appropriate actions.
    fn start(&mut self) {
        self.init();

        loop {
            // Read a single byte from stdin.
            let mut b = [0];
            self.stdin.read(&mut b).unwrap();

            match b[0] {
                b'h' | b'a' => self.slide(Direction::Left),
                b'j' | b's' => self.slide(Direction::Down),
                b'k' | b'w' => self.slide(Direction::Up),
                b'l' | b'd' => self.slide(Direction::Right),
                b'q' => return,
                _ => {},
            }

            self.stdout.flush().unwrap();
        }
    }

    /// Initialize the level.
    fn init(&mut self) {
        write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();

        let mut width_counted = false;

        for &i in self.map {
            if i == b'\n' {
                width_counted = true;
                self.stdout.write(b"\n\r").unwrap();
                if !width_counted {
                    width_counted = true;
                }
            } else {
                self.stdout.write(&[i]).unwrap();
            }
            if !width_counted {
                self.width += 1;
            }
        }
        self.update();
    }

    /// Get the position of the next step.
    ///
    /// This will calculate the position of the step in a given direction.
    fn next(&mut self, dir: Direction) -> (u16, u16) {
        let mut new_x = self.x;
        let mut new_y = self.y;

        match dir {
            Direction::Right => new_x += 1,
            Direction::Left => new_x -= 1,
            Direction::Down => new_y += 1,
            Direction::Up => new_y -= 1,
        }

        (new_x, new_y)
    }

    /// Get the character of a given (x, y).
    fn get(&mut self, x: u16, y: u16) -> u8 {
         self.map[y as usize * self.width + x as usize]
    }

    /// Move the cursor to the player position.
    fn update(&mut self) {
        write!(self.stdout, "{}", cursor::Goto(self.x + 1, self.y + 1)).unwrap();
        self.stdout.flush().unwrap();
    }

    /// The level is done. Go to the next level.
    fn done(&mut self) {
        self.level += 1;
        self.width = 1;
        self.x = 1;
        self.y = 1;

        let level = self.level;
        self.map = self.get_map(level);
        self.init();
    }

    /// Get the map of the level.
    fn get_map(&mut self, level: u8) -> &'static [u8] {
        match level {
            0 => MAP_1,
            1 => MAP_2,
            2 => MAP_3,
            3 => MAP_4,
            4 => MAP_5,
            _ => DONE,
        }
    }

    /// Slide the character over the ices until a solid block is reached.
    fn slide(&mut self, dir: Direction) {
        loop {
            let (x, y) = self.next(dir);

            match self.get(x, y) {
                b'@' => {
                    self.done();
                    break;
                },
                b' ' => {
                    self.x = x;
                    self.y = y;
                    self.update();

                    thread::sleep(time::Duration::from_millis(10));
                },
                _ => break,
            }
        }
    }
}

impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        // When done, restore the defaults to avoid messing with the terminal.
        write!(self.stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1)).unwrap();
    }
}

fn main() {
    {
        let args = args().skip(1);
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        for i in args {
            match i.as_str() {
                "-h" | "--help" => {
                    // Write man page help.
                    stdout.write(MAN_PAGE.as_bytes()).unwrap();
                }
                _ => {
                    // Unknown argument(s).
                    stdout.write(ARGS_ERR.as_bytes()).unwrap();
                }
            }
            return;
        }
    }

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut game = Game::new(stdin.lock(), stdout.lock());

    game.start();
}
