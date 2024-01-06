use std::cmp::max;

use rustchess2::{
    game::{Move, BLACK, WHITE},
    get_piece_type,
};

use super::{Engine, PvNode};

pub const MIN: i32 = -100_000_000;
pub const MAX: i32 = 100_000_000;

pub const CHECKMATE: i32 = -10_000_000;

impl Engine {
    pub fn search(
        &mut self,
        depth: u8,
        depth_from_root: u8,
        mut alpha: i32,
        beta: i32,
        mut pv: PvNode,
    ) -> (i32, PvNode) {
        if depth == 0 {
            return (self.quiet_search(alpha, beta, depth_from_root + 1), pv);
            // return self.evaluate()
        }

        let mut moves = self.board.generate_legal_moves();
        self.order_moves(&mut moves);

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

        for m in moves {
            let undo = self.board.make_move(m);

            let result = self.search(
                depth - 1,
                depth_from_root + 1,
                -beta,
                -alpha,
                PvNode::new(Some(m)),
            );
            let eval = -result.0;

            undo(&mut self.board);

            if eval >= beta {
                return (beta, pv);
            }

            if eval > alpha {
                pv.best_move = Some(m);
                pv.next = Some(Box::new(result.1));
                pv.eval = eval;

                if depth_from_root == 0 {
                    self.best_move = Some(m);
                }

                alpha = eval;
            }
        }

        (alpha, pv)
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

        self.order_moves(&mut captures);

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

    pub fn order_moves(&self, moves: &mut Vec<Move>) {
        moves.sort_by_key(|a| match a.capture_piece {
            Some(piece) => {
                Engine::get_piece_value(get_piece_type!(piece))
                    - Engine::get_piece_value(get_piece_type!(a.piece))
            }
            None => 0,
        });

        moves.reverse();
    }
}
