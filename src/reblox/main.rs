#![feature(nll)]

extern crate termion;
extern crate extra;

mod grid;

use termion::{async_stdin, clear, color, cursor, style};
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{self, Write, Read, Result};
use std::thread;
use std::time;

fn main() {
    let stdout = io::stdout();

    let mut game = Game::new(async_stdin(), stdout.lock());

    if let Err(err) = game.run() {
        write!(io::stderr(), "Failed to run reblox: {}", err).unwrap();
    }
}

impl grid::BlockType {
    pub fn to_color(&self) -> termion::color::AnsiValue {
        match *self {
            grid::BlockType::I => color::AnsiValue(15),
            grid::BlockType::J => color::AnsiValue(9),
            grid::BlockType::L => color::AnsiValue(10),
            grid::BlockType::O => color::AnsiValue(12),
            grid::BlockType::S => color::AnsiValue(11),
            grid::BlockType::T => color::AnsiValue(13),
            grid::BlockType::Z => color::AnsiValue(14),
            grid::BlockType::Garbage => color::AnsiValue(7),
            _ => color::AnsiValue(0),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match *self {
            grid::BlockType::None => " .",
            _ => "  "
        }
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

    fn run(&mut self) -> Result<()> {
        write!(self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Hide)?;
        self.draw_grid_boundaries()?;
        self.draw_usage()?;
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
                    b'h' => self.grid.activate_hold(),
                    b' ' => self.grid.fall(),
                    _ => (),
                }
                self.grid.rng.read_u8();
                self.grid.rng.write_u8(b[0]);
                b[0] = 0;
            }
            self.grid.update(time::Duration::from_millis(50));
            self.draw_status()?;
            self.draw_next()?;
            self.draw_held()?;
            self.draw_grid()?;
            self.stdout.flush()?;
        }
        write!(self.stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Show)?;

        Ok(())
    }

    fn draw_usage(&mut self) -> Result<()> {
        write!(self.stdout, "{}q - quit", cursor::Goto(24, 1))?;
        write!(self.stdout, "{}asd - move", cursor::Goto(24, 2))?;
        write!(self.stdout, "{}jk - rotate", cursor::Goto(24, 3))?;
        write!(self.stdout, "{}space - drop", cursor::Goto(24, 4))?;
        write!(self.stdout, "{}r - reset", cursor::Goto(24, 5))?;

        Ok(())
    }

    fn draw_next(&mut self) -> Result<()> {
        write!(self.stdout, "{}next:", cursor::Goto(24, 16))?;
        for x in 0..4 {
            for y in 0..4 {
                write!(self.stdout, "{}  ", cursor::Goto(24 + x * 2, 17 + y))?;
            }
        }

        write!(self.stdout, "{}", color::Bg(self.grid.get_next_type().to_color()))?;
        let piece_pos = grid::BlockPos::new(10, self.grid.get_next_rot(), self.grid.get_next_type(), 4).positions;
        for i in 0..4 {
            let (x, y) = (piece_pos[i] % 4, piece_pos[i] / 4);
            write!(self.stdout, "{}  ", cursor::Goto(24 + x as u16 * 2, 17 + (3 - y as u16)))?;
        }
        write!(self.stdout, "{}", style::Reset)?;

        Ok(())
    }

    fn draw_status(&mut self) -> Result<()> {
        write!(self.stdout, "{}level:", cursor::Goto(24, 12))?;
        write!(self.stdout, "{}{}", cursor::Goto(24, 13), self.grid.get_level().to_string())?;
        write!(self.stdout, "{}lines cleared:", cursor::Goto(24, 14))?;
        let lines_cleared = self.grid.get_lines_cleared();
        write!(self.stdout, "{}{}", cursor::Goto(24, 15), lines_cleared.to_string())?;
        if lines_cleared == 0 {
            // Clears left-over-text from scores > 10 after resetting
            write!(self.stdout, "          ")?;
        }

        Ok(())
    }

    fn draw_grid_boundaries(&mut self) -> Result<()> {
        for y in 1..22 {
            write!(self.stdout, "{}=", cursor::Goto(22, y))?;
            write!(self.stdout, "{}=", cursor::Goto(1, y))?;
            if y == 21 {
                for x in 2..22 {
                    write!(self.stdout, "{}=", cursor::Goto(x, y))?;
                }
            }
        }

        Ok(())
    }

    fn draw_grid(&mut self) -> Result<()> {
        for i in 0..(grid::GRID_WIDTH * grid::GRID_HEIGHT) {
            let grid_2d: grid::Grid2D = grid::Grid1D{x: i, width: grid::GRID_WIDTH}.into();
            let (x, y) = (grid_2d.x, grid_2d.y);

            let block_type = self.grid.grid[i as usize];

            write!(self.stdout, "{}{}{}", cursor::Goto((x * 2 + 2) as u16, (20 - y) as u16), color::Bg(block_type.to_color()), block_type.to_str())?;
        }
        write!(self.stdout, "{}", style::Reset)?;

        Ok(())
    }

    fn draw_held(&mut self) -> Result<()> {
        write!(self.stdout, "{}held (h to hold):", cursor::Goto(24, 6))?;
        for x in 0..4 {
            for y in 0..4 {
                write!(self.stdout, "{}  ", cursor::Goto(24 + x * 2, 7 + y))?;
            }
        }

        let held_type = self.grid.get_held_type();
        if held_type != grid::BlockType::None {
            write!(self.stdout, "{}", color::Bg(held_type.to_color()))?;
            let piece_pos = grid::BlockPos::new(10, self.grid.get_held_rot(), held_type, 4).positions;
            for i in 0..4 {
                let (x, y) = (piece_pos[i] % 4, piece_pos[i] / 4);
                write!(self.stdout, "{}  ", cursor::Goto(24 + x as u16 * 2, 7 + (3 - y as u16)))?;
            }
            write!(self.stdout, "{}", style::Reset)?;
        }

        Ok(())
    }
}
