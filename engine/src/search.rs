use game::{get_piece_type, Move, BISHOP, BLACK, KNIGHT, PAWN, QUEEN, ROOK, WHITE};
use std::{
    cmp::{max, min}, sync::mpsc::{Receiver, TryRecvError}, time::{Duration, Instant}
};

use crate::eval::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};

use super::{
    Engine, PvNode, TTEntry,
    TTEntryFlag::{Exact, LowerBound, UpperBound},
};

pub const MIN: i32 = -100_000_000;
pub const MAX: i32 = 100_000_000;

pub const CHECKMATE: i32 = -10_000_000;

impl Engine {
    pub fn iterative_deepening_search(
        &mut self,
        depth: u8,
        time_limit: bool,
        start_time: Instant,
        alloted_time: Duration,
        rx: Option<Receiver<bool>>,
    ) -> Option<Move> {
        let mut search_depth = 1;
        let mut alpha = MIN;
        let mut beta = MAX;
        let mut prev_eval = None;
        let window = 25;

        while search_depth <= depth {
            let start = Instant::now();
            self.nodes_searched = 0;
            self.highest_depth = 0;

            if let Some(eval) = prev_eval {
                alpha = eval - window;
                beta = eval + window;
            }

            let mut result = self.negamax(
                search_depth,
                0,
                alpha,
                beta,
                PvNode::new(None),
                time_limit,
                start_time,
                alloted_time,
                &rx,
            );
            let end = Instant::now();
            let mut dur = end - start;
            let mut eval = result.0;

            let mut delta = 20;
            loop {
                if eval <= alpha {
                    alpha -= delta;
                } else if eval >= beta {
                    beta += delta;
                } else {
                    break;
                }

                result = self.negamax(
                    search_depth,
                    0,
                    alpha,
                    beta,
                    PvNode::new(None),
                    time_limit,
                    start_time,
                    alloted_time,
                    &rx,
                );
                let end = Instant::now();
                dur = end - start;
                eval = result.0;

                delta += delta / 3
            }

            prev_eval = Some(eval);

            if eval > 1000000 {
                print!(
                    "info depth {} seldepth {} score mate {} time {} nodes {} nps {} pv",
                    search_depth,
                    self.highest_depth,
                    -CHECKMATE - eval,
                    dur.as_millis(),
                    self.nodes_searched,
                    (1_000_000.0 * self.nodes_searched as f64 / dur.as_micros() as f64) as u64
                );

                while let Some(pvn) = result.1.next {
                    if let Some(m) = pvn.best_move {
                        print!(" {}", m);
                    } else {
                        break;
                    }

                    result.1 = (*pvn).clone();
                }
                println!();
                return self.best_move;
            }

            print!(
                "info depth {} seldepth {} score cp {} time {} nodes {} nps {} pv",
                search_depth,
                self.highest_depth,
                eval,
                dur.as_millis(),
                self.nodes_searched,
                (1_000_000.0 * self.nodes_searched as f64 / dur.as_micros() as f64) as u64
            );

            while let Some(pvn) = result.1.next {
                    if let Some(m) = pvn.best_move {
                        print!(" {}", m);
                    } else {
                        break;
                    }

                    result.1 = (*pvn).clone();
                }
                println!();

            if self.canceled {
                return self.best_move;
            }

            if let Some(ref rcv) = rx {
                match rcv.try_recv() {
                    Ok(canceled) => {
                        if canceled {
                            self.canceled = true;
                            return self.best_move;
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("disconnected"),
                }
            }

            if time_limit && Instant::now() - start_time > alloted_time {
                return self.best_move;
            }

            search_depth += 1;
        }

        self.best_move
    }

    pub fn negamax(
        &mut self,
        depth: u8,
        depth_from_root: u8,
        mut alpha: i32,
        mut beta: i32,
        mut pv: PvNode,
        time_limit: bool,
        start_time: Instant,
        alloted_time: Duration,
        rx: &Option<Receiver<bool>>,
    ) -> (i32, PvNode) {
        self.nodes_searched += 1;

        // seldepth (this isnt quite correct but this method is easy)
        self.highest_depth = max(self.highest_depth, depth_from_root);

        // draw by repetition
        if depth_from_root > 0
            && !self.repetition_table.is_empty()
            && self.repetition_table[0..self.repetition_table.len() - 1].contains(&self.board.hash)
        {
            return (0, pv);
        }

        if time_limit && Instant::now() - start_time > alloted_time {
            return (0, pv);
        }

        if self.canceled {
            return (0, pv);
        }

        if let Some(rcv) = rx {
            match rcv.try_recv() {
                Ok(canceled) => {
                    if canceled {
                        self.canceled = true;
                        return (0, pv);
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => panic!("disconnected"),
            }
        }

        if depth_from_root > 0 {
            alpha = max(alpha, CHECKMATE + depth_from_root as i32);
            beta = min(beta, -CHECKMATE - depth_from_root as i32);
            if alpha >= beta {
                return (alpha, pv);
            }
        }

        let original_alpha = alpha;

        let mut hash_move = None;
        if let Some(entry) = self.transposition_table.get(&self.board.hash) {
            if entry.depth >= depth {
                match entry.flag {
                    Exact => {
                        return (entry.eval, pv);
                    }
                    LowerBound => alpha = max(alpha, entry.eval),
                    UpperBound => beta = min(beta, entry.eval),
                }

                if alpha >= beta {
                    return (entry.eval, pv);
                }

                hash_move = entry.pv.best_move;
            }

            if depth_from_root == 0 {
                hash_move = entry.pv.best_move;
            }
        }

        if depth == 0 {
            return (self.quiet_search(alpha, beta, depth_from_root + 1), pv);
        }

        let in_check = if self.board.turn {
            movegen::is_in_check(&self.board, WHITE, self.board.white_king_position)
        } else {
            movegen::is_in_check(&self.board, BLACK, self.board.black_king_position)
        };

        let mut moves = movegen::generate_legal_moves(&mut self.board, false);

        if moves.is_empty() {
            if in_check {
                return (CHECKMATE + depth_from_root as i32, pv);
            }

            return (0, pv);
        }

        // reverse futility pruning
        if !in_check && depth <= 8 && self.evaluate() >= beta + 120 * depth as i32 {
            return (beta, pv);
        }

        self.order_moves(&mut moves, hash_move);

        let mut value = MIN;
        for (pos, m) in moves.into_iter().enumerate() {
            let undo = self.board.make_move(m);
            self.repetition_table.push(self.board.hash);

            //determine search extensions
            let mut extensions = 0;
            if if self.board.turn {
                //check extension (if move is a check, extend search depth by 1)
                movegen::is_in_check(&self.board, WHITE, self.board.white_king_position)
            } else {
                movegen::is_in_check(&self.board, BLACK, self.board.black_king_position)
            } {
                extensions += 1;
            }

            let mut eval = (MIN, PvNode::new(None));
            let mut full_search = true;

            // late move reductions
            if depth > 2 && extensions == 0 && m.capture_piece.is_none() && pos > 2 {
                let reduction = if pos > 5 { depth / 3 } else { 1 };

                eval = self.negamax(
                    depth - 1 - reduction,
                    depth_from_root + 1,
                    -beta,
                    -alpha,
                    PvNode::new(Some(m)),
                    time_limit,
                    start_time,
                    alloted_time,
                    rx,
                );

                full_search = eval.0 > alpha;
            }

            if full_search {
                eval = self.negamax(
                    depth - 1 + extensions,
                    depth_from_root + 1,
                    -beta,
                    -alpha,
                    PvNode::new(Some(m)),
                    time_limit,
                    start_time,
                    alloted_time,
                    rx,
                );
            }
            self.repetition_table.pop();
            undo(&mut self.board);
            value = max(value, -eval.0);

            if time_limit && Instant::now() - start_time > alloted_time {
                return (value, pv);
            }

            if self.canceled {
                return (value, pv);
            }

            if let Some(rcv) = rx {
                match rcv.try_recv() {
                    Ok(canceled) => {
                        if canceled {
                            self.canceled = true;
                            return (value, pv);
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("disconnected"),
                }
            }

            if value > alpha {
                alpha = value;
                pv.next = Some(Box::new(eval.1));

                if depth_from_root == 0 {
                    self.best_move = Some(m);
                }
            }

            if alpha >= beta {
                break;
            }
        }

        self.transposition_table.insert(
            self.board.hash,
            TTEntry {
                eval: value,
                depth,
                flag: if value <= original_alpha {
                    UpperBound
                } else if value >= beta {
                    LowerBound
                } else {
                    Exact
                },
                pv: pv.clone(),
            },
        );

        (value, pv)
    }

    pub fn quiet_search(&mut self, mut alpha: i32, beta: i32, _depth_from_root: u8) -> i32 {
        let eval = self.evaluate();
        if eval >= beta {
            return eval;
        }

        alpha = max(alpha, eval);

        let mut captures = movegen::generate_legal_moves(&mut self.board, true);

        if captures.is_empty() {
            return eval;
        }

        self.order_moves(&mut captures, None);

        for m in &captures {
            // https://www.chessprogramming.org/Delta_Pruning
            let margin = 200;
            let delta = match get_piece_type!(m.capture_piece.unwrap()) {
                PAWN => PAWN_VALUE,
                BISHOP => BISHOP_VALUE,
                KNIGHT => KNIGHT_VALUE,
                ROOK => ROOK_VALUE,
                QUEEN => QUEEN_VALUE,
                _ => {
                    panic!("{}\nwrong piece type!\n{}", self.board, m.to_uci());
                }
            };

            if eval + delta + margin < alpha {
                continue;
            }

            let undo = self.board.make_move(*m);

            let eval = -self.quiet_search(-beta, -alpha, _depth_from_root + 1);

            undo(&mut self.board);

            if eval >= beta {
                return eval;
            }

            alpha = max(alpha, eval);
        }

        alpha
    }

    pub fn order_moves(&self, moves: &mut [Move], hash_move: Option<Move>) {
        moves.sort_by_key(|a| {
            // search hash move first
            if let Some(m) = hash_move {
                if m == *a {
                    return -10000000;
                }
            }

            match a.capture_piece {
                Some(piece) => {
                    Engine::get_piece_value(get_piece_type!(a.piece))
                        - Engine::get_piece_value(get_piece_type!(piece))
                    // order is opposite because sort_by_key sorts in ascending order
                }
                None => 0,
            }
        });
    }
}
