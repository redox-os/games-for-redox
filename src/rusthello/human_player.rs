//! Provides `game::IsPlayer<::OtherAction>` types.

use interface;
use reversi::{turn, game};
use ::{Action, Result};

/// The type of human players. Its `make_move` calls the interface to ask user for an input.
pub struct HumanPlayer;

impl game::IsPlayer<::OtherAction> for HumanPlayer {
    /// Calls the interface to ask user for an input.
    fn make_move(&self, turn: &turn::Turn) -> Result<Action> {
        interface::human_make_move(turn)
    }
}
