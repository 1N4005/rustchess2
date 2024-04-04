use crate::{
    game::{BLACK_KINGSIDE, BLACK_QUEENSIDE, WHITE_KINGSIDE, WHITE_QUEENSIDE},
    get_piece_color, get_piece_type, is_black_kingside, is_black_queenside, is_white_kingside,
    is_white_queenside,
};

use super::{Board, HashKeys, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};

impl HashKeys {
    pub fn new() -> HashKeys {
        let mut keys = HashKeys {
            white_pawn: [[0; 8]; 8],
            white_bishop: [[0; 8]; 8],
            white_knight: [[0; 8]; 8],
            white_rook: [[0; 8]; 8],
            white_queen: [[0; 8]; 8],
            white_king: [[0; 8]; 8],
            black_pawn: [[0; 8]; 8],
            black_bishop: [[0; 8]; 8],
            black_knight: [[0; 8]; 8],
            black_rook: [[0; 8]; 8],
            black_queen: [[0; 8]; 8],
            black_king: [[0; 8]; 8],
            turn_key: 0,
            white_ks: 0,
            white_qs: 0,
            black_ks: 0,
            black_qs: 0,
            en_passant_square_file: [0; 8],
        };

        for rank in 0..8 {
            for file in 0..8 {
                keys.white_pawn[rank][file] = rand::random();
                keys.white_bishop[rank][file] = rand::random();
                keys.white_knight[rank][file] = rand::random();
                keys.white_rook[rank][file] = rand::random();
                keys.white_queen[rank][file] = rand::random();
                keys.white_king[rank][file] = rand::random();
                keys.black_bishop[rank][file] = rand::random();
                keys.black_knight[rank][file] = rand::random();
                keys.black_rook[rank][file] = rand::random();
                keys.black_queen[rank][file] = rand::random();
                keys.black_king[rank][file] = rand::random();
            }

            keys.en_passant_square_file[rank] = rand::random();
        }

        keys.turn_key = rand::random();
        keys.white_ks = rand::random();
        keys.white_qs = rand::random();
        keys.black_ks = rand::random();
        keys.black_qs = rand::random();

        keys
    }

    pub fn generate_hash(&self, board: &mut Board) {
        let mut r: usize = 0;
        let mut f: usize = 0;
        for rank in board.board {
            for piece in rank {
                board.hash ^= match get_piece_type!(piece) {
                    PAWN => {
                        if get_piece_color!(piece) == WHITE {
                            self.white_pawn[r][f]
                        } else {
                            self.black_pawn[r][f]
                        }
                    }
                    BISHOP => {
                        if get_piece_color!(piece) == WHITE {
                            self.white_bishop[r][f]
                        } else {
                            self.black_bishop[r][f]
                        }
                    }
                    KNIGHT => {
                        if get_piece_color!(piece) == WHITE {
                            self.white_knight[r][f]
                        } else {
                            self.black_knight[r][f]
                        }
                    }
                    ROOK => {
                        if get_piece_color!(piece) == WHITE {
                            self.white_rook[r][f]
                        } else {
                            self.black_rook[r][f]
                        }
                    }
                    QUEEN => {
                        if get_piece_color!(piece) == WHITE {
                            self.white_queen[r][f]
                        } else {
                            self.black_queen[r][f]
                        }
                    }
                    KING => {
                        if get_piece_color!(piece) == WHITE {
                            self.white_king[r][f]
                        } else {
                            self.black_king[r][f]
                        }
                    }
                    _ => 0,
                };

                f += 1;
            }
            f = 0;
            r += 1;
        }

        if is_white_kingside!(board.castle_state) {
            board.hash ^= self.white_ks;
        }

        if is_white_queenside!(board.castle_state) {
            board.hash ^= self.white_qs;
        }

        if is_black_kingside!(board.castle_state) {
            board.hash ^= self.black_ks;
        }

        if is_black_queenside!(board.castle_state) {
            board.hash ^= self.black_qs;
        }

        if board.turn {
            board.hash ^= self.turn_key;
        }

        match board.en_passant_square {
            Some(square) => board.hash ^= self.en_passant_square_file[square.1 as usize],
            None => {}
        }
    }
}
