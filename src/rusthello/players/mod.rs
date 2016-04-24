use interface;
use reversi;

use std::thread;
use std::time;



mod ai_medium;

const STARTING_DEPTH: u8 = 2;
const TIME_LIMIT: u32 = 8 * 100000000;
const NUM_CELLS: u8 = ( reversi::BOARD_SIZE * reversi::BOARD_SIZE ) as u8;


#[derive(Clone,Debug)]
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



#[derive(Clone,Debug)]
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

pub fn ai_make_move(game: &reversi::Game, player: &Player) -> (usize, usize) {
    let mut num_moves = 0;
    let mut forced_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
    let mut game_after_move = game.clone();

    // To save computation time, first check whether the move is forced.
    for (row, &rows) in game.get_board().iter().enumerate() {
        for (col, _) in rows.iter().enumerate() {
            if game_after_move.make_move((row, col)) {
                num_moves += 1;
                forced_move = (row, col);
                game_after_move = game.clone();
            }
        }
    }

    match num_moves {
        0 => panic!("No valid move is possible!"),
        1 => forced_move,
        _ => {
            let start_time = time::Instant::now();
            let mut depth = STARTING_DEPTH;
            let mut best_move = (0, 0);

            //while start_time.elapsed() < time::Duration::new(0, TIME_LIMIT / num_moves)
            {
                let end_depth = 2 * (depth - 1);
                if game.get_tempo() + end_depth >= NUM_CELLS {
                    return find_best_move(game, &player, NUM_CELLS - game.get_tempo());
                } else {
                    best_move = find_best_move(game, &player, depth);
                }
                depth += 1;
            }
            best_move
        }
    }
}



pub fn find_best_move(game: &reversi::Game, player: &Player, depth: u8) -> (usize, usize) {

    if let reversi::Status::Running { current_turn } = game.get_status() {

        let ai_eval: fn(&reversi::Game, u8) -> Score = match *player {
			Player::AiMedium => ai_medium::ai_eval,
			Player::Human    => panic!("A human is not an AI!")
		};

        let mut best_move: Option<MoveScore> = None;

        let mut game_after_move = game.clone();

        let mut threadjoins = Vec::new();

        for (row, &rows) in game.get_board().iter().enumerate() {
            for (col, _) in rows.iter().enumerate() {
                if game_after_move.make_move((row, col)) {

                    threadjoins.push(thread::spawn(move || {
                        let new_move = MoveScore {
                            score: ai_eval(&game_after_move, depth),
                            coord: (row, col),
                        };
                        new_move
                    }));

                    game_after_move = game.clone();

                }
            }
        }

        for threadjoin in threadjoins {
            let new_move = threadjoin.join().expect("Could not receive answer");

            if let Some(old_move) = best_move.clone() {
                if MoveScore::is_better_for(new_move.clone(), old_move, current_turn) {
                    best_move = Some(new_move);
                }
            } else {
                best_move = Some(new_move);
            }
        }

        if let Some(some_move) = best_move {
            some_move.coord
        } else {
            panic!("best_eval is None");
        }

    } else {
        panic!{"Game ended, cannot make a move!"};
    }
}
