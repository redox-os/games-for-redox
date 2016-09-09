//! Implementation of Reversi rules to play a turn.

use reversi;
use reversi::board::*;
use ::Result;

/// A turn can be in two states: either running (with a side to play next) or ended.
pub type State = Option<reversi::Side>;

/// A turn is given by a board and by which player has to move next.
/// For convenience we also annotate current scores.
#[derive(Debug, Clone)]
pub struct Turn {
    board: Board,
    state: State,
    score_dark: u8,
    score_light: u8,
}

impl Turn {
    /// Initializing a new first turn: starting positions on the board and Dark is the first to play
    pub fn first_turn() -> Turn {
        let mut board = Board::new(&[[None; BOARD_SIZE]; BOARD_SIZE]);
        board.place_disk(reversi::Side::Dark, Coord::new(3, 4)).expect("This cannot fail");
        board.place_disk(reversi::Side::Dark, Coord::new(4, 3)).expect("This cannot fail");
        board.place_disk(reversi::Side::Light, Coord::new(3, 3)).expect("This cannot fail");
        board.place_disk(reversi::Side::Light, Coord::new(4, 4)).expect("This cannot fail");

        Turn {
            board: board,
            state: Some(reversi::Side::Dark),
            score_dark: 2,
            score_light: 2,
        }
    }

    /// Returns the turn's board
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Returns the board's cell corresponding to the given coordinates.
    pub fn get_cell(&self, coord: Coord) -> Result<Cell> {
        self.board.get_cell(coord)
    }

    /// Returns the turn's status
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Returns whether the turn is an endgame
    pub fn is_endgame(&self) -> bool {
        self.state == None
    }

    /// Returns the current score of the match.
    pub fn get_score(&self) -> (u8, u8) {
        (self.score_dark, self.score_light)
    }

    /// Returns the difference in score between Light and Dark.
    pub fn get_score_diff(&self) -> i16 {
        self.score_light as i16 - self.score_dark as i16
    }

    /// Returns turn's tempo (how many disks there are on the board).
    pub fn get_tempo(&self) -> u8 {
        self.score_light + self.score_dark
    }

    /// Check whether a given move is legal
    pub fn check_move (&self, coord: Coord) -> Result<()> {

        if self.state.is_none() {
            // If the game is ended, no further moves are possible
            Err(reversi::ReversiError::EndedGame)
        } else if try!(self.board.get_cell(coord)).is_some() { // This also checks `coord`
            // If a cell is already taken, it's not possible to move there
            Err(reversi::ReversiError::CellAlreadyTaken(coord))
        } else {
            // If a move leads to eat in at least one direction, then it is legal
            for &dir in &DIRECTIONS {
                if self.check_move_along_direction(coord, dir) {
                    return Ok(());
                }
            }
            // Otherwise, the move is not legal
            Err(reversi::ReversiError::IllegalMove(coord))
        }
    }

    /// Check whether a move leads to eat in a specified direction
    fn check_move_along_direction (&self, coord: Coord, dir: Direction) -> bool {
        let mut next_coord = coord;
        if let Ok(Ok(Some(next_disk))) = next_coord.step(dir).map(|()| self.get_cell(next_coord)) {
            if Some(next_disk.get_side().opposite()) == self.state {
                while let Ok(Ok(Some(successive_disk))) = next_coord.step(dir).map(|()| self.get_cell(next_coord)) {
                    if Some(successive_disk.get_side()) == self.state {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Eats all of the opponent's occupied cells from a specified cell (given by its coordinates) in a specified direction until it finds a cell of the current player.
    fn make_move_along_direction (&mut self, coord: Coord, dir: Direction) -> Result<()> {

        let side = try!(self.state.ok_or(reversi::ReversiError::EndedGame));
        let mut next_coord = coord;
        let _ = try!(next_coord.step(dir).map(|()| self.board.flip_disk(next_coord)));
        let mut eating: u8 = 1;

        while side != try!(try!(next_coord.step(dir).map(|()| self.board.get_disk(next_coord))).map(|disk| disk.get_side())) {
            try!(self.board.flip_disk(next_coord));
            eating += 1;
        }

        match side {
            reversi::Side::Light => {
                self.score_light += eating;
                self.score_dark  -= eating;
            }
            reversi::Side::Dark => {
                self.score_light -= eating;
                self.score_dark  += eating;
            }
        };

        Ok(())
    }

    /// Current player performs a move, after verifying that it is legal.
    /// It returns either the new turn or the error preventing the move to be performed.
    pub fn make_move (&self, coord: Coord) -> Result<Turn> {

        if let Ok(None) = self.board.get_cell(coord) {
            let mut turn_after_move = self.clone();
            let mut legal = false;

            if let Some(turn_side) = self.state {
                for &dir in &DIRECTIONS {
                    if self.check_move_along_direction(coord, dir) {
                        try!(turn_after_move.make_move_along_direction(coord, dir));
                        legal = true;
                    }
                }

                if legal {
                    try!(turn_after_move.board.place_disk(turn_side, coord));
                    match turn_side {
                        reversi::Side::Dark  => turn_after_move.score_dark  += 1,
                        reversi::Side::Light => turn_after_move.score_light += 1,
                    }

                    // If a move is legal, the next player to play has to be determined
                    // If the opposite player can make any move at all, it gets the turn
                    // If not, if the previous player can make any move at all, it gets the turn
                    // If not (that is, if no player can make any move at all) the game is ended
                    if turn_after_move.get_tempo() == NUM_CELLS as u8 {
                        // Quick check to rule out games with filled up boards as ended.
                        turn_after_move.state = None;
                    } else {
                        // Turn passes to the other player.
                        turn_after_move.state = Some(turn_side.opposite());
                        if !turn_after_move.can_move() {
                            // If the other player cannot move, turn passes back to the first player.
                            turn_after_move.state = Some(turn_side);
                            if !turn_after_move.can_move() {
                                // If neither platers can move, game is ended.
                                turn_after_move.state = None;
                            }
                        }
                    }
                    Ok(turn_after_move)
                } else {
                    Err(reversi::ReversiError::IllegalMove(coord))
                }
            } else {
                Err(reversi::ReversiError::EndedGame)
            }
        } else {
            Err(reversi::ReversiError::CellAlreadyTaken(coord))
        }
    }

    /// Returns whether or not next_player can make any move at all.
    /// To be used privately. User should rather look at turn's state.
    fn can_move(&self) -> bool {
        for (row, &row_array) in self.board.get_all_cells().into_iter().enumerate() {
            for (col, &cell) in row_array.into_iter().enumerate() {
                if cell.is_none() {
                    for &dir in &DIRECTIONS {
                        if self.check_move_along_direction(Coord::new(row, col), dir) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

}
