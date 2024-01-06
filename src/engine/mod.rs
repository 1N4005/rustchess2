use rustchess2::game::{Board, Move};

pub mod eval;
pub mod search;

pub struct PvNode {
    pub next: Option<Box<PvNode>>,
    pub best_move: Option<Move>,
    pub eval: i32,
}

impl PvNode {
    pub fn new(m: Option<Move>) -> PvNode {
        PvNode {
            next: None,
            best_move: m,
            eval: 0,
        }
    }
}

pub struct Engine {
    // none if move has not been found yet, otherwise Some()
    pub best_move: Option<Move>,
    pub board: Board,
}

impl Engine {
    pub fn new(board: Board) -> Engine {
        Engine {
            best_move: None,
            board: board,
        }
    }
}
