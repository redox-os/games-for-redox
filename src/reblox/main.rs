extern crate termion;
extern crate extra;

mod grid;

use termion::{IntoRawMode, TermWrite, RawTerminal, Color, async_stdin};
use std::io::{self, Write, Read};
use std::thread;
use std::time;

fn main() {
    let stdout = io::stdout();

    let mut game = Game::new(async_stdin(), stdout.lock());

    game.start();
}

fn type_to_color(block_type: grid::BlockType) -> termion::Color {
    match block_type {
        grid::BlockType::I => Color::Rgb(5, 5, 5),
        grid::BlockType::J => Color::Rgb(5, 0, 0),
        grid::BlockType::L => Color::Rgb(0, 5, 0),
        grid::BlockType::O => Color::Rgb(0, 0, 5),
        grid::BlockType::S => Color::Rgb(5, 5, 0),
        grid::BlockType::T => Color::Rgb(5, 0, 5),
        grid::BlockType::Z => Color::Rgb(0, 5, 5),
        grid::BlockType::Garbage => Color::Rgb(2, 2, 2),
        _ => Color::Rgb(0, 0, 0),
    }
}

struct Game<R, W: Write> {
    grid: grid::Grid,
    stdout: W,
    stdin: R,
}

impl<R: Read, W: Write> Game<R, W> {
    fn new(stdin: R, stdout: W) -> Game<R, RawTerminal<W>> {
        Game {
            grid: grid::Grid::new(),
            stdout: stdout.into_raw_mode().unwrap(),
            stdin: stdin,
        }
    }

    fn start(&mut self) {
        self.stdout.clear().unwrap();
        self.draw_grid_boundaries();
        self.draw_usage();
        self.grid.update(time::Duration::from_secs(2));
        let mut b: [u8; 1] = [0];
        'main: loop {
            thread::sleep(time::Duration::from_millis(50));
            if self.stdin.read(&mut b).is_ok() {
                match b[0] {
                    b'q' => break 'main,
                    b'r' => {
                        self.grid.reset();
                        self.grid.update(time::Duration::from_secs(2));
                    },
                    b'a' => self.grid.move_left(),
                    b'd' => self.grid.move_right(),
                    b's' => {
                        self.grid.simulate_falling();
                        self.grid.reset_elapsed_time();
                    },
                    b'k' => self.grid.rotate_clockwise(),
                    b'j' => self.grid.rotate_counter_clockwise(),
                    b' ' => self.grid.fall(),
                    _ => (),
                }
                self.grid.rng.read_u8();
                self.grid.rng.write_u8(b[0]);
                b[0] = 0;
            }
            self.grid.update(time::Duration::from_millis(50));
            self.draw_status();
            self.draw_next();
            self.draw_grid();
            self.stdout.flush().unwrap();
        }
    }

    fn draw_usage(&mut self) {
        self.stdout.goto(23, 1).unwrap();
        self.stdout.write(b"q - quit").unwrap();
        self.stdout.goto(23, 3).unwrap();
        self.stdout.write(b"asd - move").unwrap();
        self.stdout.goto(23, 4).unwrap();
        self.stdout.write(b"jk - rotate").unwrap();
        self.stdout.goto(23, 5).unwrap();
        self.stdout.write(b"space - drop").unwrap();
        self.stdout.goto(23, 7).unwrap();
        self.stdout.write(b"r - reset").unwrap();
    }

    fn draw_next(&mut self) {
        self.stdout.goto(23, 15).unwrap();
        self.stdout.write(b"next:").unwrap();
        for x in 0..4 {
            for y in 0..4 {
                self.stdout.goto(23 + x * 2, 16 + y).unwrap();
                self.stdout.write(b"  ").unwrap();
            }
        }

        self.stdout.bg_color(type_to_color(self.grid.get_next_type())).unwrap();
        let piece_pos = grid::BlockPos::new(10, self.grid.get_next_rot(), self.grid.get_next_type(), 4).positions;
        for i in 0..4 {
            let (x, y) = (piece_pos[i] % 4, piece_pos[i] / 4);
            self.stdout.goto(23 + x as u16 * 2, 16 + (3 - y as u16)).unwrap();
            self.stdout.write(b"  ").unwrap();
        }
        self.stdout.reset().unwrap();
    }

    fn draw_status(&mut self) {
        self.stdout.goto(23, 9).unwrap();
        self.stdout.write(b"level:").unwrap();
        self.stdout.goto(23, 10).unwrap();
        self.stdout.write(self.grid.get_level().to_string().as_bytes()).unwrap();
        self.stdout.goto(23, 12).unwrap();
        self.stdout.write(b"lines cleared:").unwrap();
        self.stdout.goto(23, 13).unwrap();
        self.stdout.write(self.grid.get_lines_cleared().to_string().as_bytes()).unwrap();
    }

    fn draw_grid_boundaries(&mut self) {
        for y in 0..21 {
            self.stdout.goto(21, y).unwrap();
            self.stdout.write(b"=").unwrap();
            self.stdout.goto(0, y).unwrap();
            self.stdout.write(b"=").unwrap();
            if y == 20 {
                for x in 1..21 {
                    self.stdout.goto(x, y).unwrap();
                    self.stdout.write(b"=").unwrap();
                }
            }
        }
    }

    fn draw_grid(&mut self) {
        for i in 0..(grid::GRID_WIDTH * grid::GRID_HEIGHT) {
            let grid_2D = grid::Grid1D{x: i}.to_2D(grid::GRID_WIDTH);
            let (x, y) = (grid_2D.x, grid_2D.y);
            match self.grid.grid[i as usize] {
                grid::BlockType::I => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(5, 5, 5)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::J => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(5, 0, 0)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::L => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(0, 5, 0)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::O => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(0, 0, 5)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::S => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(5, 5, 0)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::T => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(5, 0, 5)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::Z => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(0, 5, 5)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::Garbage => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(2, 2, 2)).unwrap();
                    self.stdout.write(b"  ").unwrap();
                },
                grid::BlockType::None => {
                    self.stdout.goto((x * 2 + 1) as u16, (19 - y) as u16).unwrap();
                    self.stdout.bg_color(Color::Rgb(0, 0, 0)).unwrap();
                    self.stdout.write(b" .").unwrap();
                },
            }
        }
        self.stdout.reset().unwrap();
    }
}


