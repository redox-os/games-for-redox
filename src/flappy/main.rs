/// TODO:
/// * Randomly generated walls
/// * User-defined width and height
/// * Tune speed, walls for better gameplay
extern crate termion;
use termion::{async_stdin, clear, cursor, style};
use termion::raw::{IntoRawMode, RawTerminal};

use std::collections::VecDeque;

use std::io::{self, Write, Read};
use std::thread;
use std::time::Duration;


const BIRD_CHAR: u8 = b'@';
const WALL_CHAR: u8 = b'#';
/// The rate of change of the bird's velocity
const GRAVITY: f32 = 0.75;
/// The width and height of the field of play
const WIDTH: usize = 80;
const HEIGHT: usize = 30;
/// The distance between the walls
const WALL_SEP: usize = 15;

const WELCOME_SCREEN: &'static str = "+=====================+\n\r\
                                      | space to jump       |\n\r\
                                      | q     to quit       |\n\r\
                                      +=====================+";


/// A flappy bird game
struct Game<R, W: Write> {
    /// The walls currently on the screen
    walls: VecDeque<Wall>,
    closest_wall: usize,
    /// The current position and vertical velocity of the bird
    height: f32,
    velocity: f32,
    /// The distance traveled
    distance: usize,
    /// Input/Output
    stdout: W,
    stdin: R,
}


struct Wall {
    pub gap_width: usize,
    pub gap_start: usize,
}

impl Wall {
    fn new(wid: usize, start: usize) -> Self {
        Wall {
            gap_width: wid,
            gap_start: start,
        }
    }
}


impl<R: Read, W: Write> Game<R, W> {
    /// Construct the default game
    fn new(stdin: R, stdout: W) -> Game<R, RawTerminal<W>> {
        Game {
            walls: vec![Wall::new(10, 5),
                        Wall::new(8, 4),
                        Wall::new(12, 14),
                        Wall::new(8, 10),
                        Wall::new(8, 7),
                        Wall::new(7, 8)]
                       .into_iter()
                       .collect(),
            closest_wall: 30,
            height: 15.0,
            velocity: 0.0,
            distance: 0,
            stdout: stdout.into_raw_mode().unwrap(),
            stdin: stdin,
        }
    }

    /// Play the game until it is over
    fn start(&mut self) {
        self.init();

        write!(self.stdout, "{}{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Hide, WELCOME_SCREEN).unwrap();
        self.stdout.flush().unwrap();

        loop {
            let mut b = [0];
            if self.stdin.read(&mut b).is_ok() {
                if b[0] != 0 {
                    break;
                }
            }
        }

        loop {
            let mut b = [0];
            if self.stdin.read(&mut b).is_ok() {
                match b[0] {
                    b'q' => break,
                    b' ' => self.jump(),
                    _ => {}
                };
            }

            thread::sleep(Duration::from_millis(150));
            self.step();
            self.draw();

            if self.game_over() {
                break;
            }
        }
    }


    fn step(&mut self) {
        self.height -= self.velocity;
        self.velocity -= GRAVITY;
        if self.closest_wall == 0 {
            if let Some(wall) = self.walls.pop_front() {
                self.walls.push_back(wall);
            }
            self.closest_wall = WALL_SEP;
        } else {
            self.closest_wall -= 1;
        }
        self.distance += 1;
    }


    fn draw(&mut self) {
        write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
        for _ in 0..WIDTH {
            self.stdout.write(&[WALL_CHAR]).unwrap();
        }
        self.stdout.write(b"\n\r").unwrap();

        for row in 0..HEIGHT {
            let first_wall = if self.height as usize == row {
                self.stdout.write(&[BIRD_CHAR]).unwrap();
                if self.closest_wall == 0 {
                    0
                } else {
                    self.closest_wall - 1
                }
            } else {
                self.closest_wall
            };

            let mut col = 0;

            for _ in 0..first_wall {
                self.stdout.write(b" ").unwrap();
                col += 1;
            }

            if let Some(wall) = self.walls.get(0) {
                if (row < wall.gap_start) || (row > wall.gap_start + wall.gap_width) {
                    self.stdout.write(&[WALL_CHAR])
                } else {
                    self.stdout.write(b" ")
                }
                .unwrap();
                col += 1;
            }

            for wall in self.walls.iter().skip(1) {
                col += WALL_SEP + 1;
                if col >= WIDTH {
                    break;
                }
                self.stdout.write(&[b' '; WALL_SEP]).unwrap();
                if (row < wall.gap_start) || (row > wall.gap_start + wall.gap_width) {
                    self.stdout.write(&[WALL_CHAR])
                } else {
                    self.stdout.write(b" ")
                }
                .unwrap();
            }

            self.stdout.write(b"\n\r").unwrap();
        }

        for _ in 0..WIDTH {
            self.stdout.write(&[WALL_CHAR]).unwrap();
        }
        self.stdout.write(b"\n\r").unwrap();
    }


    fn game_over(&self) -> bool {
        if self.closest_wall == 0 {
            if let Some(wall) = self.walls.get(0) {
                let height = self.height as usize;
                // If the bird is in contact with the wall
                height < wall.gap_start || height >= (wall.gap_start + wall.gap_width)
            } else {
                false
            }
        } else {
            // If the bird is out of bounds
            self.height < 0.0 || self.height >= HEIGHT as f32
        }
    }

    /// Perform a jump.
    fn jump(&mut self) {
        if self.velocity < 0.0 {
            self.velocity = 2.5;
        }
    }

    /// Initializes the screen
    fn init(&mut self) {
        write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    }
}


impl<R, W: Write> Drop for Game<R, W> {
    fn drop(&mut self) {
        // When done, restore the defaults to avoid messing with the terminal.
        // (same as in ice and minesweeper)
        write!(self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Show).unwrap();
        self.stdout.write(b"Distance traveled: ").unwrap();
        self.stdout.write(self.distance.to_string().as_bytes()).unwrap();
        self.stdout.write(b"\n\r").unwrap();
    }
}

fn main() {
    let stdout = io::stdout();

    let mut game = Game::new(async_stdin(), stdout.lock());

    game.start();
}
