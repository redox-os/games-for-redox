extern crate termion;

use termion::{IntoRawMode, TermWrite, Color, async_stdin};
use std::io::{stdout, stdin, Read, Write};
use std::time::{Instant, Duration};
use std::collections::VecDeque;

#[derive(PartialEq, Copy, Clone)]
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

/// Snake's Food
struct Food {
    x: u16,
    y: u16,
}

impl BodyPart {
    fn crawl(&self) -> BodyPart {
        let mut x = self.x;
        let mut y = self.y;
        let new_direction: Direction;

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
    /// Snake's Food
    food: Food,
    /// Speed
    speed: u64,
    /// Game Score
    score: i32,
    /// Interval between frames
    interval: u64,
    /// This will be modified when a random value is read or written.
    seed: usize,
}

impl<R: Read, W: Write> Game<R, W> {
    /// Start the game loop.
    ///
    /// This will listen to events and do the appropriate actions.
    fn start(&mut self) {
        self.stdout.hide_cursor().unwrap();

        let mut before = Instant::now();

        loop {
            self.interval = 1000 / self.speed;
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

            if self.check_game_over() {
                if self.game_over() {
                    self.reset();
                    continue;
                } else {
                    return;
                }
            }

            if self.check_eating() {
                self.score += 1;
                self.speed += 4;
                self.grow_snake();
                self.move_food();
            }

            self.clear_snake();
            self.draw_snake();
            self.draw_food();

            self.stdout.flush().unwrap();
            self.stdout.reset().unwrap();
        }
    }

    /// Reset the game.
    ///
    /// This will display the starting play area.
    fn reset(&mut self) {
        self.stdout.clear().unwrap();
        self.stdout.flush().unwrap();
        self.stdout.reset().unwrap();

        self.draw_walls();

        self.snake = Snake {
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
        };
        
        self.food = Food {
            x: self.width as u16 / 2,
            y: self.height as u16 / 2,
        };
        
        self.score = 0;
        self.speed = 10;
        self.interval = 0;
        self.seed = 0;
    }

    /// Update the game.
    ///
    /// This will receive and process input. As well as update the game world.
    /// Returns false if the game is supposed to be closed.
    fn update(&mut self) -> bool {
        let mut key_bytes = [0];
        self.stdin.read(&mut key_bytes).unwrap();

        self.write_rand(key_bytes[0]);

        match key_bytes[0] {
            b'q' => return false,
            b'k' => self.turn_snake(Direction::Up),
            b'j' => self.turn_snake(Direction::Down),
            b'h' => self.turn_snake(Direction::Left),
            b'l' => self.turn_snake(Direction::Right),
            b'0' => {},
            _ => {},
        }

        self.move_snake();

        true
    }

    /// Read a number from the randomizer.
    fn read_rand(&mut self) -> u8 {
        self.seed ^= self.seed.rotate_right(4).wrapping_add(0x25A45B35C4FD3DF2);
        self.seed ^= self.seed >> 7;
        self.seed as u8
    }

    /// This is used for collecting entropy to the randomizer.
    fn write_rand(&mut self, b: u8) {
        self.seed ^= b as usize;
        self.read_rand();
    }

    /// Check if the Snake is overlapping a wall or a body part
    fn check_game_over(&mut self) -> bool {
        let head = &self.snake.body.back().unwrap();

        if self.snake.body.iter().filter(|part| {
            (head.x, head.y) == (part.x, part.y)
        }).count() > 1 {
            return true;
        }

        match (head.x, head.y) {
            (0, _) => true,
            (_, 0) => true,
            (x, _) if x == self.width as u16 => true,
            (_, y) if y == self.height as u16 - 1 => true,
            _ => false,
        }
    }

    /// Grows the Snake's tail
    fn grow_snake(&mut self) {
        let (x, y, direction) = {
            let tail = &self.snake.body.front().unwrap();

            (match tail.direction {
                Direction::Left => tail.x + 1,
                Direction::Right => tail.x - 1,
                _ => tail.x,
            },
            match tail.direction {
                Direction::Up => tail.y + 1,
                Direction::Down => tail.y - 1,
                _ => tail.y,
            },
            tail.direction)
        };

        self.snake.body.push_front(BodyPart {
            x: x,
            y: y,
            direction: direction,
        });
    }

    /// Checks if the Snake is overlapping the food
    fn check_eating(&mut self) -> bool {
        let head = &self.snake.body.back().unwrap();
        if (head.x, head.y) == (self.food.x, self.food.y) {
            return true;
        }
        false
    }

    fn clear_snake(&mut self) {
        for part in &self.snake.body {
            self.stdout.goto(part.x, part.y).unwrap();
            self.stdout.write(" ".as_bytes()).unwrap();
        }
    }

    fn move_snake(&mut self) {
        {
            let tail = self.snake.body.pop_front().unwrap();
            self.stdout.goto(tail.x, tail.y).unwrap();
            self.stdout.write(" ".as_bytes()).unwrap();
        }

        for part in self.snake.body.iter_mut() {
            part.crawl();
        }

        let (x, y, direction) = {
            let head = self.snake.body.back().unwrap();
            
            match self.snake.direction {
                Direction::Up => (head.x, head.y - 1, Direction::Up),
                Direction::Down => (head.x, head.y + 1, Direction::Down),
                Direction::Left => (head.x - 1, head.y, Direction::Left),
                Direction::Right => (head.x + 1, head.y, Direction::Right),
            }
        };

        self.snake.body.push_back(BodyPart{
            x: x,
            y: y,
            direction: direction
        });
    }

    fn turn_snake(&mut self, direction: Direction) {
        match direction {
            Direction::Up if self.snake.direction == Direction::Down => return,
            Direction::Down if self.snake.direction == Direction::Up => return,
            Direction::Left if self.snake.direction == Direction::Right => return,
            Direction::Right if self.snake.direction == Direction::Left => return,
            _ => self.snake.direction = direction,
        }
    }

    fn game_over(&mut self) -> bool {
        self.stdout.goto(0, 0).unwrap();

        self.stdout.write_fmt(format_args!("╔═════════════════╗\n\r\
                                            ║────Game over────║\n\r\
                                            ║ r ┆ replay      ║\n\r\
                                            ║ q ┆ quit        ║\n\r\
                                            ╚═══╧═════════════╝
                           ")).unwrap();
        self.stdout.goto((self.width as u16 / 2) - 3, self.height as u16 / 2).unwrap();
        self.stdout.write_fmt(format_args!("SCORE: {}", self.score)).unwrap();
        self.stdout.flush().unwrap();

        loop {
            // Repeatedly read a single byte.
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();

            match buf[0] {
                b'r' => {
                    return true;
                },
                b'q' => return false,
                _ => {},
            }
        }
    }

    fn draw_vertical_line(&mut self, chr: &str, width: u16) {
        for _ in 0..width { self.stdout.write(chr.as_bytes()).unwrap(); }
    }

    /// Move the snake's food.
    fn move_food(&mut self) {
        loop {
            let x = (self.read_rand() as u16 % (self.width as u16 - 2)) + 1;
            let y = (self.read_rand() as u16 % (self.height as u16 - 2)) + 1;

            if self.snake.body.iter().filter(|part| {
                (x, y) == (part.x, part.y)
            }).count() > 0 {
                continue;
            } else {
                self.food.x = x;
                self.food.y = y;
                break;
            }
        };
    }

    /// Draws the snake's food.
    fn draw_food(&mut self) {
        self.stdout.reset().unwrap();

        self.stdout.goto(self.food.x, self.food.y).unwrap();
        self.stdout.write(b"*").unwrap();

        self.stdout.flush().unwrap();
        self.stdout.reset().unwrap();
    }

    /// Draws the snake.
    fn draw_snake(&mut self) {
        self.stdout.reset().unwrap();

        for part in &self.snake.body {
            self.stdout.goto(part.x, part.y).unwrap();
            match part.direction {
                Direction::Up | Direction::Down => self.stdout.write("║".as_bytes()).unwrap(),
                Direction::Left | Direction::Right => self.stdout.write("═".as_bytes()).unwrap(),
            };
        }

        let head = self.snake.body.back().unwrap();

        self.stdout.goto(head.x, head.y).unwrap();
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
        self.stdout.write("╔".as_bytes()).unwrap();
        self.stdout.goto(1, 0).unwrap();
        self.draw_vertical_line("═", width - 2);
        self.stdout.goto(width - 1, 0).unwrap();
        self.stdout.write("╗".as_bytes()).unwrap();

        self.stdout.goto(0, 2).unwrap();
        for y in 1..height {
            self.stdout.goto(0, y as u16).unwrap();
            self.stdout.write("║".as_bytes()).unwrap();

            self.stdout.goto((self.width - 1) as u16, y as u16).unwrap();
            self.stdout.write("║".as_bytes()).unwrap();
        }

        self.stdout.goto(0, height - 1).unwrap();
        self.stdout.write("╚".as_bytes()).unwrap();
        self.stdout.goto(1, height - 1).unwrap();
        self.draw_vertical_line("═", width - 2);
        self.stdout.goto(width - 1, height - 1).unwrap();
        self.stdout.write("╝".as_bytes()).unwrap();
        
        self.stdout.flush().unwrap();
        self.stdout.reset().unwrap();
    }
}

/// Initializes the game.
fn init(width: usize, height: usize) {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = async_stdin();

    stdout.clear().unwrap();
    stdout.goto(0, 0).unwrap();
    stdout.flush().unwrap();

    let mut game = Game {
        width: width,
        height: height,
        stdin: stdin,
        stdout: stdout,
        snake: Snake {
            direction: Direction::Right,
            body: vec![].into_iter().collect(),
        },
        food: Food {
            x: 0,
            y: 0,
        },
        score: 0,
        speed: 0,
        interval: 0,
        seed: 0,
    };

    game.reset();
    game.start();

    game.stdout.restore().unwrap();
}

fn main() {
    init(80, 40);
}
