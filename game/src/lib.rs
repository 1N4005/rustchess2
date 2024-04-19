pub mod genkeys;
pub mod precomputed;
pub use rand;

use std::fmt::Display;

pub const WHITE_TO_MOVE: bool = true;
pub const BLACK_TO_MOVE: bool = false;

pub const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
// for testing
pub const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

/**
 * right 5 bits are used
 * 0bXXXYY
 * XXX:
 * 001 - pawn
 * 010 - bishop
 * 011 - knight
 * 100 - rook
 * 101 - queen
 * 110 - king
 * YY:
 * 01 - white
 * 10 - black
 *
 * 0b00000 means empty square
 */
pub type Piece = u8;

/**
 * right four bits are used, 1 = can castle, 0 = can't
 * from the left:
 * 1 - white kingside
 * 2 - white queenside
 * 3 - black kingside
 * 4 - black queenside
 */
pub type CastleState = u8;

#[macro_export]
macro_rules! is_white_kingside {
    ($state: expr) => {
        ($state & WHITE_KINGSIDE > 0)
    };
}

#[macro_export]
macro_rules! is_white_queenside {
    ($state: expr) => {
        ($state & WHITE_QUEENSIDE > 0)
    };
}

#[macro_export]
macro_rules! is_black_kingside {
    ($state: expr) => {
        ($state & BLACK_KINGSIDE > 0)
    };
}

#[macro_export]
macro_rules! is_black_queenside {
    ($state: expr) => {
        ($state & BLACK_QUEENSIDE > 0)
    };
}

// 0 is h1, 63 is a8
#[macro_export]
macro_rules! get_bit_index {
    ($rank: expr, $file: expr) => {
        ((7 - $file) + 8 * (7 - $rank))
    };

    ($square: expr) => {
        ((7 - $square.1) + 8 * (7 - $square.0))
    };
}

#[macro_export]
macro_rules! get_piece_type {
    ($piece: expr) => {
        ($piece & 0b11100)
    };
}

#[macro_export]
macro_rules! get_piece_color {
    ($piece: expr) => {
        ($piece & 0b00011)
    };
}

// matches the first 3 bits of the "piece" type
// last 2 bits are 0 so the type can be ANDed with the color
pub const PAWN: u8 = 0b00100;
pub const BISHOP: u8 = 0b01000;
pub const KNIGHT: u8 = 0b01100;
pub const ROOK: u8 = 0b10000;
pub const QUEEN: u8 = 0b10100;
pub const KING: u8 = 0b11000;

// last 2 bits of the "piece" type
pub const WHITE: u8 = 0b01;
pub const BLACK: u8 = 0b10;

// castling rights
pub const WHITE_KINGSIDE: u8 = 0b1000;
pub const WHITE_QUEENSIDE: u8 = 0b0100;
pub const BLACK_KINGSIDE: u8 = 0b0010;
pub const BLACK_QUEENSIDE: u8 = 0b0001;

// masks for moving the rook
const WHITE_KINGSIDE_MASK: u64 = 0b101;
const WHITE_QUEENSIDE_MASK: u64 = 0b10010000;
const BLACK_KINGSIDE_MASK: u64 =
    0b00000101_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BLACK_QUEENSIDE_MASK: u64 =
    0b10010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

// masks for detecting if the king has moved correctly
const WHITE_KINGSIDE_MOVE_MASK: u64 = 0b1010;
const WHITE_QUEENSIDE_MOVE_MASK: u64 = 0b00101000;
const BLACK_KINGSIDE_MOVE_MASK: u64 =
    0b00001010_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BLACK_QUEENSIDE_MOVE_MASK: u64 =
    0b00101000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

//rank file, uci -> "a1", "a2", etc
pub fn square_from_uci(uci: &str) -> (u8, u8) {
    let file: char = uci.chars().nth(0).unwrap();
    let rank: u8 = 8 - uci.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
    (rank, file as u8 - 97)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: (u8, u8),
    pub to: (u8, u8),

    // Piece, not piece type, so color should be included
    pub piece: Piece,

    // if en passant, still set the capture piece
    // Piece, not piece type, so color should be included
    pub capture_piece: Option<Piece>,
    pub promotion_piece: Option<Piece>,
    pub en_passant: bool,
}

impl Move {
    pub fn new(
        from: (u8, u8),
        to: (u8, u8),
        piece: Piece,
        capture_piece: Option<Piece>,
        promotion_piece: Option<Piece>,
        en_passant: bool,
    ) -> Move {
        Move {
            from,
            to,
            piece,
            capture_piece,
            promotion_piece,
            en_passant,
        }
    }

    //requires board so move information can be added
    pub fn from_uci(uci: &str, board: Board) -> Move {
        let from = square_from_uci(&uci[..2]);
        let to = square_from_uci(&uci[2..4]);
        let promotion = uci.chars().nth(4);
        // println!("{:?}, {:?}", from, to);
        Move::new(
            from,
            to,
            board.board[from.0 as usize][from.1 as usize],
            match board.board[to.0 as usize][to.1 as usize] {
                0 => None,
                piece => Some(piece),
            },
            match promotion {
                Some(c) => match c {
                    'n' => Some(
                        get_piece_color!(board.board[from.0 as usize][from.1 as usize]) | KNIGHT,
                    ),
                    'b' => Some(
                        get_piece_color!(board.board[from.0 as usize][from.1 as usize]) | BISHOP,
                    ),
                    'r' => {
                        Some(get_piece_color!(board.board[from.0 as usize][from.1 as usize]) | ROOK)
                    }
                    'q' => Some(
                        get_piece_color!(board.board[from.0 as usize][from.1 as usize]) | QUEEN,
                    ),
                    _ => {
                        panic!("promotion in uci string is invalid! :skull:")
                    }
                },
                None => None,
            },
            match board.en_passant_square {
                Some(square) => square == to,
                None => false,
            },
        )
    }

    pub fn to_uci(&self) -> String {
        let mut s = String::new();

        s += ["a", "b", "c", "d", "e", "f", "g", "h"][self.from.1 as usize];
        s += ["8", "7", "6", "5", "4", "3", "2", "1"][self.from.0 as usize];
        s += ["a", "b", "c", "d", "e", "f", "g", "h"][self.to.1 as usize];
        s += ["8", "7", "6", "5", "4", "3", "2", "1"][self.to.0 as usize];

        s += match self.promotion_piece {
            Some(piece) => match get_piece_type!(piece) {
                KNIGHT => "n",
                BISHOP => "b",
                ROOK => "r",
                QUEEN => "q",
                _ => panic!("invalid promotion type! (again) :skull:"),
            },
            None => "",
        };

        s
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(&self.to_uci());

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bitboards {
    pub pawns: u64,
    pub bishops: u64,
    pub knights: u64,
    pub rooks: u64,
    pub queens: u64,
    pub king: u64,
    pub all: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct HashKeys {
    pub white_pawn: [[u64; 8]; 8],
    pub white_bishop: [[u64; 8]; 8],
    pub white_knight: [[u64; 8]; 8],
    pub white_rook: [[u64; 8]; 8],
    pub white_queen: [[u64; 8]; 8],
    pub white_king: [[u64; 8]; 8],

    pub black_pawn: [[u64; 8]; 8],
    pub black_bishop: [[u64; 8]; 8],
    pub black_knight: [[u64; 8]; 8],
    pub black_rook: [[u64; 8]; 8],
    pub black_queen: [[u64; 8]; 8],
    pub black_king: [[u64; 8]; 8],

    pub turn_key: u64,

    pub white_ks: u64,
    pub white_qs: u64,
    pub black_ks: u64,
    pub black_qs: u64,

    // one key for each file
    pub en_passant_square_file: [u64; 8],
}

#[derive(Clone, Copy, Debug)]
pub struct Board {
    // board[rank][file]
    // rank index: 8th rank -> 0, 1st rank -> 7
    // file index: a file -> 0, h file -> 7
    pub board: [[Piece; 8]; 8],
    //ranks are stored continuously (row-major), ones bit is h1, highest is a8 (to make visualization easier)
    pub white: Bitboards,
    pub black: Bitboards,
    pub turn: bool,
    pub castle_state: CastleState,

    // (rank, file)
    pub en_passant_square: Option<(u8, u8)>,
    //incremented after black's move
    pub fullmoves: u16,
    //precomputed for knight moves
    pub knight_masks: [[u64; 8]; 8],
    // precomputed for king moves
    pub king_masks: [[u64; 8]; 8],
    // precomputed ray attacks (shifted bitboards)
    // ray_attacks[direction][rank][file]
    pub ray_attacks: [[[u64; 8]; 8]; 8],
    // precomputed bishop and rook blocker masks (magic bitboards)
    pub bishop_blocker_masks: [[u64; 8]; 8],
    pub rook_blocker_masks: [[u64; 8]; 8],

    pub white_king_position: (u8, u8),
    pub black_king_position: (u8, u8),

    pub hash_keys: HashKeys,
    pub hash: u64,
}

impl Board {
    fn update_bitboards(&mut self, move_to_make: Move) {
        let color = get_piece_color!(move_to_make.piece);
        let piece_type = get_piece_type!(move_to_make.piece);
        let from_mask = 1 << get_bit_index!(move_to_make.from);
        let to_mask = 1 << get_bit_index!(move_to_make.to);
        let move_mask = from_mask | to_mask;

        // update bitboard
        if color == WHITE {
            self.white.all ^= move_mask;

            match piece_type {
                PAWN => {
                    self.white.pawns ^= from_mask;

                    // promotions require different handling of the "to" square
                    match move_to_make.promotion_piece {
                        Some(piece) => match get_piece_type!(piece) {
                            BISHOP => {
                                self.white.bishops ^= to_mask;
                            }
                            KNIGHT => {
                                self.white.knights ^= to_mask;
                            }
                            ROOK => {
                                self.white.rooks ^= to_mask;
                            }
                            QUEEN => {
                                self.white.queens ^= to_mask;
                            }
                            _ => {
                                panic!("invalid promotion piece type! :skull:")
                            }
                        },
                        None => {
                            self.white.pawns ^= to_mask;
                        }
                    }
                }
                BISHOP => {
                    self.white.bishops ^= move_mask;
                }
                KNIGHT => {
                    self.white.knights ^= move_mask;
                }
                ROOK => {
                    self.white.rooks ^= move_mask;
                }
                QUEEN => {
                    self.white.queens ^= move_mask;
                }
                KING => {
                    self.white.king ^= move_mask;
                    if move_mask == WHITE_KINGSIDE_MOVE_MASK {
                        self.white.rooks ^= WHITE_KINGSIDE_MASK;
                        self.white.all ^= WHITE_KINGSIDE_MASK;
                    } else if move_mask == WHITE_QUEENSIDE_MOVE_MASK {
                        self.white.rooks ^= WHITE_QUEENSIDE_MASK;
                        self.white.all ^= WHITE_QUEENSIDE_MASK;
                    }
                }
                _ => {
                    panic!("invalid piece type :skull:")
                }
            }

            if move_to_make.en_passant {
                self.black.pawns ^= to_mask >> 8;
                self.black.all ^= to_mask >> 8;
            } else {
                match move_to_make.capture_piece {
                    Some(piece) => {
                        self.black.all ^= to_mask;
                        match get_piece_type!(piece) {
                            PAWN => {
                                self.black.pawns ^= to_mask;
                            }
                            BISHOP => {
                                self.black.bishops ^= to_mask;
                            }
                            KNIGHT => {
                                self.black.knights ^= to_mask;
                            }
                            ROOK => {
                                self.black.rooks ^= to_mask;
                            }
                            QUEEN => {
                                self.black.queens ^= to_mask;
                            }
                            _ => {
                                panic!("invalid capture piece type! :skull:")
                            }
                        }
                    }
                    None => {}
                }
            }
        } else if color == BLACK {
            self.black.all ^= move_mask;

            match piece_type {
                PAWN => {
                    self.black.pawns ^= from_mask;

                    // promotions require different handling of the "to" square
                    match move_to_make.promotion_piece {
                        Some(piece) => match get_piece_type!(piece) {
                            BISHOP => {
                                self.black.bishops ^= to_mask;
                            }
                            KNIGHT => {
                                self.black.knights ^= to_mask;
                            }
                            ROOK => {
                                self.black.rooks ^= to_mask;
                            }
                            QUEEN => {
                                self.black.queens ^= to_mask;
                            }
                            _ => {
                                panic!("invalid promotion piece type! :skull:")
                            }
                        },
                        None => {
                            self.black.pawns ^= to_mask;
                        }
                    }
                }
                BISHOP => {
                    self.black.bishops ^= move_mask;
                }
                KNIGHT => {
                    self.black.knights ^= move_mask;
                }
                ROOK => {
                    self.black.rooks ^= move_mask;
                }
                QUEEN => {
                    self.black.queens ^= move_mask;
                }
                KING => {
                    self.black.king ^= move_mask;
                    if move_mask == BLACK_KINGSIDE_MOVE_MASK {
                        self.black.rooks ^= BLACK_KINGSIDE_MASK;
                        self.black.all ^= BLACK_KINGSIDE_MASK;
                    } else if move_mask == BLACK_QUEENSIDE_MOVE_MASK {
                        self.black.rooks ^= BLACK_QUEENSIDE_MASK;
                        self.black.all ^= BLACK_QUEENSIDE_MASK;
                    }
                }
                _ => {
                    panic!("invalid piece type :skull:")
                }
            }

            if move_to_make.en_passant {
                self.white.pawns ^= to_mask << 8;
                self.white.all ^= to_mask << 8;
            } else {
                match move_to_make.capture_piece {
                    Some(piece) => {
                        self.white.all ^= to_mask;
                        match get_piece_type!(piece) {
                            PAWN => {
                                self.white.pawns ^= to_mask;
                            }
                            BISHOP => {
                                self.white.bishops ^= to_mask;
                            }
                            KNIGHT => {
                                self.white.knights ^= to_mask;
                            }
                            ROOK => {
                                self.white.rooks ^= to_mask;
                            }
                            QUEEN => {
                                self.white.queens ^= to_mask;
                            }
                            _ => {
                                println!("{:08b}", piece);
                                panic!("invalid capture piece type! :skull:")
                            }
                        }
                    }
                    None => {}
                }
            }
        }
    }

    // assumes move is legal, does not check
    pub fn make_move(&mut self, move_to_make: Move) -> impl Fn(&mut Board) {
        // variables used to undo a move if needed
        let prev_board = self.board;
        let prev_white = self.white;
        let prev_black = self.black;
        let prev_turn = self.turn; // this one probably isn't necessary but whatever
        let prev_castle_state = self.castle_state;
        let prev_en_passant_square = self.en_passant_square;
        let prev_fullmoves = self.fullmoves;
        let prev_white_king_position = self.white_king_position;
        let prev_black_king_position = self.black_king_position;
        let prev_hash = self.hash;

        // check that there is a color, check that color matches turn
        assert!(
            get_piece_color!(move_to_make.piece) == WHITE
                || get_piece_color!(move_to_make.piece) == BLACK
        );
        assert_eq!((get_piece_color!(move_to_make.piece) == WHITE), self.turn);

        self.hash ^= match get_piece_type!(move_to_make.piece) {
            PAWN => {
                if self.turn {
                    self.hash_keys.white_pawn[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                } else {
                    self.hash_keys.black_pawn[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                }
            }
            BISHOP => {
                if self.turn {
                    self.hash_keys.white_bishop[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                } else {
                    self.hash_keys.black_bishop[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                }
            }
            KNIGHT => {
                if self.turn {
                    self.hash_keys.white_knight[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                } else {
                    self.hash_keys.black_knight[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                }
            }
            ROOK => {
                if self.turn {
                    self.hash_keys.white_rook[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                } else {
                    self.hash_keys.black_rook[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                }
            }
            QUEEN => {
                if self.turn {
                    self.hash_keys.white_queen[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                } else {
                    self.hash_keys.black_queen[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                }
            }
            KING => {
                if self.turn {
                    self.hash_keys.white_king[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                } else {
                    self.hash_keys.black_king[move_to_make.from.0 as usize]
                        [move_to_make.from.1 as usize]
                }
            }
            _ => panic!("invalid piece type! :skull:"),
        };

        //set en passant square
        if get_piece_type!(move_to_make.piece) == PAWN
            && ((self.turn && move_to_make.from.0 - move_to_make.to.0 == 2)
                | (!self.turn && move_to_make.to.0 - move_to_make.from.0 == 2))
        {
            self.en_passant_square = Some((if self.turn { 5 } else { 2 }, move_to_make.to.1));

            self.hash ^= self.hash_keys.en_passant_square_file[move_to_make.to.1 as usize]
        } else {
            match self.en_passant_square {
                Some(square) => {
                    self.hash ^= self.hash_keys.en_passant_square_file[square.1 as usize]
                }
                None => {}
            }

            self.en_passant_square = None;
        }

        // en passant captures
        if move_to_make.en_passant {
            if self.turn {
                self.board[move_to_make.to.0 as usize + 1][move_to_make.to.1 as usize] = 0;
                self.hash ^= self.hash_keys.white_pawn[move_to_make.to.0 as usize]
                    [move_to_make.to.1 as usize];
            } else {
                self.board[move_to_make.to.0 as usize - 1][move_to_make.to.1 as usize] = 0;
                self.hash ^= self.hash_keys.black_pawn[move_to_make.to.0 as usize]
                    [move_to_make.to.1 as usize];
            }

            self.hash ^= match get_piece_type!(move_to_make.capture_piece.unwrap()) {
                PAWN => {
                    if !self.turn {
                        self.hash_keys.white_pawn[move_to_make.from.0 as usize - 1]
                            [move_to_make.from.1 as usize]
                    } else {
                        self.hash_keys.black_pawn[move_to_make.from.0 as usize + 1]
                            [move_to_make.from.1 as usize]
                    }
                }
                BISHOP => {
                    if !self.turn {
                        self.hash_keys.white_bishop[move_to_make.from.0 as usize - 1]
                            [move_to_make.from.1 as usize]
                    } else {
                        self.hash_keys.black_bishop[move_to_make.from.0 as usize + 1]
                            [move_to_make.from.1 as usize]
                    }
                }
                KNIGHT => {
                    if !self.turn {
                        self.hash_keys.white_knight[move_to_make.from.0 as usize - 1]
                            [move_to_make.from.1 as usize]
                    } else {
                        self.hash_keys.black_knight[move_to_make.from.0 as usize + 1]
                            [move_to_make.from.1 as usize]
                    }
                }
                ROOK => {
                    if !self.turn {
                        self.hash_keys.white_rook[move_to_make.from.0 as usize - 1]
                            [move_to_make.from.1 as usize]
                    } else {
                        self.hash_keys.black_rook[move_to_make.from.0 as usize + 1]
                            [move_to_make.from.1 as usize]
                    }
                }
                QUEEN => {
                    if !self.turn {
                        self.hash_keys.white_queen[move_to_make.from.0 as usize - 1]
                            [move_to_make.from.1 as usize]
                    } else {
                        self.hash_keys.black_queen[move_to_make.from.0 as usize + 1]
                            [move_to_make.from.1 as usize]
                    }
                }
                _ => panic!("invalid piece type! :skull:"),
            }
        } else {
            match move_to_make.capture_piece {
                Some(piece) => {
                    self.hash ^= match get_piece_type!(piece) {
                        PAWN => {
                            if !self.turn {
                                self.hash_keys.white_pawn[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            } else {
                                self.hash_keys.black_pawn[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            }
                        }
                        BISHOP => {
                            if !self.turn {
                                self.hash_keys.white_bishop[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            } else {
                                self.hash_keys.black_bishop[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            }
                        }
                        KNIGHT => {
                            if !self.turn {
                                self.hash_keys.white_knight[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            } else {
                                self.hash_keys.black_knight[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            }
                        }
                        ROOK => {
                            if !self.turn {
                                self.hash_keys.white_rook[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            } else {
                                self.hash_keys.black_rook[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            }
                        }
                        QUEEN => {
                            if !self.turn {
                                self.hash_keys.white_queen[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            } else {
                                self.hash_keys.black_queen[move_to_make.from.0 as usize]
                                    [move_to_make.from.1 as usize]
                            }
                        }
                        _ => panic!("invalid piece type! :skull:"),
                    }
                }
                None => {}
            }
        }

        // move
        self.board[move_to_make.to.0 as usize][move_to_make.to.1 as usize] =
            match move_to_make.promotion_piece {
                Some(piece) => piece,
                None => self.board[move_to_make.from.0 as usize][move_to_make.from.1 as usize],
            };
        self.board[move_to_make.from.0 as usize][move_to_make.from.1 as usize] = 0;

        match move_to_make.promotion_piece {
            Some(piece) => {
                self.hash ^= match get_piece_type!(piece) {
                    BISHOP => {
                        if self.turn {
                            self.hash_keys.white_bishop[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_bishop[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    KNIGHT => {
                        if self.turn {
                            self.hash_keys.white_knight[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_knight[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    ROOK => {
                        if self.turn {
                            self.hash_keys.white_rook[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_rook[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    QUEEN => {
                        if self.turn {
                            self.hash_keys.white_queen[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_queen[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    _ => panic!("invalid piece type! :skull:"),
                }
            }
            None => {
                self.hash ^= match get_piece_type!(move_to_make.piece) {
                    PAWN => {
                        if self.turn {
                            self.hash_keys.white_pawn[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_pawn[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    BISHOP => {
                        if self.turn {
                            self.hash_keys.white_bishop[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_bishop[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    KNIGHT => {
                        if self.turn {
                            self.hash_keys.white_knight[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_knight[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    ROOK => {
                        if self.turn {
                            self.hash_keys.white_rook[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_rook[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    QUEEN => {
                        if self.turn {
                            self.hash_keys.white_queen[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_queen[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    KING => {
                        if self.turn {
                            self.hash_keys.white_king[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        } else {
                            self.hash_keys.black_king[move_to_make.to.0 as usize]
                                [move_to_make.to.1 as usize]
                        }
                    }
                    _ => panic!("invalid piece type! :skull:"),
                }
            }
        }

        // castling
        if get_piece_type!(move_to_make.piece) == KING {
            // king position logic
            if self.turn {
                self.white_king_position = move_to_make.to;
            } else {
                self.black_king_position = move_to_make.to;
            }

            if move_to_make.from == (7, 4) && move_to_make.to == (7, 6) {
                self.board[7][5] = get_piece_color!(move_to_make.piece) | ROOK;
                self.board[7][7] = 0;
            } else if move_to_make.from == (7, 4) && move_to_make.to == (7, 2) {
                self.board[7][3] = get_piece_color!(move_to_make.piece) | ROOK;
                self.board[7][0] = 0;
            } else if move_to_make.from == (0, 4) && move_to_make.to == (0, 6) {
                self.board[0][5] = get_piece_color!(move_to_make.piece) | ROOK;
                self.board[0][7] = 0;
            } else if move_to_make.from == (0, 4) && move_to_make.to == (0, 2) {
                self.board[0][3] = get_piece_color!(move_to_make.piece) | ROOK;
                self.board[0][0] = 0;
            }

            if self.turn {
                if is_white_kingside!(self.castle_state) {
                    self.hash ^= self.hash_keys.white_ks;
                }
                if is_white_queenside!(self.castle_state) {
                    self.hash ^= self.hash_keys.white_qs;
                }
            } else {
                if is_black_kingside!(self.castle_state) {
                    self.hash ^= self.hash_keys.black_ks;
                }
                if is_black_queenside!(self.castle_state) {
                    self.hash ^= self.hash_keys.black_qs;
                }
            }

            // if king is moved, cannot castle
            self.castle_state &= if self.turn { 0b0011 } else { 0b1100 };
        } else if get_piece_type!(move_to_make.piece) == ROOK {
            // if rook is moved, that side can no longer castle
            if move_to_make.from == (0, 0) {
                if is_black_queenside!(self.castle_state) {
                    self.hash ^= self.hash_keys.black_qs;
                }

                self.castle_state &= !BLACK_QUEENSIDE;
            } else if move_to_make.from == (0, 7) {
                if is_black_kingside!(self.castle_state) {
                    self.hash ^= self.hash_keys.black_ks;
                }

                self.castle_state &= !BLACK_KINGSIDE;
            } else if move_to_make.from == (7, 0) {
                if is_white_queenside!(self.castle_state) {
                    self.hash ^= self.hash_keys.white_qs;
                }

                self.castle_state &= !WHITE_QUEENSIDE;
            } else if move_to_make.from == (7, 7) {
                if is_white_kingside!(self.castle_state) {
                    self.hash ^= self.hash_keys.white_ks;
                }

                self.castle_state &= !WHITE_KINGSIDE;
            }
        }
        // if rook is captured, can no longer castle
        if move_to_make.to == (0, 0) {
            if is_black_queenside!(self.castle_state) {
                self.hash ^= self.hash_keys.black_qs;
            }
            self.castle_state &= !BLACK_QUEENSIDE;
        } else if move_to_make.to == (0, 7) {
            if is_black_kingside!(self.castle_state) {
                self.hash ^= self.hash_keys.black_ks;
            }
            self.castle_state &= !BLACK_KINGSIDE;
        } else if move_to_make.to == (7, 0) {
            if is_white_queenside!(self.castle_state) {
                self.hash ^= self.hash_keys.white_qs;
            }
            self.castle_state &= !WHITE_QUEENSIDE;
        } else if move_to_make.to == (7, 7) {
            if is_white_kingside!(self.castle_state) {
                self.hash ^= self.hash_keys.white_ks;
            }
            self.castle_state &= !WHITE_KINGSIDE;
        }

        self.update_bitboards(move_to_make);

        if !self.turn {
            self.fullmoves += 1
        }
        self.turn = !self.turn;
        self.hash ^= self.hash_keys.turn_key;

        move |board: &mut Board| {
            board.board = prev_board;
            board.white = prev_white;
            board.black = prev_black;
            board.turn = prev_turn;
            board.castle_state = prev_castle_state;
            board.en_passant_square = prev_en_passant_square;
            board.fullmoves = prev_fullmoves;
            board.white_king_position = prev_white_king_position;
            board.black_king_position = prev_black_king_position;
            board.hash = prev_hash;
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board_str: String = String::from(" abcdefgh\n");
        let mut rank_index: u8 = 0;
        for rank in self.board {
            board_str += &(8 - rank_index).to_string();
            for square in rank {
                board_str += match get_piece_type!(square) {
                    PAWN => match get_piece_color!(square) {
                        WHITE => "P",
                        BLACK => "p",
                        _ => "",
                    },
                    BISHOP => match get_piece_color!(square) {
                        WHITE => "B",
                        BLACK => "b",
                        _ => "",
                    },
                    KNIGHT => match get_piece_color!(square) {
                        WHITE => "N",
                        BLACK => "n",
                        _ => "",
                    },
                    ROOK => match get_piece_color!(square) {
                        WHITE => "R",
                        BLACK => "r",
                        _ => "",
                    },
                    QUEEN => match get_piece_color!(square) {
                        WHITE => "Q",
                        BLACK => "q",
                        _ => "",
                    },
                    KING => match get_piece_color!(square) {
                        WHITE => "K",
                        BLACK => "k",
                        _ => "",
                    },
                    _ => ".",
                }
            }
            board_str += &(rank_index.to_string() + " | ");

            if rank_index == 0 {
                board_str += &("pawns:   ".to_owned()
                    + &format!("{:064b}", self.white.pawns)
                    + " | "
                    + &format!("{:064b}", self.black.pawns));
            } else if rank_index == 1 {
                board_str += &("bishops: ".to_owned()
                    + &format!("{:064b}", self.white.bishops)
                    + " | "
                    + &format!("{:064b}", self.black.bishops));
            } else if rank_index == 2 {
                board_str += &("knights: ".to_owned()
                    + &format!("{:064b}", self.white.knights)
                    + " | "
                    + &format!("{:064b}", self.black.knights));
            } else if rank_index == 3 {
                board_str += &("rooks:   ".to_owned()
                    + &format!("{:064b}", self.white.rooks)
                    + " | "
                    + &format!("{:064b}", self.black.rooks));
            } else if rank_index == 4 {
                board_str += &("queens:  ".to_owned()
                    + &format!("{:064b}", self.white.queens)
                    + " | "
                    + &format!("{:064b}", self.black.queens));
            } else if rank_index == 5 {
                board_str += &("king:    ".to_owned()
                    + &format!("{:064b}", self.white.king)
                    + " | "
                    + &format!("{:064b}", self.black.king));
            } else if rank_index == 6 {
                board_str += &("all:     ".to_owned()
                    + &format!("{:064b}", self.white.all)
                    + " | "
                    + &format!("{:064b}", self.black.all))
            } else if rank_index == 7 {
                match self.en_passant_square {
                    Some(square) => board_str += &format!("{:?} ", square),
                    None => board_str += "No En Passant Square ",
                }
                board_str += if is_white_kingside!(self.castle_state) {
                    "K"
                } else {
                    ""
                };
                board_str += if is_white_queenside!(self.castle_state) {
                    "Q"
                } else {
                    ""
                };
                board_str += if is_black_kingside!(self.castle_state) {
                    "k"
                } else {
                    ""
                };
                board_str += if is_black_queenside!(self.castle_state) {
                    "q"
                } else {
                    ""
                };

                board_str += &format!(" hash: {:064b}", self.hash);
            }

            board_str += "\n";
            rank_index += 1;
        }
        board_str += " 01234567\n";

        f.write_str(&board_str)
    }
}

pub struct BoardBuilder {
    board: Board,
}

impl BoardBuilder {
    pub fn new() -> BoardBuilder {
        BoardBuilder {
            board: Board {
                board: [[0; 8]; 8],
                white: Bitboards {
                    pawns: 0u64,
                    bishops: 0u64,
                    knights: 0u64,
                    rooks: 0u64,
                    queens: 0u64,
                    king: 0u64,
                    all: 0u64,
                },
                black: Bitboards {
                    pawns: 0u64,
                    bishops: 0u64,
                    knights: 0u64,
                    rooks: 0u64,
                    queens: 0u64,
                    king: 0u64,
                    all: 0u64,
                },
                turn: WHITE_TO_MOVE,
                castle_state: 0,
                en_passant_square: None,
                fullmoves: 0,
                knight_masks: [[0; 8]; 8],
                king_masks: [[0; 8]; 8],
                ray_attacks: [[[0; 8]; 8]; 8],
                bishop_blocker_masks: [[0; 8]; 8],
                rook_blocker_masks: [[0; 8]; 8],
                white_king_position: (0, 0),
                black_king_position: (0, 0),
                hash_keys: HashKeys::new(),
                hash: 0,
            },
        }
    }

    pub fn set_turn(&mut self, to_move: bool) -> &mut BoardBuilder {
        self.board.turn = to_move;
        self
    }

    pub fn set_castle_state(
        &mut self,
        wks: bool,
        wqs: bool,
        bks: bool,
        bqs: bool,
    ) -> &mut BoardBuilder {
        self.board.castle_state = 0;

        if wks {
            self.board.castle_state |= 0b1000;
        }

        if wqs {
            self.board.castle_state |= 0b0100;
        }

        if bks {
            self.board.castle_state |= 0b0010;
        }

        if bqs {
            self.board.castle_state |= 0b0001;
        }

        self
    }

    pub fn set_en_passant_square(&mut self, rank: u8, file: u8) -> &mut BoardBuilder {
        self.board.en_passant_square = Some((rank, file));

        self
    }

    pub fn set_fullmoves(&mut self, fullmoves: u16) -> &mut BoardBuilder {
        self.board.fullmoves = fullmoves;

        self
    }

    pub fn set_position(&mut self, fen: String) -> &mut BoardBuilder {
        let tokens: Vec<&str> = fen.split(' ').collect();

        let mut index: usize = 63;
        for row in tokens[0].split('/') {
            for c in row.chars() {
                if !c.is_digit(10) {
                    if c.is_uppercase() {
                        self.board.white.all |= 0b1 << index;
                    } else {
                        self.board.black.all |= 0b1 << index;
                    }
                }

                self.board.board[7 - index / 8][7 - index % 8] = match c {
                    'P' => {
                        self.board.white.pawns |= 0b1 << index;
                        WHITE | PAWN
                    }
                    'p' => {
                        self.board.black.pawns |= 0b1 << index;
                        BLACK | PAWN
                    }
                    'B' => {
                        self.board.white.bishops |= 0b1 << index;
                        WHITE | BISHOP
                    }
                    'b' => {
                        self.board.black.bishops |= 0b1 << index;
                        BLACK | BISHOP
                    }
                    'N' => {
                        self.board.white.knights |= 0b1 << index;
                        WHITE | KNIGHT
                    }
                    'n' => {
                        self.board.black.knights |= 0b1 << index;
                        BLACK | KNIGHT
                    }
                    'R' => {
                        self.board.white.rooks |= 0b1 << index;
                        WHITE | ROOK
                    }
                    'r' => {
                        self.board.black.rooks |= 0b1 << index;
                        BLACK | ROOK
                    }
                    'Q' => {
                        self.board.white.queens |= 0b1 << index;
                        WHITE | QUEEN
                    }
                    'q' => {
                        self.board.black.queens |= 0b1 << index;
                        BLACK | QUEEN
                    }
                    'K' => {
                        self.board.white.king |= 0b1 << index;
                        self.board.white_king_position = (7 - index as u8 / 8, 7 - index as u8 % 8);
                        WHITE | KING
                    }
                    'k' => {
                        self.board.black.king |= 0b1 << index;
                        self.board.black_king_position = (7 - index as u8 / 8, 7 - index as u8 % 8);
                        BLACK | KING
                    }
                    _ => {
                        let num: Option<u32> = c.to_digit(10);
                        for _ in 0..(num.unwrap_or(0) - 1) {
                            self.board.board[7 - index / 8][7 - index % 8] = 0b0;
                            index -= 1;
                        }

                        0b0
                    }
                };

                index -= if index > 0 { 1 } else { 0 };
            }
        }

        self.board.turn = tokens[1] == "w";

        if tokens[2].contains('K') {
            self.board.castle_state |= WHITE_KINGSIDE;
        }

        if tokens[2].contains('Q') {
            self.board.castle_state |= WHITE_QUEENSIDE;
        }

        if tokens[2].contains('k') {
            self.board.castle_state |= BLACK_KINGSIDE;
        }

        if tokens[2].contains('q') {
            self.board.castle_state |= BLACK_QUEENSIDE;
        }

        if tokens[3] != "-" {
            self.board.en_passant_square = Some(square_from_uci(tokens[3]));
        }

        self.board.fullmoves = tokens[5].parse().expect("could not parse fullmoves");

        self.board.hash_keys.clone().generate_hash(&mut self.board);

        self
    }

    pub fn build(&mut self) -> Board {
        self.board.generate_knight_masks();
        self.board.generate_king_masks();
        self.board.generate_ray_attacks();
        self.board.generate_rook_blocker_masks();
        self.board.generate_bishop_blocker_masks();

        self.board
    }
}

pub fn print_bitboard(bitboard: u64) {
    let row_mask: u64 = 0b11111111;

    println!("{:08b}", (bitboard & (row_mask << 56)) >> 56);
    println!("{:08b}", (bitboard & (row_mask << 48)) >> 48);
    println!("{:08b}", (bitboard & (row_mask << 40)) >> 40);
    println!("{:08b}", (bitboard & (row_mask << 32)) >> 32);
    println!("{:08b}", (bitboard & (row_mask << 24)) >> 24);
    println!("{:08b}", (bitboard & (row_mask << 16)) >> 16);
    println!("{:08b}", (bitboard & (row_mask << 8)) >> 8);
    println!("{:08b}", bitboard & row_mask);
}
