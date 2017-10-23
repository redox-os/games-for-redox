extern crate termion;
extern crate extra;

use termion::{async_stdin, clear, color, cursor, style};
use termion::raw::IntoRawMode;
use std::io::{stdout, stdin, Read, Write};
use std::time::{Instant, Duration};
use std::collections::VecDeque;
use std::thread::sleep;
use extra::rand::Randomizer;

mod graphics {
    pub const TOP_LEFT_CORNER: &'static str = "╔";
    pub const TOP_RIGHT_CORNER: &'static str = "╗";
    pub const BOTTOM_LEFT_CORNER: &'static str = "╚";
    pub const BOTTOM_RIGHT_CORNER: &'static str = "╝";
    pub const VERTICAL_WALL: &'static str = "║";
    pub const HORIZONTAL_WALL: &'static str = "═";
    pub const VERTICAL_SNAKE_BODY: &'static str = "║";
    pub const HORIZONTAL_SNAKE_BODY: &'static str = "═";
    pub const SNAKE_HEAD: &'static str = "@";
    pub const FOOD: &'static str = "o";
    pub const GAME_OVER: &'static str = "╔═════════════════╗\n\r\
                                         ║───┬GAME OVER────║\n\r\
                                         ║ r ┆ replay      ║\n\r\
                                         ║ q ┆ quit        ║\n\r\
                                         ╚═══╧═════════════╝";
    pub const GAME_START_PROMPT: &'static str = "╔══════════════════════════════╗\n\r\
                                                 ║──Welcome to Snake for Redox──║\n\r\
                                                 ║──────────────────────────────║\n\r\
                                                 ║ h ┆ left                     ║\n\r\
                                                 ║ j ┆ down      Press space    ║\n\r\
                                                 ║ k ┆ up         to BEGIN!     ║\n\r\
                                                 ║ l ┆ right                    ║\n\r\
                                                 ╚═══╧══════════════════════════╝";

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
        write!(self.stdout, "{}", cursor::Hide).unwrap();
        self.game_start_prompt();
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

            write!(self.stdout, "{}", style::Reset).unwrap();
            self.stdout.flush().unwrap();
        }
    }

    /// Reset the game.
    ///
    /// This will display the starting play area.
    fn reset(&mut self) {
        write!(self.stdout, "{}{}", clear::All, style::Reset);

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
            b'k' | b'w' => self.turn_snake(Direction::Up),
            b'j' | b's' => self.turn_snake(Direction::Down),
            b'h' | b'a' => self.turn_snake(Direction::Left),
            b'l' | b'd' => self.turn_snake(Direction::Right),
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
            write!(self.stdout, "{} ", cursor::Goto(part.x + 1, part.y + 1)).unwrap();
        }
    }

    fn move_snake(&mut self) {
        {
            let tail = self.snake.body.pop_front().unwrap();
            write!(self.stdout, "{} ", cursor::Goto(tail.x + 1, tail.y + 1)).unwrap();
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

    fn game_start_prompt(&mut self) {
        write!(self.stdout, "{}{}", cursor::Goto(1, 1), GAME_START_PROMPT).unwrap();
        self.stdout.flush().unwrap();
        loop {
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();
            self.rand.write_u8(buf[0]);
            if buf[0] == b' ' {
                return;
            }
        }
    }

    fn game_over(&mut self) -> bool {
        write!(self.stdout, "{}{}", cursor::Goto(1, 1), GAME_OVER).unwrap();
        write!(self.stdout, "{}", cursor::Goto((self.width as u16 / 2) - 2, self.height as u16 / 2 + 1)).unwrap();
        write!(self.stdout, "SCORE: {}", self.score).unwrap();
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
        write!(self.stdout, "{}", cursor::Goto(self.food.x + 1, self.food.y + 1)).unwrap();
        self.stdout.write(FOOD.as_bytes()).unwrap();
    }

    /// Draws the snake.
    fn draw_snake(&mut self) {
        for part in &self.snake.body {
            write!(self.stdout, "{}", cursor::Goto(part.x + 1, part.y + 1)).unwrap();
            match part.direction {
                Direction::Up | Direction::Down => self.stdout.write(VERTICAL_SNAKE_BODY.as_bytes()).unwrap(),
                Direction::Left | Direction::Right => self.stdout.write(HORIZONTAL_SNAKE_BODY.as_bytes()).unwrap(),
            };
        }

        let head = self.snake.body.back().unwrap();

        write!(self.stdout, "{}", cursor::Goto(head.x + 1, head.y + 1)).unwrap();
        self.stdout.write(SNAKE_HEAD.as_bytes()).unwrap();
    }

    /// Draws the game walls.
    fn draw_walls(&mut self) {
        let width: u16 = self.width as u16;
        let height: u16 = self.height as u16;

        write!(self.stdout, "{}", color::Fg(color::Red)).unwrap();

        write!(self.stdout, "{}{}", cursor::Goto(1, 1), TOP_LEFT_CORNER).unwrap();
        write!(self.stdout, "{}", cursor::Goto(2, 1)).unwrap();
        self.draw_horizontal_line(HORIZONTAL_WALL, width - 2);
        write!(self.stdout, "{}{}", cursor::Goto(width, 1), TOP_RIGHT_CORNER).unwrap();

        for y in 1..height {
            write!(self.stdout, "{}{}", cursor::Goto(1, y as u16 + 1), VERTICAL_WALL).unwrap();
            write!(self.stdout, "{}{}", cursor::Goto(self.width as u16, y as u16 + 1), VERTICAL_WALL).unwrap();
        }

        write!(self.stdout, "{}{}", cursor::Goto(1, height), BOTTOM_LEFT_CORNER).unwrap();
        write!(self.stdout, "{}", cursor::Goto(2, height)).unwrap();
        self.draw_horizontal_line(HORIZONTAL_WALL, width - 2);
        write!(self.stdout, "{}{}", cursor::Goto(width, height), BOTTOM_RIGHT_CORNER).unwrap();

        write!(self.stdout, "{}", color::Fg(color::Reset)).unwrap();
    }
}

/// Initializes the game.
fn init(width: usize, height: usize) {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let stdin = async_stdin();

    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
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

    write!(game.stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1)).unwrap();
    game.stdout.flush().unwrap();
}

fn main() {
    init(80, 40);
}
