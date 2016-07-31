#![cfg_attr(feature = "nightly", feature(io))]

extern crate clap;
extern crate libgo;
extern crate rustyline;
extern crate termion;

use clap::{App, Arg};
use std::io::{self, ErrorKind, Write, stdout};
use std::net::SocketAddr;
use std::str::FromStr;

use libgo::game::Game;
use libgo::game::board::Board;
use libgo::gtp::{self, AGENT_VERSION, gtp_connect};
use libgo::gtp::command::Command;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use termion::clear;
use termion::color::{self, AnsiValue};
use termion::cursor::Goto;
use termion::raw::{IntoRawMode, RawTerminal};

const PROMPT_HISTORY_FILE: &'static str = ".history.txt";

fn main() {
    let matches = App::new("Play Go")
        .about("\r\nA Go Text Protocol (GTP) engine with extensions.")
        .version(AGENT_VERSION)
        .arg(Arg::with_name("gtp-connect")
            .long("gtp-connect")
            .value_name("host:port")
            .takes_value(true)
            .help("Connects to host and port to receive GTP commands"))
        .arg(Arg::with_name("gtp-mode")
            .long("gtp-mode")
            .help("Runs in GTP mode using stdin and stdout for communication"))
        .get_matches();

    if matches.is_present("gtp-mode") {
        gtp::play_go();
    } else if let Some(address) = matches.value_of("gtp-connect") {
        gtp_connect::play_go(SocketAddr::from_str(&address).expect("failed to parse address"));
    } else {
        start_interactive_mode();
    }
}

fn reset_screen(stdout: &mut RawTerminal<io::StdoutLock>) {
    write!(stdout, "{}{}", clear::All, Goto(1, 1)).expect("reset_screen: failed write");
    stdout.flush().expect("reset_screen: failed to flush stdout");
}

fn read_from_prompt(prompt: &mut Editor, prompt_text: &str) -> Option<String> {
    let line_result = prompt.readline(prompt_text);

    // Hack to convert the prompt's EOL to "\n\r".
    print!("\r");

    match line_result {
        Ok(line) => {
            prompt.add_history_entry(&line);
            Some(line)
        },
        Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
            None
        },
        Err(err) => {
            panic!(err);
        }
    }
}

fn load_prompt_history(prompt: &mut Editor) {
    if let Err(error) = prompt.load_history(PROMPT_HISTORY_FILE) {
        if let ReadlineError::Io(ref error) = error {
            println!("{:?}", error.kind());
            if error.kind() == ErrorKind::NotFound {
                return;
            }
        }
        panic!(error);
    }
}

/// Run the engine in interactive mode.
pub fn start_interactive_mode() {
    let command_map = gtp::register_commands();
    let mut game = Game::new();
    let mut result_buffer = "\r\n Enter 'list_commands' for a full list of options.".to_owned();
    let prompt_text = "GTP> ".to_owned();

    let mut prompt = Editor::new();
    load_prompt_history(&mut prompt);

    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();

    loop {
        reset_screen(&mut stdout);
        draw_board(game.board());
        print!("\r\n{}", result_buffer);

        write!(stdout, "{}", Goto(1, (game.board().size() + 3) as u16)).expect("goto failed");

        if let Some(line) = read_from_prompt(&mut prompt, &prompt_text) {
            if let Some(command) = Command::from_line(&line) {
                prompt.save_history(PROMPT_HISTORY_FILE).unwrap();

                let result = gtp::gtp_exec(&mut game, &command, &command_map);
                result_buffer = gtp::command_result::display(command.id, result);

                if command.name == "quit" {
                    break;
                }
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
