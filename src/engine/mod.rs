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

    // extracts pv from TT
    pub fn find_pv(&self, len: u8) -> Vec<Move> {
        let mut moves_board = self.board.clone();
        let mut pv = Vec::new();

        loop {
            if pv.len() >= len.into() {
                break;
            }

            match self.transposition_table.get(&moves_board.hash) {
                Some(entry) => {
                    if let Some(m) = entry.best_move {
                        pv.push(m);
                        let _ = moves_board.make_move(m);
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        pv
    }
}
