use game::{get_piece_color, get_piece_type};
use game::{BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};

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

struct MaterialInfo {
    material: i32, // total value
    num_pieces: i32,
    num_pawns: i32,
    num_bishops: i32,
    num_knights: i32,
    num_rooks: i32,
    num_queens: i32,
}

/// generates a bitboard containing the ranks ahead of the square, and the files to the left and
/// right
///
/// * `square`: square
/// * `color`: color of the pawn on the given square
///
///
/// example:
/// Square: e4, color: white
/// `pawn_blocker_mask(crate::game::square_from_uci("e4"), crate::game::WHITE);`
/// ```
/// 00011100 8
/// 00011100 7
/// 00011100 6
/// 00011100 5
/// 00000000 4
/// 00000000 3
/// 00000000 2
/// 00000000 1
/// abcdefgh
/// ```
pub fn pawn_blocker_mask(square: (u8, u8), color: u8) -> u64 {
    assert!(color == WHITE || color == BLACK);
    let a_file: u64 = 0x8080808080808080;
    let direction_mask = if color == WHITE {
        u64::MAX << (8 * (8 - square.0))
    } else {
        u64::MAX >> (8 * (square.0 + 1))
    };

    let mask: u64 = direction_mask
        & (a_file >> square.1
            | if square.1 > 0 {
                a_file >> (square.1 - 1)
            } else {
                0
            }
            | if square.1 < 7 {
                a_file >> (square.1 + 1)
            } else {
                0
            });

    mask
}

// evaluated from perspective of side to move
// perspective is evaluated in the main evaluation function,
// so each sub-function evaluates from white's perspective
impl Engine {
    pub fn evaluate(&self) -> i32 {
        let material = self.get_material_value();
        let midgame_eval = self.midgame_eval(material.material);
        let endgame_eval = self.endgame_eval(material.material);

        let phase = self.phase(material);
        let eval = ((midgame_eval * (256 - phase)) + (endgame_eval * phase)) / 256;
        let perspective = if self.board.turn { 1 } else { -1 };

        eval * perspective
    }

    fn midgame_eval(&self, material: i32) -> i32 {
        let eval = material;

        eval
    }

    fn endgame_eval(&self, material: i32) -> i32 {
        let mut eval = material;

        for rank in 0..8 {
            for (file, square) in self.board.board[rank].into_iter().enumerate() {
                if get_piece_type!(square) == PAWN {
                    if get_piece_color!(square) == WHITE {
                        eval += self.endgame_pawn_value_adjustment(
                            (rank as u8, file as u8),
                            get_piece_color!(square),
                        )
                    } else if get_piece_color!(square) == BLACK {
                        eval += self.endgame_pawn_value_adjustment(
                            (rank as u8, file as u8),
                            get_piece_color!(square),
                        )
                    }
                }
            }
        }

        eval
    }

    // chessprogramming.org/TaperedEval
    fn phase(&self, info: MaterialInfo) -> i32 {
        let pawn_phase = 0;
        let knight_phase = 1;
        let bishop_phase = 1;
        let rook_phase = 2;
        let queen_phase = 4;

        let total_phase = 16 * pawn_phase
            + 4 * knight_phase
            + 4 * bishop_phase
            + 4 * rook_phase
            + 2 * queen_phase;

        let mut phase = total_phase;
        phase -= info.num_pawns * pawn_phase;
        phase -= info.num_knights * knight_phase;
        phase -= info.num_bishops * bishop_phase;
        phase -= info.num_rooks * rook_phase;
        phase -= info.num_queens * queen_phase;

        phase = (phase * 256 + total_phase / 2) / total_phase;

        phase
    }

    // adjusts value of pawn for things like doubled pawns, backwards pawn, etc
    // relative to side to move
    fn endgame_pawn_value_adjustment(&self, square: (u8, u8), color: u8) -> i32 {
        let our_blockers = pawn_blocker_mask(square, if self.board.turn { WHITE } else { BLACK });
        let opponent_blockers =
            pawn_blocker_mask(square, if self.board.turn { BLACK } else { WHITE });

        let opponent_pawns = if self.board.turn {
            self.board.black.pawns
        } else {
            self.board.white.pawns
        };
        let our_pawns = if self.board.turn {
            self.board.white.pawns
        } else {
            self.board.black.pawns
        };

        let mut adjustment: i32 = 0;

        // passed pawn (if there are opposing pawns on the file or the adjacent ones)
        if (color == WHITE) == self.board.turn {
            if opponent_pawns & our_blockers == 0 {
                adjustment += 45
                    + 4 * if color == WHITE {
                        7 - square.1 as i32
                    } else if color == BLACK {
                        square.1 as i32
                    } else {
                        panic!("wrong piece color!")
                    };
            }
        } else {
            if our_pawns & opponent_blockers == 0 {
                adjustment -= 45
                    + 4 * if color == WHITE {
                        7 - square.1 as i32
                    } else if color == BLACK {
                        square.1 as i32
                    } else {
                        panic!("wrong piece color!")
                    };
            }
        }

        adjustment
    }

    fn get_material_value(&self) -> MaterialInfo {
        let mut info = MaterialInfo {
            material: 0,
            num_pieces: 0,
            num_pawns: 0,
            num_bishops: 0,
            num_knights: 0,
            num_rooks: 0,
            num_queens: 0,
        };

        let mut r = 0;
        let mut f = 0;

        //used to determine which piece table to use
        let total_piece_count =
            self.board.white.all.count_ones() + self.board.black.all.count_ones();

        for rank in self.board.board {
            for piece in rank {
                if get_piece_color!(piece) == WHITE {
                    info.material += match get_piece_type!(piece) {
                        PAWN => {
                            info.num_pawns += 1;
                            PAWN_VALUE + WHITE_PAWN_TABLE[r][f]
                        }
                        BISHOP => {
                            info.num_bishops += 1;
                            BISHOP_VALUE + WHITE_BISHOP_TABLE[r][f]
                        }
                        KNIGHT => {
                            info.num_knights += 1;
                            KNIGHT_VALUE + WHITE_KNIGHT_TABLE[r][f]
                        }
                        ROOK => {
                            info.num_rooks += 1;
                            ROOK_VALUE + WHITE_ROOK_TABLE[r][f]
                        }
                        QUEEN => {
                            info.num_queens += 1;
                            QUEEN_VALUE + WHITE_QUEEN_TABLE[r][f]
                        }
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
                    info.material -= match get_piece_type!(piece) {
                        PAWN => {
                            info.num_pawns += 1;
                            PAWN_VALUE + BLACK_PAWN_TABLE[r][f]
                        }
                        BISHOP => {
                            info.num_bishops += 1;
                            BISHOP_VALUE + BLACK_BISHOP_TABLE[r][f]
                        }
                        KNIGHT => {
                            info.num_knights += 1;
                            KNIGHT_VALUE + BLACK_KNIGHT_TABLE[r][f]
                        }
                        ROOK => {
                            info.num_rooks += 1;
                            ROOK_VALUE + BLACK_ROOK_TABLE[r][f]
                        }
                        QUEEN => {
                            info.num_queens += 1;
                            QUEEN_VALUE + BLACK_QUEEN_TABLE[r][f]
                        }
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

        info.num_pieces = total_piece_count as i32;

        info
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
