//! Provides `game::IsPlayer<::OtherAction>` types.

use {Result, Action};
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use rayon::prelude::*;
use reversi::{board, turn, game, Side, ReversiError};
use reversi::board::Coord;
use std::cmp::Ordering;

const RANDOMNESS: f64 = 0.05f64;
const WEAK:		u32 = 100;
const MEDIUM:	u32 = 10000;
const STRONG:	u32 = 1000000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Score {
    Running(f64),
    Ended(i16),
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        if match (*self, *other) {
    		(Score::Running(val1), Score::Running(val2)) => val1 > val2,
        	(Score::Running(val1), Score::Ended(scr2)) => scr2 < 0i16 || (scr2 == 0i16 && val1 > 0f64),
        	(Score::Ended(scr1), Score::Running(val2)) => scr1 > 0i16 || (scr1 == 0i16 && val2 < 0f64),
        	(Score::Ended(scr1), Score::Ended(scr2)) => scr1 > scr2,
    	} {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Eq for Score {}

impl Ord for Score {
    fn cmp(&self, other: &Score) -> Ordering {
        if *self == *other {
            Ordering::Equal
        } else {
            self.partial_cmp(other).expect("Should be ordered")
        }
    }
}

pub enum AiPlayer {
    Weak,
    Medium,
    Strong,
}

impl game::IsPlayer<::OtherAction> for AiPlayer {
    /// Calls `find_best_move` with suitable parameters
    fn make_move(&self, turn: &turn::Turn) -> Result<Action> {
        Ok(game::PlayerAction::Move(try!(match *self {
                                             AiPlayer::Weak => AiPlayer::find_best_move(turn, WEAK),
                                             AiPlayer::Medium => AiPlayer::find_best_move(turn, MEDIUM),
                                             AiPlayer::Strong => AiPlayer::find_best_move(turn, STRONG),
                                         })))
    }
}

impl AiPlayer {
    /// Find best moves among the legal ones.
    /// Each possibility is evaluated by a method depending on the value of `self` and confronted with the others.
    pub fn find_best_move(turn: &turn::Turn, comps: u32) -> Result<board::Coord> {

        // If everything is alright, turn shouldn't be ended
        let side = turn.get_state()
            .ok_or_else(|| ReversiError::EndedGame(*turn))?;

        // Finds all possible legal moves and records their coordinates
        let mut moves: Vec<Coord> = Vec::new();
        for row in 0..board::BOARD_SIZE {
            for col in 0..board::BOARD_SIZE {
                let coord = board::Coord::new(row, col);
                if turn.check_move(coord).is_ok() {
                    moves.push(coord);
                }
            }
        }

        match moves.len() {
            0 => unreachable!("Game is not ended!"), // Game can't be ended
            1 => Ok(moves[0]), // If there is only one possible move, there's no point in evaluating it.
            num_moves => {
                // Each move has to be evaluated in order to find the best one
                let moves_and_scores = moves
                    .par_iter()
                    .map(|&coord| {
                        let mut turn_after_move = *turn;
                        turn_after_move
                            .make_move(coord)
                            .expect("The move was checked, but something went wrong!");
                        let score = AiPlayer::ai_eval(&turn_after_move, comps / num_moves as u32)
                            .expect("Something went wrong with `AiPlayer::ai_eval`!");
                        (coord, score)
                    });
                let best_move_and_score = match side {
                        Side::Dark => moves_and_scores.min_by_key(|&(_, score)| score),
                        Side::Light => moves_and_scores.max_by_key(|&(_, score)| score),
                    }
                    .expect("No best move found!");
                Ok(best_move_and_score.0)
            }
        }
    }

    fn ai_eval(turn: &turn::Turn, comps: u32) -> Result<Score> {
        if turn.get_state().is_none() {
            Ok(Score::Ended(turn.get_score_diff()))
        } else {
            let mut score = try!(AiPlayer::ai_eval_with_leftover(turn, comps)).0;
            // Add some randomness
            let between = Range::new(-RANDOMNESS, RANDOMNESS);
            let mut rng = thread_rng();
            score = match score {
                Score::Running(val) => Score::Running(val * (1.0 + between.ind_sample(&mut rng))),
                _ => score,
            };
            // Done, return
            Ok(score)
        }
    }

    fn ai_eval_with_leftover(turn: &turn::Turn, comps: u32) -> Result<(Score, u32)> {

        // If everything is alright, turn shouldn't be ended
        // assert!(!this_turn.is_endgame());

        // Finds all possible legal moves and records their coordinates
        let mut moves: Vec<Coord>;
        let mut turn = *turn;
        loop {
            moves = Vec::new();
            for row in 0..board::BOARD_SIZE {
                for col in 0..board::BOARD_SIZE {
                    let coord = board::Coord::new(row, col);
                    if turn.check_move(coord).is_ok() {
                        moves.push(coord);
                    }
                }
            }
            match moves.len() {
                0 => unreachable!("Endgame should have been detected earlier: here it's a waste of computations!"),
                1 => {
                    turn.make_move(moves[0])?; //.expect("There is one move and it should be legit");
                    if turn.get_state().is_none() {
                        return Ok((Score::Ended(turn.get_score_diff()), comps));
                    }
                }
                _num_moves => {
                    break;
                }
            }
        }

        let mut scores: Vec<Score> = Vec::new();
        let mut leftover = comps.checked_sub(moves.len() as u32).unwrap_or(0);

        while let Some(coord) = moves.pop() {
            let mut turn_after_move = turn;
            turn_after_move.make_move(coord)?;
            let turns_left = (moves.len() + 1) as u32;
            scores.push(match turn_after_move.get_state() {
                            None => Score::Ended(turn_after_move.get_score_diff()),
                            Some(_) if leftover < turns_left => Score::Running(try!(AiPlayer::heavy_eval(&turn_after_move))),
                            _ => {
                                let new_comps = leftover / turns_left; // since leftover >= turns_left, then new_comps >= 1
                                let new_score_leftover = try!(AiPlayer::ai_eval_with_leftover(&turn_after_move, new_comps));
                                leftover += new_score_leftover.1;
                                leftover -= new_comps; // since leftover >= turns_left, leftover - newcomps >= 0
                                new_score_leftover.0
                            }
                        });
        }

        Ok((match turn.get_state() {
                Some(Side::Dark) => scores.into_iter().min().expect("Why should this fail?"),
                Some(Side::Light) => scores.into_iter().max().expect("Why should this fail?"),
                None => unreachable!("turn is ended but it should not be"),
            },
            leftover))
    }

    fn heavy_eval(turn: &turn::Turn) -> Result<f64> {
        // Weights
        const CORNER_BONUS: u16 = 50;
        const ODD_CORNER_MALUS: u16 = 20;
        const EVEN_CORNER_BONUS: u16 = 10;
        const ODD_MALUS: u16 = 7; // x2
        const EVEN_BONUS: u16 = 3; // x2
        // ------------------------ Sum = 100

        let sides: [(Coord, Coord, Coord, Coord, Coord, Coord, Coord); 4] = [(/* NW corner */
                                                                              Coord::new(0, 0),
                                                                              Coord::new(0, 1),
                                                                              Coord::new(1, 1),
                                                                              Coord::new(0, 2),
                                                                              Coord::new(2, 2),
                                                                              Coord::new(1, 0),
                                                                              Coord::new(2, 0)),
                                                                             (/* NE corner */
                                                                              Coord::new(0, 7),
                                                                              Coord::new(1, 7),
                                                                              Coord::new(1, 6),
                                                                              Coord::new(2, 7),
                                                                              Coord::new(2, 5),
                                                                              Coord::new(0, 6),
                                                                              Coord::new(0, 5)),
                                                                             (/* SW corner */
                                                                              Coord::new(7, 0),
                                                                              Coord::new(6, 0),
                                                                              Coord::new(6, 1),
                                                                              Coord::new(5, 0),
                                                                              Coord::new(5, 2),
                                                                              Coord::new(7, 1),
                                                                              Coord::new(7, 2)),
                                                                             (/* SE corner */
                                                                              Coord::new(7, 7),
                                                                              Coord::new(6, 7),
                                                                              Coord::new(6, 6),
                                                                              Coord::new(5, 7),
                                                                              Coord::new(5, 5),
                                                                              Coord::new(7, 6),
                                                                              Coord::new(7, 5))];

        let mut score_light: u16 = 0;
        let mut score_dark: u16 = 0;

        for &(corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) in &sides {

            if let Some(disk) = *turn.get_cell(corner)? {
                match disk.get_side() {
                    Side::Light => score_light += CORNER_BONUS,
                    Side::Dark => score_dark += CORNER_BONUS,
                }
            } else {
                for &(coord_odd, coord_even) in &[(odd, even), (counter_odd, counter_even)] {
                    if let Some(disk) = *turn.get_cell(coord_odd)? {
                        match disk.get_side() {
                            Side::Light => score_dark += ODD_MALUS,
                            Side::Dark => score_light += ODD_MALUS,
                        }
                    } else if let Some(disk) = *turn.get_cell(coord_even)? {
                        match disk.get_side() {
                            Side::Light => score_light += EVEN_BONUS,
                            Side::Dark => score_dark += EVEN_BONUS,
                        }
                    }
                }
                if let Some(disk) = *turn.get_cell(odd_corner)? {
                    match disk.get_side() {
                        Side::Light => score_dark += ODD_CORNER_MALUS,
                        Side::Dark => score_light += ODD_CORNER_MALUS,
                    }

                } else if let Some(disk) = *turn.get_cell(even_corner)? {
                    match disk.get_side() {
                        Side::Light => score_light += EVEN_CORNER_BONUS,
                        Side::Dark => score_dark += EVEN_CORNER_BONUS,
                    }
                }
            }
        }
        Ok(score_light as f64 - score_dark as f64)
    }
}
