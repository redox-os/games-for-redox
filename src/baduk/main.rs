#![cfg_attr(feature = "nightly", feature(io))]

extern crate libgo;
extern crate liner;
extern crate termion;

use std::cmp;
use std::env::args;
use std::io::{self, Write};

use libgo::game::Game;
use libgo::game::board::Board;
use libgo::gtp;
use libgo::gtp::command::Command;
use liner::Context;
use termion::clear;
use termion::color::{self, AnsiValue};
use termion::cursor::Goto;
use termion::raw::{IntoRawMode, RawTerminal};

static MAN_PAGE: &'static str = /* @MANSTART{baduk} */ r#"
NAME
    baduk - text based Baduk game.

SYNOPSIS
    baduk [-h | --help]

DESCRIPTION
    This program is a text based Baduk (Go) game.

    A strategy game where apposing teams place stones at line intersections to surround or control
    the other's movements.

OPTIONS
    (none)
        Run the program.
    -h
    --help
        Print this manual page.

AUTHOR
    This program was written by David Campbell for Redox OS. Bugs, issues, or feature requests
    should be reported in the Gitlab repository, 'redox-os/games'.

COPYRIGHT
    Copyright (c) 2016 David Campbell

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
unknown argument, try `baduk --help`
"#;

fn main() {
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

    start_interactive_mode();
}

fn reset_screen(stdout: &mut RawTerminal<io::StdoutLock>) {
    write!(stdout, "{}{}", clear::All, Goto(1, 1)).expect("reset_screen: failed write");
    stdout.flush().expect("reset_screen: failed to flush stdout");
}

/// Run the engine in interactive mode.
pub fn start_interactive_mode() {
    let command_map = gtp::register_commands();
    let mut game = Game::new();
    let mut result_buffer = "\r\n Enter 'list_commands' for a full list of options.".to_owned();
    let mut prompt = Context::new();

    let stdout = io::stdout();
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

        let line = prompt.read_line("GTP> ", &mut |_event_handler| {}).unwrap();
        if let Some(command) = Command::from_line(&line) {
            prompt.history.push(line.into()).unwrap();

            let result = gtp::gtp_exec(&mut game, &command, &command_map);
            result_buffer = gtp::command_result::display(command.id, result);

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
            },
            'o' => {
                write!(stdout, "{}", color::Fg(AnsiValue::grayscale(23))).unwrap();
                stdout.write("●".as_bytes()).unwrap();
            },
            '\n' => {
                write!(stdout, "{}", color::Bg(color::Reset)).unwrap();
                stdout.write(character.to_string().as_bytes()).unwrap();
                write!(stdout, "{}", color::Bg(AnsiValue::grayscale(11))).unwrap();
            },
            _ => {
                write!(stdout, "{}", color::Fg(AnsiValue::grayscale(23))).unwrap();
                stdout.write(character.to_string().as_bytes()).unwrap();
            }
        }
    }

    write!(stdout, "{}{}", color::Fg(color::Reset), color::Bg(color::Reset)).unwrap();
    stdout.flush().unwrap();
}
