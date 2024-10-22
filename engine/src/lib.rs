use std::collections::HashMap;

use game::{Board, Move};

pub mod eval;
pub mod search;

#[derive(Debug, Clone)]
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
pub enum TTEntryFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug)]
pub struct TTEntry {
    pub eval: i32,
    pub depth: u8,
    pub flag: TTEntryFlag,
    pub pv: PvNode,
}

pub struct Engine {
    // none if move has not been found yet, otherwise Some()
    pub best_move: Option<Move>,
    pub board: Board,
    pub transposition_table: HashMap<u64, TTEntry>,
    pub repetition_table: Vec<u64>,
    pub nodes_searched: u64,
    pub canceled: bool,
    pub highest_depth: u8,
}

impl Engine {
    pub fn new(board: Board) -> Engine {
        Engine {
            best_move: None,
            board,
            transposition_table: HashMap::new(),
            repetition_table: Vec::new(),
            nodes_searched: 0,
            canceled: false,
            highest_depth: 0,
        }
    }
}
