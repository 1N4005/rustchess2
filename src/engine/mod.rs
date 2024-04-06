use std::collections::HashMap;

use rustchess2::game::{Board, Move};

pub mod eval;
pub mod search;

#[derive(Debug)]
pub enum TTEntryFlag {
    Exact,
    LowerBound,
    UpperBound,
}

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

#[derive(Debug)]
pub struct TTEntry {
    pub eval: i32,
    pub depth: u8,
    pub flag: TTEntryFlag,
    pub best_move: Option<Move>,
}

pub struct Engine {
    // none if move has not been found yet, otherwise Some()
    pub best_move: Option<Move>,
    pub board: Board,
    pub transposition_table: HashMap<u64, TTEntry>,
    pub nodes_searched: u64,
}

impl Engine {
    pub fn new(board: Board) -> Engine {
        Engine {
            best_move: None,
            board,
            transposition_table: HashMap::new(),
            nodes_searched: 0,
        }
    }
}
