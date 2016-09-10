//! Implementation of a complete Reversi match.

use std::marker::{PhantomData, Sized};
use reversi;
use reversi::board::*;
use reversi::turn::*;
use ::Result;


pub enum PlayerAction<A> {
    Move(Coord),
    Undo,
    Other(A),
}

/// Being able to make moves is the trait characterizing players.
pub trait IsPlayer<A> {
    fn make_move(&self, turn: &Turn) -> Result<PlayerAction<A>>;
}

/// A game is given by a list of past turns (with the successive move), a current turn, and the two players.
pub struct Game<'a, A, D: 'a + ?Sized + IsPlayer<A>, L: 'a + ?Sized + IsPlayer<A>> {
    current_turn: Turn,
    turns_history: Vec<(Turn, Coord)>,
    dark:  &'a D,
    light: &'a L,
    phantom: PhantomData<A>
}

impl<'a, A, D: 'a + ?Sized + IsPlayer<A>, L: 'a + ?Sized + IsPlayer<A>> Game<'a, A, D, L> {

    /// Creates a new game, with first turn already set and empty turns' history.
    /// It requires the two players as input.
    pub fn new(dark: &'a D, light: &'a L) -> Game<'a, A, D, L> where D: IsPlayer<A>, L: IsPlayer<A> {
        Game {
            current_turn: Turn::first_turn(),
            turns_history: vec![],
            dark: dark,
            light: light,
            phantom: PhantomData,
        }
    }

    /// Gets the board of the current turn.
    pub fn get_current_board(&self) -> &Board {
        self.current_turn.get_board()
    }

    /// Gets current turn.
    pub fn get_current_turn(&self) -> &Turn {
        &self.current_turn
    }

    /// Gets current score.
    pub fn get_current_score(&self) -> (u8, u8) {
        self.current_turn.get_score()
    }

    /// Gets the state of the current turn.
    pub fn get_current_state(&self) -> State {
        self.current_turn.get_state()
    }

    /// Returns true if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.get_current_state().is_none()
    }

    /// It has the correct player return an action and applies its effects.
    pub fn play_turn(&mut self) -> Result<PlayerAction<A>> {
        let action = match self.current_turn.get_state() {
            None => return Err(reversi::ReversiError::EndedGame),
            Some(reversi::Side::Dark)  => try!(self.dark.make_move(&self.current_turn)),
            Some(reversi::Side::Light) => try!(self.light.make_move(&self.current_turn)),
        };

        match action {
            // If that move is legal, it is applied and the turns' history is updated.
            PlayerAction::Move(coord) => try!(self.make_move(coord)),
            PlayerAction::Undo => try!(self.undo()),
            _ => {}
        }

        Ok(action)
    }

    /// A move (given by `coord`) is applied. If that move is legal, game's history is updated.
    fn make_move(&mut self, coord: Coord) -> Result<()> {
        let new_turn = try!(self.current_turn.make_move(coord));
        self.turns_history.push((self.current_turn.clone(), coord));
        self.current_turn = new_turn;
        Ok(())
    }

    /// Undo last move(s) till the player asking for undoing can play again.
    fn undo(&mut self) -> Result<()> {
        let backup = self.turns_history.clone();
        match self.get_current_state() {
            None => {
                self.current_turn = try!(self.turns_history.pop().ok_or(reversi::ReversiError::NoUndo)).0;
                let last_side = self.get_current_state().unwrap();
                while let Some((previous_turn, _)) = self.turns_history.pop() {
                    if last_side.opposite() == previous_turn.get_state().unwrap() {
                        self.current_turn = previous_turn;
                        return Ok(());
                    }
                }
                self.turns_history = backup;
                return Err(reversi::ReversiError::NoUndo);
            },
            Some(current_side) => {
                while let Some((previous_turn, _)) = self.turns_history.pop() {
                    if current_side == previous_turn.get_state().unwrap() {
                        self.current_turn = previous_turn;
                        return Ok(());
                    }
                }
                self.turns_history = backup;
                return Err(reversi::ReversiError::NoUndo);
            }
        }
    }
}
