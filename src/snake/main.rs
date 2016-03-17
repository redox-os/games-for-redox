extern crate libterm;

use libterm::{IntoRawMode, TermRead, TermWrite, Color, async_stdin};
use std::io::{stdout, stdin, Read, Write};
use std::time::{Instant, Duration};
use std::collections::VecDeque;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Snake's Body Part location and direction
struct BodyPart {
    x: u16,
    y: u16,
    direction: Direction,
}

impl BodyPart {
    fn crawl(&self) -> BodyPart {
        let mut x = self.x;
        let mut y = self.y;
        let mut new_direction: Direction = Direction::Up;

        match self.direction {
            Direction::Up => { 
                y += 1;
                new_direction = Direction::Up;
            },
            Direction::Down => { 
                y -= 1;
                new_direction = Direction::Down;
            },
            Direction::Left => {
                x -= 1;
                new_direction = Direction::Left;
            },
            Direction::Right => {
                x += 1;
                new_direction = Direction::Right;
            },
        };

        BodyPart {
            x: x,
            y: y,
            direction: new_direction
        }
    }
}

/// Snake
struct Snake {
    size: u16,
    direction: Direction,
    body: VecDeque<BodyPart>,
}

/// The game state.
struct Game<R, W> {
    /// The play area width.
    width: usize,
    /// The play area height.
    height: usize,
    /// Standard input.
    stdin: R,
    /// Standard output.
    stdout: W,
    /// Snake
    snake: Snake,
    /// Speed
    speed: u64,
    /// Game Score
    score: i32,
    /// Interval between frames
    interval: u64,
}

impl<R: Read, W: Write> Game<R, W> {
    /// Start the game loop.
    ///
    /// This will listen to events and do the appropriate actions.
    fn start(&mut self) {
        self.stdout.hide_cursor().unwrap();
        self.draw_walls();

        self.interval = 1000 / self.speed;
        let mut before = Instant::now();

        loop {
            let now = Instant::now();
            let dt = (now.duration_since(before).subsec_nanos() / 1_000_000) as u64;

            if dt < self.interval {
                std::thread::sleep(Duration::from_millis(self.interval - dt));
                continue;
            }

            before = now;

            if !self.update() {
                return;
            }

            self.clear_snake();
            self.draw_snake();

            self.stdout.flush().unwrap();
            self.stdout.reset().unwrap();
        }
    }

    /// Reset the game.
    ///
    /// This will display the starting play area.
    fn reset(&mut self) {
    }

    /// Update the game.
    ///
    /// This will receive and process input. As well as update the game world.
    /// Returns false if the game is supposed to be closed.
    fn update(&mut self) -> bool {
        let mut key_bytes = [0];
        self.stdin.read(&mut key_bytes).unwrap();

        match key_bytes[0] {
            b'q' => return false,
            b'k' => self.snake.direction = Direction::Up,
            b'j' => self.snake.direction = Direction::Down,
            b'h' => self.snake.direction = Direction::Left,
            b'l' => self.snake.direction = Direction::Right,
            b'0' => {},
            _ => {},
        }

        self.move_snake();

        true
    }

    fn clear_snake(&mut self) {
        for part in &self.snake.body {
            self.stdout.goto(part.x, part.y);
            self.stdout.write(" ".as_bytes()).unwrap();
        }
    }

    fn move_snake(&mut self) {
        self.stdout.write("".as_bytes()).unwrap();

        self.snake.body.pop_front();

        for part in self.snake.body.iter_mut() {
            part.crawl();
        }

        let head = self.snake.body.back().unwrap();

        let (mut x, mut y, mut direction) = match head.direction {
            Direction::Up => (head.x, head.y + 1, Direction::Up),
            Direction::Down => (head.x, head.y - 1, Direction::Down),
            Direction::Left => (head.x - 1, head.y, Direction::Left),
            Direction::Right => (head.x + 1, head.y, Direction::Right),
        };

        self.snake.body.push_back(BodyPart{
            x: x,
            y: y,
            direction: direction
        });
    }

    fn draw_vertical_line(&mut self, chr: &str, width: u16) {
        for _ in 0..width { self.stdout.write(chr.as_bytes()); }
    }

    /// Draws the snake.
    fn draw_snake(&mut self) {
        self.stdout.reset().unwrap();

        for part in &self.snake.body {
            self.stdout.goto(part.x, part.y);
            match part.direction {
                Direction::Up | Direction::Down => self.stdout.write("║".as_bytes()).unwrap(),
                Direction::Left | Direction::Right => self.stdout.write("═".as_bytes()).unwrap(),
            };
        }

        let head = self.snake.body.back().unwrap();

        self.stdout.goto(head.x, head.y);
        self.stdout.write("@".as_bytes()).unwrap();

        self.stdout.flush().unwrap();
        self.stdout.reset().unwrap();
    }

    /// Draws the game walls.
    fn draw_walls(&mut self) {
        let width: u16 = self.width as u16;
        let height: u16 = self.height as u16;

        self.stdout.color(Color::Red).unwrap();

        self.stdout.goto(0, 0).unwrap();
        self.stdout.write("╔".as_bytes());
        self.stdout.goto(1, 0).unwrap();
        self.draw_vertical_line("═", width - 2);
        self.stdout.goto(width - 1, 0).unwrap();
        self.stdout.write("╗".as_bytes());

        self.stdout.goto(0, 2).unwrap();
        for y in 1..height {
            self.stdout.goto(0, y as u16);
            self.stdout.write("║".as_bytes()).unwrap();

            self.stdout.goto((self.width - 1) as u16, y as u16);
            self.stdout.write("║".as_bytes()).unwrap();
        }

        self.stdout.goto(0, height - 1).unwrap();
        self.stdout.write("╚".as_bytes());
        self.stdout.goto(1, height - 1).unwrap();
        self.draw_vertical_line("═", width - 2);
        self.stdout.goto(width - 1, height - 1).unwrap();
        self.stdout.write("╝".as_bytes());
        
        self.stdout.flush().unwrap();
        self.stdout.reset().unwrap();
    }
}

/// Initializes the game.
fn init(width: usize, height: usize) {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = async_stdin();

    stdout.clear().unwrap();
    stdout.goto(0, 0).unwrap();
    stdout.flush().unwrap();

    let mut game = Game {
        width: width,
        height: height,
        stdin: stdin,
        stdout: stdout,
        snake: Snake {
            size: 10,
            direction: Direction::Right,
            body: vec![
                BodyPart { x: 10, y: 10, direction: Direction::Right},
                BodyPart { x: 11, y: 10, direction: Direction::Right},
                BodyPart { x: 12, y: 10, direction: Direction::Right},
                BodyPart { x: 13, y: 10, direction: Direction::Right},
                BodyPart { x: 14, y: 10, direction: Direction::Right},
                BodyPart { x: 15, y: 10, direction: Direction::Right},
                BodyPart { x: 16, y: 10, direction: Direction::Right},
                BodyPart { x: 17, y: 10, direction: Direction::Right},
                BodyPart { x: 18, y: 10, direction: Direction::Right},
                BodyPart { x: 19, y: 10, direction: Direction::Right},
            ].into_iter().collect(),
        },
        score: 0,
        speed: 10,
        interval: 0,
    };

    game.reset();
    game.start();

    game.stdout.restore().unwrap();
}

fn main() {
    init(100, 100);
}
