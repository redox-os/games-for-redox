extern crate termion;
extern crate extra;

use termion::{clear, color, cursor, style};
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{self, Write, Read};
use extra::rand::Randomizer;
use std::collections::HashSet;
use std::env;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let result = parse_args();
    match result {
        Ok((b, s)) => {
            if b {
                println!("Using seed = {}", s);
                let mut game = Game::new_no_raw(
                    stdin.lock(),
                    stdout.lock(),
                    s.into_bytes());
                game.generate();
                game.simple_print_grid();
                ::std::process::exit(0);
            }
        },
        Err(c) => ::std::process::exit(c),
    }

    let (vec, hidden_count) =
        get_settings(&mut stdin.lock(), &mut stdout.lock());
    let mut game = Game::new(stdin.lock(), stdout.lock(),
        vec);

    game.generate();
    game.obfuscate_grid(hidden_count);
    game.run();
}

fn print_usage() {
    println!("Usage: [-h | --help] [-s <seed>]");
    println!("-h | --help - print this usage");
    println!("-s <seed> - generate a puzzle, print it out, and exit");
    println!("Run this program with no arguments to play redoku.");
}

fn parse_args() -> Result<(bool, String), i32> {
    let mut first_arg = true;
    let mut next_is_seed = false;
    let mut result: Result<(bool, String), i32> = Ok((false, String::new()));
    for arg in env::args() {
        if first_arg {
            first_arg = false;
            continue;
        }
        if next_is_seed {
            next_is_seed = false;
            result = Ok((true, arg));
        }
        else {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_usage();
                    return Err(0);
                },
                "-s" => {
                    if next_is_seed {
                        println!("ERROR: \"-s\" specified twice");
                        print_usage();
                        return Err(1);
                    }
                    next_is_seed = true;
                },
                _ => {
                    println!("ERROR: Got invalid arguments ({})", arg);
                    print_usage();
                    return Err(2);
                },
            }
        }
    }

    if next_is_seed {
        println!("ERROR: \"-s\" specified but no seed was given");
        return Err(3);
    }

    result
}

fn get_settings(stdin: &mut io::StdinLock, stdout: &mut io::StdoutLock) 
        -> (Vec<u8>, usize) {
    writeln!(stdout,
        "Input a seed for generating the puzzle, and hit enter").unwrap();
    writeln!(stdout,
        "(for a randomized experience, you may mash the keyboard now)")
            .unwrap();
    write!(stdout, "seed: ").unwrap();
    stdout.flush().unwrap();

    let mut vec: Vec<u8> = Vec::new();
    let mut buf: [u8; 1] = [0; 1];
    loop {
        stdin.read(&mut buf).unwrap();
        if buf[0] == b'\n' {
            break;
        }
        vec.push(buf[0]);
    }
    {
        let string = String::from_utf8_lossy(vec.as_slice());
        write!(stdout, "Using seed = {}", string).unwrap();
    }

    write!(stdout, "\n\rHow many items on the grid should be hidden? \
        (out of 81 total)\n\rnumber: ").unwrap();
    stdout.flush().unwrap();

    let mut v: Vec<u8> = Vec::new();
    loop {
        stdin.read(&mut buf).unwrap();
        if buf[0] == b'\n' {
            break;
        }
        v.push(buf[0]);
    }
    let string: String = String::from_utf8(v).unwrap();
    let mut hidden_count: usize = string.parse::<usize>().unwrap();
    if hidden_count > 81 {
        hidden_count = 81;
    }
    (vec, hidden_count)
}

struct Game<R, W: Write> {
    stdin: R,
    stdout: W,
    grid: [u8; 81],
    rng: Randomizer,
    x: usize,
    y: usize,
    show_solution: bool,
    wrong_input: bool,
    seed: String,
}

impl <R: Read, W: Write> Game<R, W> {
    fn new(stdin: R, stdout: W, vec: Vec<u8>)
            -> Game<R, RawTerminal<W>> {
        Game {
            stdin: stdin,
            stdout: stdout.into_raw_mode().unwrap(),
            grid: [0x80; 81],
            rng: Randomizer::new(vec.iter().fold(
                0u64, |acc, &x| acc + x as u64)),
            x: 0,
            y: 0,
            show_solution: false,
            wrong_input: false,
            seed: String::from_utf8(vec).unwrap(),
        }
    }

    fn new_no_raw(stdin: R, stdout: W, vec: Vec<u8>)
            -> Game<R, W> {
        Game {
            stdin: stdin,
            stdout: stdout,
            grid: [0x80; 81],
            rng: Randomizer::new(vec.iter().fold(
                0u64, |acc, &x| acc + x as u64)),
            x: 0,
            y: 0,
            show_solution: false,
            wrong_input: false,
            seed: String::from_utf8(vec).unwrap(),
        }
    }

    fn generate(&mut self) {
        write!(self.stdout, "Generating puzzle").unwrap();
        let mut possible: Vec<u8> = Vec::new();
        'outer: loop {
            for i in 0..81 {
                if self.grid[i] != 0x80 {
                    continue;
                }
                // get possible
                possible.clear();
                let set = self.get_possible_inverse(i);
                for j in 1..10 {
                    if !set.contains(&(j as u8)) {
                        possible.push(j as u8);
                    }
                }
                if possible.is_empty() {
                    write!(self.stdout, ".").unwrap();

                    let mut v: Vec<usize> = Vec::new();

                    for j in 0..81 {
                        if self.grid[j] != 0x80 {
                            v.push(j);
                        }
                    }

                    for _x in 0..7 {
                        let picked = self.rng.read_u8() as usize % v.len();
                        self.grid[v[picked]] = 0x80;
                    }
                    continue 'outer;
                }
                // pick random possible
                self.grid[i] =
                    possible[self.rng.read_u8() as usize % possible.len()];
            }
            break 'outer;
        }
    }

    fn get_possible_inverse(&mut self, index: usize) -> HashSet<u8> {
        let mut set: HashSet<u8> = HashSet::new();
        let segment = index / 3;
        // first block
        if segment == 0 || segment == 3 || segment == 6 {
            for i in 0..9 {
                let sindex: usize = i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // second block
        else if segment == 1 || segment == 4 || segment == 7 {
            for i in 0..9 {
                let sindex: usize = 3 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // third block
        else if segment == 2 || segment == 5 || segment == 8 {
            for i in 0..9 {
                let sindex: usize = 6 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // fourth block
        else if segment == 9 || segment == 12 || segment == 15 {
            for i in 0..9 {
                let sindex: usize = 27 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // fifth block
        else if segment == 10 || segment == 13 || segment == 16 {
            for i in 0..9 {
                let sindex: usize = 30 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // sixth block
        else if segment == 11 || segment == 14 || segment == 17 {
            for i in 0..9 {
                let sindex: usize = 33 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // seventh block
        else if segment == 18 || segment == 21 || segment == 24 {
            for i in 0..9 {
                let sindex: usize = 54 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // eighth block
        else if segment == 19 || segment == 22 || segment == 25 {
            for i in 0..9 {
                let sindex: usize = 57 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }
        // ninth block
        else if segment == 20 || segment == 23 || segment == 26 {
            for i in 0..9 {
                let sindex: usize = 60 + i + i / 3 * 6;
                if sindex != index && self.grid[sindex] != 0x80 {
                    set.insert(self.grid[sindex]);
                }
            }
        }

        // row
        let segment = index / 9 * 9;
        for i in segment..(segment + 9) {
            if i != index && self.grid[i] != 0x80 {
                set.insert(self.grid[i]);
            }
        }

        // column
        let mut segment = index % 9;
        while segment < 81 {
            if segment != index && self.grid[segment] != 0x80 {
                set.insert(self.grid[segment]);
            }
            segment += 9;
        }

        set
    }

    fn print_grid(&mut self) {
        for i in 0..13 {
            write!(self.stdout, "{}{}               ",
                cursor::Goto(1, i + 1),
                color::Bg(color::AnsiValue(14))).unwrap();
        }
        write!(self.stdout, "{}{}", style::Reset, cursor::Goto(1, 1))
            .unwrap();
        let mut offset = 0;
        for i in 0..81 {
            if i % 27 == 0 && i != 0 {
                write!(self.stdout, "{}|---+---+---|{}",
                    cursor::Goto(2, 2 + i / 9 + offset),
                    cursor::Goto(2, 3 + i / 9 + offset)).unwrap();
                offset += 1;
            }
            else if i % 9 == 0 {
                write!(self.stdout, "{}",
                    cursor::Goto(2, 2 + i / 9 + offset)).unwrap();
            }
            if i % 3 == 0 {
                write!(self.stdout, "|").unwrap();
            }
            if self.show_solution {
                write!(self.stdout, "{}", self.grid[i as usize] & 0xF).unwrap();
            }
            else {
                let color = if self.wrong_input {
                        color::Bg(color::AnsiValue(9))}
                    else {
                        color::Bg(color::AnsiValue(2))};
                if self.grid[i as usize] & 0x80 == 0 {
                    if self.x + self.y * 9 == i as usize {
                        write!(self.stdout, "{}{}{}",
                            color,
                            self.grid[i as usize],
                            style::Reset).unwrap();
                    }
                    else {
                        write!(self.stdout, "{}", self.grid[i as usize])
                            .unwrap();
                    }
                }
                else {
                    if self.x + self.y * 9 == i as usize {
                        write!(self.stdout, "{}?{}",
                            color,
                            style::Reset).unwrap();
                    }
                    else {
                        write!(self.stdout, "{}?{}",
                            color::Bg(color::AnsiValue(13)),
                            style::Reset).unwrap();
                    }
                }
            }
            if i % 9 == 8 {
                write!(self.stdout, "|").unwrap();
            }
        }
    }

    fn obfuscate_grid(&mut self, hidden_count: usize) {
        let mut s: HashSet<usize> = HashSet::new();
        for _i in 0..hidden_count {
            let mut r: usize = self.rng.read_u8() as usize % 81;
            while s.contains(&r) {
                r = self.rng.read_u8() as usize % 81;
            }
            self.grid[r] |= 0x80;
            s.insert(r);
        }
    }

    fn run(&mut self) {
        'main: loop {
            write!(self.stdout, "{}", clear::All).unwrap();
            self.print_grid();
            self.print_usage();

            write!(self.stdout, "{}", cursor::Goto(1, 14)).unwrap();
            if self.wrong_input {
                write!(self.stdout, "wrong input").unwrap();
            }
            else if self.show_solution {
                write!(self.stdout, "Showing solution").unwrap();
            }
            else {
                write!(self.stdout, "Seed = {}", self.seed).unwrap();
            }
            self.stdout.flush().unwrap();

            self.wrong_input = false;

            let mut buf: [u8; 1] = [0; 1];
            self.stdin.read(&mut buf).unwrap();
            match buf[0] {
                b'q' => break 'main,
                b'w' => {
                    if self.y == 0 {
                        self.y = 8;
                    }
                    else {
                        self.y -= 1;
                    }
                },
                b'a' => {
                    if self.x == 0 {
                        self.x = 8;
                    }
                    else {
                        self.x -= 1;
                    }
                }
                b's' => self.y = (self.y + 1) % 9,
                b'd' => self.x = (self.x + 1) % 9,
                b'r' => self.show_solution = !self.show_solution,
                b'1' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 1 {
                        self.grid[self.x + self.y * 9] = 1;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'2' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 2 {
                        self.grid[self.x + self.y * 9] = 2;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'3' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 3 {
                        self.grid[self.x + self.y * 9] = 3;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'4' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 4 {
                        self.grid[self.x + self.y * 9] = 4;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'5' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 5 {
                        self.grid[self.x + self.y * 9] = 5;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'6' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 6 {
                        self.grid[self.x + self.y * 9] = 6;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'7' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 7 {
                        self.grid[self.x + self.y * 9] = 7;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'8' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 8 {
                        self.grid[self.x + self.y * 9] = 8;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                b'9' => if self.grid[self.x + self.y * 9] & 0x80 != 0 {
                    if self.grid[self.x + self.y * 9] & 0xF == 9 {
                        self.grid[self.x + self.y * 9] = 9;
                    }
                    else {
                        self.wrong_input = true;
                    }
                },
                _ => (),
            }
        }
    }

    fn print_usage(&mut self) {
        write!(self.stdout, "{}q - quit", cursor::Goto(17, 2)).unwrap();
        write!(self.stdout, "{}wasd - move cursor",
            cursor::Goto(17, 4)).unwrap();
        write!(self.stdout, "{}1-9 - input number at cursor",
            cursor::Goto(17, 5)).unwrap();
        write!(self.stdout, "{}r - reveal solution",
            cursor::Goto(17, 7)).unwrap();
    }

    fn simple_print_grid(&mut self) {
        for i in 0..81 {
            if i % 27 == 0 {
                write!(self.stdout, "\n\r").unwrap();
            }
            if i % 9 == 0 {
                write!(self.stdout, "\n\r").unwrap();
            }
            if i % 3 == 0 {
                write!(self.stdout, " ").unwrap();
            }
            write!(self.stdout, "{}", self.grid[i] & 0xF).unwrap();
        }
        write!(self.stdout, "\n\r").unwrap();
    }
}

