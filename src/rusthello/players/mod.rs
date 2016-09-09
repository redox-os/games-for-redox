use interface;
use reversi;

use std::thread;
use std::time;



mod ai_medium;

const STARTING_DEPTH: u8 = 2;
const TIME_LIMIT: u32 = 8 * 100000000;
const NUM_CELLS: u8 = ( reversi::BOARD_SIZE * reversi::BOARD_SIZE ) as u8;


#[derive(Clone, Debug)]
pub enum Score {
    Running(f32),
    EndGame(i16),
}

impl Score {

    pub fn is_better_for(first: Score, second: Score, side: reversi::Disk) -> bool {
        match side {
            reversi::Disk::Light =>  Score::is_better(first, second),
            reversi::Disk::Dark  => !Score::is_better(first, second),
        }
    }

    pub fn is_better(first: Score, second: Score) -> bool {
        match first {
            Score::Running(val1) => {
                match second {
                    Score::Running(val2) => val1 > val2,
                    Score::EndGame(scr2) => scr2 < 0i16 || ( scr2 == 0i16 && val1 > 0f32 ),
                }
            }
            Score::EndGame(scr1) => {
                match second {
                    Score::Running(val2) => scr1 > 0i16 || ( scr1 == 0i16 && val2 < 0f32 ),
                    Score::EndGame(scr2) => scr1 > scr2,
                }
            }
        }
    }
}



#[derive(Clone, Debug)]
struct MoveScore{
    score: Score,
    coord: (usize, usize),
}

impl MoveScore {
    pub fn is_better_for(first: MoveScore, second: MoveScore, side: reversi::Disk) -> bool {
        match side {
            reversi::Disk::Light =>  Score::is_better(first.score, second.score),
            reversi::Disk::Dark  => !Score::is_better(first.score, second.score),
        }
    }
}




/// It represents the different kind of player who can take part to the game.
#[derive(Clone)]
pub enum Player {
    Human,
    AiMedium,
}


impl Player {

    /// It produces the new move from each kind of Player.
    pub fn make_move(&self, game: &reversi::Game) -> interface::UserCommand {

        if let reversi::Status::Ended = game.get_status() {
            panic!("make_move called on ended game!");
        }

        if let Player::Human = *self {
			interface::human_make_move(game)
		} else {
			let (row, col) = ai_make_move(game, &self.clone());

			interface::print_move(game, (row, col));

			interface::UserCommand::Move(row, col)
        }
    }
}
