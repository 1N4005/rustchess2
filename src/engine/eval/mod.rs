use rustchess2::{
    game::{BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE},
    get_piece_color, get_piece_type,
};

use self::piecetables::{
    BLACK_BISHOP_TABLE, BLACK_ENDGAME_KING_TABLE, BLACK_KNIGHT_TABLE, BLACK_MIDGAME_KING_TABLE,
    BLACK_PAWN_TABLE, BLACK_QUEEN_TABLE, BLACK_ROOK_TABLE, WHITE_BISHOP_TABLE,
    WHITE_ENDGAME_KING_TABLE, WHITE_KNIGHT_TABLE, WHITE_MIDGAME_KING_TABLE, WHITE_PAWN_TABLE,
    WHITE_QUEEN_TABLE, WHITE_ROOK_TABLE,
};

// probably doesnt need to be in its own file but whatever
use super::Engine;

mod piecetables;

const PAWN_VALUE: i32 = 100;
const BISHOP_VALUE: i32 = 320;
const KNIGHT_VALUE: i32 = 300;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000;

// used to determine if in the endgame
const ENDGAME_PIECE_COUNT: u32 = 15;

//evaluated from perspective of side to move
impl Engine {
    pub fn evaluate(&self) -> i32 {
        let mut eval = 0;

        let mut r = 0;
        let mut f = 0;

        //used to determine which piece table to use
        let total_piece_count =
            self.board.white.all.count_ones() + self.board.black.all.count_ones();

        for rank in self.board.board {
            for piece in rank {
                if get_piece_color!(piece) == WHITE {
                    eval += match get_piece_type!(piece) {
                        PAWN => PAWN_VALUE + WHITE_PAWN_TABLE[r][f],
                        BISHOP => BISHOP_VALUE + WHITE_BISHOP_TABLE[r][f],
                        KNIGHT => KNIGHT_VALUE + WHITE_KNIGHT_TABLE[r][f],
                        ROOK => ROOK_VALUE + WHITE_ROOK_TABLE[r][f],
                        QUEEN => QUEEN_VALUE + WHITE_QUEEN_TABLE[r][f],
                        KING => {
                            KING_VALUE
                                + if total_piece_count > ENDGAME_PIECE_COUNT {
                                    WHITE_MIDGAME_KING_TABLE[r][f]
                                } else {
                                    WHITE_ENDGAME_KING_TABLE[r][f]
                                }
                        }
                        _ => 0,
                    };
                } else if get_piece_color!(piece) == BLACK {
                    eval -= match get_piece_type!(piece) {
                        PAWN => PAWN_VALUE + BLACK_PAWN_TABLE[r][f],
                        BISHOP => BISHOP_VALUE + BLACK_BISHOP_TABLE[r][f],
                        KNIGHT => KNIGHT_VALUE + BLACK_KNIGHT_TABLE[r][f],
                        ROOK => ROOK_VALUE + BLACK_ROOK_TABLE[r][f],
                        QUEEN => QUEEN_VALUE + BLACK_QUEEN_TABLE[r][f],
                        KING => {
                            KING_VALUE
                                + if total_piece_count > ENDGAME_PIECE_COUNT {
                                    BLACK_MIDGAME_KING_TABLE[r][f]
                                } else {
                                    BLACK_ENDGAME_KING_TABLE[r][f]
                                }
                        }
                        _ => 0,
                    };
                }

                f += 1;
            }

            r += 1;
            f = 0;
        }

        eval * if self.board.turn { 1 } else { -1 }
    }

    pub fn get_piece_value(piece_type: u8) -> i32 {
        match piece_type {
            PAWN => PAWN_VALUE,
            BISHOP => BISHOP_VALUE,
            KNIGHT => KNIGHT_VALUE,
            ROOK => ROOK_VALUE,
            QUEEN => QUEEN_VALUE,
            KING => 10_000,
            _ => panic!("lihgoiwgeehiwg"),
        }
    }
}
