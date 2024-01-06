use crate::get_piece_type;

// standard algebraic notation
use super::{Board, Move, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};

impl Move {
    // does not modify board, however generate_legal_moves requrires a mutable board
    // this function is probably expensive to call
    // also why is algebraic notation so stupid
    pub fn to_san(&self, board: &mut Board) -> Result<String, String> {
        let mut s = String::new();

        s += match get_piece_type!(self.piece) {
            PAWN => match self.capture_piece {
                Some(_) => ["a", "b", "c", "d", "e", "f", "g", "h"][self.from.1 as usize],
                None => "",
            },
            BISHOP => "B",
            KNIGHT => "N",
            ROOK => "R",
            QUEEN => "Q",
            KING => "K",
            _ => return Err("invalid piece type! :skull:".to_owned()),
        };

        // for moves such as Rexe4 vs Raxe4
        let mut possible_origins: Vec<Move> = Vec::new();

        for m in board.generate_legal_moves() {
            if m.to == self.to && m.piece == self.piece {
                possible_origins.push(m);
            }
        }

        if possible_origins.len() == 2 {
            if possible_origins[0].from.1 != possible_origins[0].from.1 {
                s += ["a", "b", "c", "d", "e", "f", "g", "h"][self.from.1 as usize];
            } else {
                s += ["8", "7", "6", "5", "4", "3", "2", "1"][self.to.0 as usize];
            }
        }

        s += match self.capture_piece {
            Some(_) => "x",
            None => "",
        };

        s += ["a", "b", "c", "d", "e", "f", "g", "h"][self.to.1 as usize];
        s += ["8", "7", "6", "5", "4", "3", "2", "1"][self.to.0 as usize];

        let undo = board.make_move(*self);

        if board.is_in_check(
            if board.turn { WHITE } else { BLACK },
            if board.turn {
                board.white_king_position
            } else {
                board.black_king_position
            },
        ) {
            s += if board.generate_legal_moves().len() == 0 {
                "#"
            } else {
                "+"
            }
        }

        undo(board);

        Ok(s)
    }
}
