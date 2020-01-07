#![cfg_attr(feature = "nightly", feature(io))]

extern crate libgo;
extern crate liner;
extern crate termion;

use std::cmp;
use std::io::{self, stdout, Write};

use libgo::game::board::Board;
use libgo::game::Game;
use libgo::gtp::engine::Engine;
use libgo::gtp::command::Command;
use liner::Context;
use liner::Prompt;
use termion::clear;
use termion::color::{self, AnsiValue};
use termion::cursor::Goto;
use termion::raw::{IntoRawMode, RawTerminal};

fn main() {
    start_interactive_mode();
}

fn reset_screen(stdout: &mut RawTerminal<io::StdoutLock>) {
    write!(stdout, "{}{}", clear::All, Goto(1, 1)).expect("reset_screen: failed write");
    stdout
        .flush()
        .expect("reset_screen: failed to flush stdout");
}

/// Run the engine in interactive mode.
pub fn start_interactive_mode() {
    let mut engine = Engine::new();
    let mut game = Game::new();
    let mut result_buffer = "\r\n Enter 'list_commands' for a full list of options.".to_owned();
    let mut prompt = Context::new();

    // get list of commands to register with liner
    engine.register_all_commands();
    let commands: Vec<String> = engine
        .exec(&mut game, &Command::from_line("list_commands").unwrap())
        .result
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|s| String::from(s))
        .collect();
    let mut liner_completer = liner::BasicCompleter::new(commands);

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    loop {
        let board_size = game.board().size();
        let below_the_board = board_size as u16 + 3;

        reset_screen(&mut stdout);
        draw_board(game.board());

        let column_offset = 2 * board_size as u16 + 8;
        let mut line_number = 0;
        for line in result_buffer.lines() {
            line_number += 1;
            write!(stdout, "{}{}", Goto(column_offset, line_number), line).expect("failed write");
        }

        let gtp_line = cmp::max(line_number, below_the_board);
        write!(stdout, "{}", Goto(1, gtp_line)).expect("goto failed");

        let line = prompt
            .read_line(Prompt::from("GTP> "), None, &mut liner_completer)
            .unwrap();
        if let Some(command) = Command::from_line(&line) {
            prompt.history.push(line.into()).unwrap();

            let result = engine.exec(&mut game, &command);
            result_buffer = format!("{}", result);

            if command.name == "quit" {
                break;
            }
        }
    }

    // Do clean-up here!
    reset_screen(&mut stdout);
}

/// Writes a colored version of showboard to stdout using termion.
pub fn draw_board(board: &Board) {
    let stdout = io::stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut board = board.to_ascii();
    board.push_str("\r\n");

    write!(stdout, "{}", color::Bg(AnsiValue::grayscale(11))).unwrap();
    for character in board.chars() {
        match character {
            'x' => {
                write!(stdout, "{}", color::Fg(AnsiValue::grayscale(0))).unwrap();
                stdout.write("●".as_bytes()).unwrap();
            }
            'o' => {
                write!(stdout, "{}", color::Fg(AnsiValue::grayscale(23))).unwrap();
                stdout.write("●".as_bytes()).unwrap();
            }
            '\n' => {
                write!(stdout, "{}", color::Bg(color::Reset)).unwrap();
                stdout.write(character.to_string().as_bytes()).unwrap();
                write!(stdout, "{}", color::Bg(AnsiValue::grayscale(11))).unwrap();
            }
            _ => {
                write!(stdout, "{}", color::Fg(AnsiValue::grayscale(23))).unwrap();
                stdout.write(character.to_string().as_bytes()).unwrap();
            }
        }
    }

    write!(
        stdout,
        "{}{}",
        color::Fg(color::Reset),
        color::Bg(color::Reset)
    )
    .unwrap();
    stdout.flush().unwrap();
}
