//! Provides `game::IsPlayer<::OtherAction>` types.

use reversi;
use reversi::{board, turn, game};
use reversi::board::Coord;
use ::{Result, Action};
use std::cmp::Ordering;
use std::thread;
use rand::distributions::{Range, Sample};
use rand::ChaChaRng;

const RANDOMNESS: f64 = 0.05f64;
const WEAK:   u32 = 100;
const MEDIUM: u32 = 1000;
const STRONG: u32 = 10000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Score {
    Running(f64),
    Ended(i16),
}

impl PartialOrd<Score> for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        if *self == *other {
            Some(Ordering::Equal)
        } else if match *self {
            Score::Running(val1) => {
                match *other {
                    Score::Running(val2) => val1 > val2,
                    Score::Ended(scr2) => scr2 < 0i16 || (scr2 == 0i16 && val1 > 0f64),
                }
            }
            Score::Ended(scr1) => {
                match *other {
                    Score::Running(val2) => scr1 > 0i16 || (scr1 == 0i16 && val2 < 0f64),
                    Score::Ended(scr2) => scr1 > scr2,
                }
            }
        } {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

impl Eq for Score { }

impl Ord for Score {
    fn cmp(&self, other: &Score) -> Ordering {
        self.partial_cmp(other).expect("Should be ordered")
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
        let side = try!(turn.get_state().ok_or(reversi::ReversiError::EndedGame));

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
            num_moves @ _ => { // Each move has to be evaluated in order to find the best one
                let mut threadjoins = Vec::new();
                while let Some(coord) = moves.pop() {
                    let turn_after_move = try!(turn.make_move(coord));
                    threadjoins.push(thread::spawn(move || {
                        Ok((coord, try!(AiPlayer::ai_eval(&turn_after_move, comps / num_moves as u32))))
                    }));
                }

                let mut coord_eval_moves: Vec<(board::Coord, Score)> = Vec::new();
                for threadjoin in threadjoins {
                    coord_eval_moves.push(try!(threadjoin.join().expect("Could not receive answer")));
                }

                // Find best move (min or max depending on turn.state). Weird tricks with iterators and tuples
                let score_index_iter = coord_eval_moves.clone().into_iter().enumerate()
                    .map( move |(i, coord_score)| (coord_score.1, i) );
                let best_score_index = match side {
                    reversi::Side::Dark  => score_index_iter.min().expect("Why should this fail?"),
                    reversi::Side::Light => score_index_iter.max().expect("Why should this fail?"),
                };
                Ok(coord_eval_moves[best_score_index.1].0)
            }
        }
    }

    fn ai_eval(turn: &turn::Turn, comps: u32) -> Result<Score> {
        if turn.is_endgame() {
            Ok(Score::Ended(turn.get_score_diff()))
        } else {
            let mut score = try!(AiPlayer::ai_eval_with_leftover(turn, comps)).0;
            // Add some randomness
            let mut between = Range::new(-RANDOMNESS, RANDOMNESS);
            let mut rng = ChaChaRng::new_unseeded();
            score = match score {
                Score::Running(val) => {
                    Score::Running(val * (1.0 + between.sample(&mut rng)))
                }
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
        let mut turn = turn.clone();
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
                    turn = turn.make_move(moves[0]).expect("There is one move and it should be legit");
                    if turn.is_endgame() {
                        return Ok((Score::Ended(turn.get_score_diff()), comps));
                    }
                }
                _ => break,
            }
        }

        // If everything is alright, turn shouldn't be ended
        // assert!(!turn.is_endgame());

        let mut scores: Vec<Score> = Vec::new();
        let mut leftover = comps.checked_sub(moves.len() as u32).unwrap_or(0);

        while let Some(coord) = moves.pop() {
            let turn_after_move = try!(turn.make_move(coord));
            let turns_left = ( moves.len() + 1 ) as u32;
            scores.push(
                match turn_after_move.get_state() {
                    None => Score::Ended(turn_after_move.get_score_diff()),
                    Some(_) if leftover < turns_left => Score::Running(try!(AiPlayer::heavy_eval(&turn_after_move))),
                    _ => {
                        let new_comps = leftover / turns_left; // since leftover >= turns_left, then new_comps >= 1
                        let new_score_leftover = try!(AiPlayer::ai_eval_with_leftover(&turn_after_move, new_comps));
                        leftover += new_score_leftover.1;
                        leftover -= new_comps; // since leftover >= turns_left, leftover - newcomps >= 0
                        new_score_leftover.0
                    }
                }
            );
        }

        Ok((
            match turn.get_state() {
                Some(reversi::Side::Dark)  => scores.into_iter().min().expect("Why should this fail?"),
                Some(reversi::Side::Light) => scores.into_iter().max().expect("Why should this fail?"),
                None => unreachable!("turn is ended but it should not be")
                },
            leftover
        ))
    }

    fn heavy_eval(turn: &turn::Turn) -> Result<f64> {
        // Weights
        const CORNER_BONUS: u16 = 45;
        const ODD_CORNER_MALUS: u16 = 25;
        const EVEN_CORNER_BONUS: u16 = 10;
        const ODD_MALUS: u16 = 6; // x2
        const EVEN_BONUS: u16 = 4; // x2
        // ------------------------ Sum = 100

        // Special cells
        let sides: [(Coord,Coord,Coord,Coord,Coord,Coord,Coord); 4]
            =  [(Coord::new(0, 0), Coord::new(0, 1), Coord::new(1, 1), Coord::new(0, 2), Coord::new(2, 2), Coord::new(1, 0), Coord::new(2, 0)),   /* NW corner */
                (Coord::new(0, 7), Coord::new(1, 7), Coord::new(1, 6), Coord::new(2, 7), Coord::new(2, 5), Coord::new(0, 6), Coord::new(0, 5)),   /* NE corner */
                (Coord::new(7, 0), Coord::new(6, 0), Coord::new(6, 1), Coord::new(5, 0), Coord::new(5, 2), Coord::new(7, 1), Coord::new(7, 2)),   /* SW corner */
                (Coord::new(7, 7), Coord::new(6, 7), Coord::new(6, 6), Coord::new(5, 7), Coord::new(5, 5), Coord::new(7, 6), Coord::new(7, 5))];  /* SE corner */

        let mut score_light: u16 = 1;
        let mut score_dark: u16 = 1;

        for &(corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) in &sides {

            if let Some(disk) = try!(turn.get_cell(corner)) {
                match disk.get_side() {
                    reversi::Side::Light => {
                        score_light += CORNER_BONUS;
                    }
                    reversi::Side::Dark => {
                        score_dark += CORNER_BONUS;
                    }
                }
            } else {

                if let Some(disk) = try!(turn.get_cell(odd)) {
                    match disk.get_side() {
                        reversi::Side::Light => score_dark += ODD_MALUS,
                        reversi::Side::Dark => score_light += ODD_MALUS,
                    }
                } else if let Some(disk) = try!(turn.get_cell(even)) {
                    match disk.get_side() {
                        reversi::Side::Light => score_light += EVEN_BONUS,
                        reversi::Side::Dark => score_dark += EVEN_BONUS,
                    }
                }

                if let Some(disk) = try!(turn.get_cell(counter_odd)) {
                    match disk.get_side() {
                        reversi::Side::Light => score_dark += ODD_MALUS,
                        reversi::Side::Dark => score_light += ODD_MALUS,
                    }
                } else if let Some(disk) = try!(turn.get_cell(counter_even)) {
                    match disk.get_side() {
                        reversi::Side::Light => score_light += EVEN_BONUS,
                        reversi::Side::Dark => score_dark += EVEN_BONUS,
                    }
                }

                if let Some(disk) = try!(turn.get_cell(odd_corner)) {
                    match disk.get_side() {
                        reversi::Side::Light => score_dark += ODD_CORNER_MALUS,
                        reversi::Side::Dark => score_light += ODD_CORNER_MALUS,
                    }

                } else if let Some(disk) = try!(turn.get_cell(even_corner)) {
                    match disk.get_side() {
                        reversi::Side::Light => score_light += EVEN_CORNER_BONUS,
                        reversi::Side::Dark => score_dark += EVEN_CORNER_BONUS,
                    }
                }
            }
        }
        Ok((score_light as f64 - score_dark as f64) / (score_dark + score_light) as f64)
    }
}
