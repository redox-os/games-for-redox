extern crate termion;

use termion::{clear, color, cursor};
use std::env::args;
use std::io::{self, Write};
use std::{time, thread};

static MAN_PAGE: &'static str = /* @MANSTART{cur} */ r#"
NAME
    commie - text hammer and sickle animation loop.

SYNOPSIS
    commie [-h | --help]

DESCRIPTION
    Hammer and sickle ASCII art style animation.

OPTIONS
    (none)
        Run the program.
    -h
    --help
        Print this manual page.

AUTHOR
    This program was written by Ticki for Redox OS. Bugs, issues, or feature requests
    should be reported in the Gitlab repository, 'redox-os/games'.

COPYRIGHT
    Copyright (c) 2016 Ticki

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
unknown argument, try `commie --help`
"#;

const COMMUNISM: &'static str = r#"
              !#########       #
            !########!          ##!
         !########!               ###
      !##########                  ####
    ######### #####                ######
     !###!      !####!              ######
       !           #####            ######!
                     !####!         #######
                        #####       #######
                          !####!   #######!
                             ####!########
          ##                   ##########
        ,######!          !#############
      ,#### ########################!####!
    ,####'     ##################!'    #####
  ,####'            #######              !####!
 ####'                                      #####
 ~##                                          ##~
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

    let mut state = 0;

    println!("\n{}{}{}{}{}{}", cursor::Hide, clear::All, cursor::Goto(1, 1), color::Fg(color::Black), color::Fg(color::Red), COMMUNISM);
    loop {
        println!("{}{}           ☭ GAY ☭ SPACE ☭ COMMUNISM ☭           ", cursor::Goto(1, 1), color::Fg(color::AnsiValue(state)));
        println!("{}{}             WILL PREVAIL, COMRADES!             ", cursor::Goto(1, 20), color::Fg(color::AnsiValue(state)));

        state += 1;
        state %= 8;

        thread::sleep(time::Duration::from_millis(90));
    }
}
