use std::{
    cmp::{max, min},
    sync::mpsc::{Receiver, TryRecvError},
    time::{Duration, Instant},
};

use rustchess2::{
    game::{Move, BLACK, WHITE},
    get_piece_type,
};

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

        while search_depth <= depth {
            self.nodes_searched = 0;
            self.highest_depth = 0;
            let start = Instant::now();
            let result = self.negamax(
                search_depth,
                0,
                MIN,
                MAX,
                PvNode::new(None),
                time_limit,
                start_time,
                alloted_time,
                &rx,
            );
            let end = Instant::now();
            let dur = end - start;
            let eval = result.0;

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
                for m in self.find_pv(search_depth - 1) {
                    print!(" {}", m.to_uci());
                }
                println!();
                return Some(self.best_move?);
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
            for m in self.find_pv(search_depth - 1) {
                print!(" {}", m.to_uci());
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
        if self.repetition_table.len() > 0
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
        match self.transposition_table.get(&self.board.hash) {
            Some(entry) => {
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

                    if let Some(_) = entry.best_move {
                        hash_move = entry.best_move;
                    }
                }

                if depth_from_root == 0 {
                    if let Some(_) = entry.best_move {
                        hash_move = entry.best_move;
                    }
                }
            }
            None => {}
        }

        if depth == 0 {
            return (self.quiet_search(alpha, beta, depth_from_root + 1), pv);
        }

        let mut moves = self.board.generate_legal_moves();

        if moves.len() == 0 {
            if self.board.turn {
                if self
                    .board
                    .is_in_check(WHITE, self.board.white_king_position)
                {
                    return (CHECKMATE + depth_from_root as i32, pv);
                }
            } else {
                if self
                    .board
                    .is_in_check(BLACK, self.board.black_king_position)
                {
                    return (CHECKMATE + depth_from_root as i32, pv);
                }
            }

            return (0, pv);
        }

        self.order_moves(&mut moves, hash_move);

        // print!("{}", "\t".repeat(depth_from_root as usize));
        // for m in &moves {
        //     print!("{} ", m.to_uci());
        // }
        // println!();

        let mut value = MIN;
        for (pos, m) in moves.into_iter().enumerate() {
            // println!("{}making move: {} in position with hash: {}", "\t".repeat(depth_from_root as usize), m.to_uci(), self.board.hash);
            let undo = self.board.make_move(m);
            self.repetition_table.push(self.board.hash);

            //determine search extensions
            let mut extensions = 0;
            if if self.board.turn {
                //check extension (if move is a check, extend search depth by 1)
                self.board
                    .is_in_check(WHITE, self.board.white_king_position)
            } else {
                self.board
                    .is_in_check(BLACK, self.board.black_king_position)
            } {
                extensions += 1;
            }

            let mut eval = (MIN, PvNode::new(None));
            let mut full_search = true;

            // late move reductions
            if depth > 3 && extensions == 0 && pos > 3 {
                if let None = m.capture_piece {
                    const REDUCTION: u8 = 1;
                    eval = self.negamax(
                        depth - 1 - REDUCTION,
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
            // print!("{}result of move move: {}, value: {} in hash: {}", "\t".repeat(depth_from_root as usize), m.to_uci(), value, self.board.hash);

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

            if value > alpha {
                // print!(" NEW ALPHA: {}", value);
                alpha = value;
                pv.next = Some(Box::new(eval.1));

                if depth_from_root == 0 {
                    self.best_move = Some(m);
                }
            }

            if alpha >= beta {
                // println!(" ALPHA({}) > BETA({}):", alpha, beta);
                break;
            }
            // println!();
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
                best_move: if depth_from_root == 0 {
                    self.best_move
                } else {
                    match pv.next {
                        Some(ref bpv) => (*bpv).best_move,
                        None => None,
                    }
                },
            },
        );

        return (alpha, pv);
    }

    pub fn quiet_search(&mut self, mut alpha: i32, beta: i32, depth_from_root: u8) -> i32 {
        let eval = self.evaluate();
        if eval > beta {
            return beta;
        }
        alpha = max(alpha, eval);

        let mut captures = self.board.generate_legal_captures();

        if captures.len() == 0 {
            return self.evaluate();
        }

        self.order_moves(&mut captures, None);

        for m in captures {
            let undo = self.board.make_move(m);

            let eval = -self.quiet_search(-beta, -alpha, depth_from_root + 1);

            undo(&mut self.board);

            if eval >= beta {
                return beta;
            }

            alpha = max(alpha, eval);
        }

        alpha
    }

    pub fn order_moves(&self, moves: &mut Vec<Move>, hash_move: Option<Move>) {
        moves.sort_by_key(|a| {
            // search hash move first
            if let Some(m) = hash_move {
                if m == *a {
                    return 100000;
                }
            }

            match a.capture_piece {
                Some(piece) => {
                    Engine::get_piece_value(get_piece_type!(piece))
                        - Engine::get_piece_value(get_piece_type!(a.piece))
                }
                None => 0,
            }
        });

        moves.reverse();
    }
}
