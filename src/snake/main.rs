extern crate termion;
extern crate extra;

use termion::{IntoRawMode, TermWrite, Color, async_stdin};
use std::io::{stdout, stdin, Read, Write};
use std::time::{Instant, Duration};
use std::collections::VecDeque;
use std::thread::sleep;
use extra::rand::Randomizer;

#[cfg(target_os = "redox")]
mod graphics {
    pub const TOP_LEFT_CORNER: &'static str = "+";
    pub const TOP_RIGHT_CORNER: &'static str = "+";
    pub const BOTTOM_LEFT_CORNER: &'static str = "+";
    pub const BOTTOM_RIGHT_CORNER: &'static str = "+";
    pub const VERTICAL_WALL: &'static str = "|";
    pub const HORIZONTAL_WALL: &'static str = "-";
    pub const VERTICAL_SNAKE_BODY: &'static str = "|";
    pub const HORIZONTAL_SNAKE_BODY: &'static str = "-";
    pub const SNAKE_HEAD: &'static str = "@";
    pub const FOOD: &'static str = "$";
    pub const GAME_OVER: &'static str = "+-----------------+\n\r\
                                         |----Game over----|\n\r\
                                         | r | replay      |\n\r\
                                         | q | quit        |\n\r\
                                         +-----------------+";
}

#[cfg(not(target_os = "redox"))]
mod graphics {
    pub const TOP_LEFT_CORNER: &'static str = "╔";
    pub const TOP_RIGHT_CORNER: &'static str = "╗";
    pub const BOTTOM_LEFT_CORNER: &'static str = "╚";
    pub const BOTTOM_RIGHT_CORNER: &'static str = "╝";
    pub const VERTICAL_WALL: &'static str = "║";
    pub const HORIZONTAL_WALL: &'static str = "═";
    pub const VERTICAL_SNAKE_BODY: &'static str = "║";
    pub const HORIZONTAL_SNAKE_BODY: &'static str = "═";
    pub const SNAKE_HEAD: &'static str = "⧲";
    pub const FOOD: &'static str = "⊛";
    pub const GAME_OVER: &'static str = "╔═════════════════╗\n\r\
                                         ║───┬Game over────║\n\r\
                                         ║ r ┆ replay      ║\n\r\
                                         ║ q ┆ quit        ║\n\r\
                                         ╚═══╧═════════════╝";
}

use self::graphics::*;

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
    /// Move's the Snake along it's current direction
    fn crawl(&self) -> BodyPart {
        let mut x = self.x;
        let mut y = self.y;

        match self.direction {
            Direction::Up => y += 1,
            Direction::Down => y -= 1,
            Direction::Left => x -= 1,
            Direction::Right => x += 1,
        }

        BodyPart {
            x: x,
            y: y,
            direction: self.direction,
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
    /// The randomizer
    rand: Randomizer,
}

impl<R: Read, W: Write> Game<R, W> {
    /// Start the game loop.
    ///
    /// This will listen to events and do the appropriate actions.
    fn start(&mut self) {
        self.stdout.hide_cursor().unwrap();
        
        // Display a small prompt and
        // wait for the user to start the game
        self.game_start();
        self.reset();
        
        let mut before = Instant::now();

        loop {
            let interval = 1000 / self.speed;
            let now = Instant::now();
            let dt = (now.duration_since(before).subsec_nanos() / 1_000_000) as u64;

            if dt < interval {
                sleep(Duration::from_millis(interval - dt));
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
    }

    /// Update the game.
    ///
    /// This will receive and process input. As well as update the game world.
    /// Returns false if the game is supposed to be closed.
    fn update(&mut self) -> bool {
        let mut key_bytes = [0];
        self.stdin.read(&mut key_bytes).unwrap();

        self.rand.write_u8(key_bytes[0]);

        match key_bytes[0] {
            b'q' => return false,
            b'k' => self.turn_snake(Direction::Up),
            b'j' => self.turn_snake(Direction::Down),
            b'h' => self.turn_snake(Direction::Left),
            b'l' => self.turn_snake(Direction::Right),
            _ => {},
        }

        self.move_snake();

        true
    }

    /// Check if the Snake is overlapping a wall or a body part
    fn check_game_over(&mut self) -> bool {
        let head = &self.snake.body.back().unwrap();

        self.snake.body.iter().filter(|part| (head.x, head.y) == (part.x, part.y)).count() > 1
        || head.x == 0
        || head.y == 0
        || head.x == self.width as u16
        || head.y == self.height as u16 - 1
    }

    /// Grows the Snake's tail
    fn grow_snake(&mut self) {
        let x; 
        let y;
        let direction;

        {
            let tail = &self.snake.body.front().unwrap();

            x = match tail.direction {
                Direction::Left => tail.x + 1,
                Direction::Right => tail.x - 1,
                _ => tail.x,
            };

            y = match tail.direction {
                Direction::Up => tail.y + 1,
                Direction::Down => tail.y - 1,
                _ => tail.y,
            };

            direction = tail.direction;
        }

        self.snake.body.push_front(BodyPart {
            x: x,
            y: y,
            direction: direction,
        });
    }

    /// Checks if the Snake is overlapping the food
    fn check_eating(&mut self) -> bool {
        let head = &self.snake.body.back().unwrap();
        (head.x, head.y) == (self.food.x, self.food.y)
    }

    fn clear_snake(&mut self) {
        for part in &self.snake.body {
            self.stdout.goto(part.x, part.y).unwrap();
            self.stdout.write(b" ").unwrap();
        }
    }

    fn move_snake(&mut self) {
        {
            let tail = self.snake.body.pop_front().unwrap();
            self.stdout.goto(tail.x, tail.y).unwrap();
            self.stdout.write(b" ").unwrap();
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

        self.snake.body.push_back(BodyPart {
            x: x,
            y: y,
            direction: direction
        });
    }

    fn turn_snake(&mut self, direction: Direction) {
        match (direction, self.snake.direction) {
            (Direction::Up, Direction::Down)
            | (Direction::Down, Direction::Up)
            | (Direction::Left, Direction::Right)
            | (Direction::Right, Direction::Left) => return,
            _ => self.snake.direction = direction,
        }
    }

    fn game_start(&mut self) {
        self.stdout.goto(0, 0).unwrap();
        self.stdout.write_fmt(format_args!("╔══════════════════════════════════╗\n\r\
                                            ║────Welcome to Snake for Redox────║\n\r\
                                            ║──────────────────────────────────║\n\r\
                                            ║ space ┆ start game               ║\n\r\
                                            ║──────────────────────────────────║\n\r\
                                            ║   h   ┆ left                     ║\n\r\
                                            ║   j   ┆ down                     ║\n\r\
                                            ║   k   ┆ up                       ║\n\r\
                                            ║   l   ┆ right                    ║\n\r\
                                            ╚═══╧══════════════════════════════╝
                                    ")).unwrap();
        loop {
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();

            match buf[0] {
                b' ' => return,
                _ => {},
            }
        }
    }

    fn game_over(&mut self) -> bool {
        self.stdout.goto(0, 0).unwrap();

        self.stdout.write(GAME_OVER.as_bytes()).unwrap();
        self.stdout.goto((self.width as u16 / 2) - 3, self.height as u16 / 2).unwrap();
        self.stdout.write_fmt(format_args!("SCORE: {}", self.score)).unwrap();
        self.stdout.flush().unwrap();

        loop {
            // Repeatedly read a single byte.
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();

            match buf[0] {
                b'r' => return true,
                b'q' => return false,
                _ => {},
            }
        }
    }

    fn draw_horizontal_line(&mut self, chr: &str, width: u16) {
        for _ in 0..width { self.stdout.write(chr.as_bytes()).unwrap(); }
    }

    /// Move the snake's food.
    fn move_food(&mut self) {
        loop {
            let x = (self.rand.read_u8() as u16 % (self.width as u16 - 2)) + 1;
            let y = (self.rand.read_u8() as u16 % (self.height as u16 - 2)) + 1;

            if self.snake.body.iter().filter(|part| {
                (x, y) == (part.x, part.y)
            }).next().is_some() {
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
        self.stdout.write(FOOD.as_bytes()).unwrap();

        self.stdout.flush().unwrap();
        self.stdout.reset().unwrap();
    }

    /// Draws the snake.
    fn draw_snake(&mut self) {
        self.stdout.reset().unwrap();

        for part in &self.snake.body {
            self.stdout.goto(part.x, part.y).unwrap();
            match part.direction {
                Direction::Up | Direction::Down => self.stdout.write(VERTICAL_SNAKE_BODY.as_bytes()).unwrap(),
                Direction::Left | Direction::Right => self.stdout.write(HORIZONTAL_SNAKE_BODY.as_bytes()).unwrap(),
            };
        }

        let head = self.snake.body.back().unwrap();

        self.stdout.goto(head.x, head.y).unwrap();
        self.stdout.write(SNAKE_HEAD.as_bytes()).unwrap();

        self.stdout.flush().unwrap();
    }

    /// Draws the game walls.
    fn draw_walls(&mut self) {
        let width: u16 = self.width as u16;
        let height: u16 = self.height as u16;

        self.stdout.color(Color::Red).unwrap();

        self.stdout.goto(0, 0).unwrap();
        self.stdout.write(TOP_LEFT_CORNER.as_bytes()).unwrap();
        self.stdout.goto(1, 0).unwrap();
        self.draw_horizontal_line(HORIZONTAL_WALL, width - 2);
        self.stdout.goto(width - 1, 0).unwrap();
        self.stdout.write(TOP_RIGHT_CORNER.as_bytes()).unwrap();

        self.stdout.goto(0, 2).unwrap();
        for y in 1..height {
            self.stdout.goto(0, y as u16).unwrap();
            self.stdout.write(VERTICAL_WALL.as_bytes()).unwrap();

            self.stdout.goto((self.width - 1) as u16, y as u16).unwrap();
            self.stdout.write(VERTICAL_WALL.as_bytes()).unwrap();
        }

        self.stdout.goto(0, height - 1).unwrap();
        self.stdout.write(BOTTOM_LEFT_CORNER.as_bytes()).unwrap();
        self.stdout.goto(1, height - 1).unwrap();
        self.draw_horizontal_line(HORIZONTAL_WALL, width - 2);
        self.stdout.goto(width - 1, height - 1).unwrap();
        self.stdout.write(BOTTOM_RIGHT_CORNER.as_bytes()).unwrap();

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
            body: VecDeque::new(),
        },
        food: Food {
            x: 0,
            y: 0,
        },
        score: 0,
        speed: 0,
        rand: Randomizer::new(0),
    };

    game.reset();
    game.start();

    game.stdout.clear().unwrap();
    game.stdout.flush().unwrap();
    game.stdout.restore().unwrap();
}

fn main() {
    init(80, 40);
}
