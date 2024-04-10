use crate::{
    get_bit_index, get_piece_color, get_piece_type, is_black_kingside, is_black_queenside,
    is_white_kingside, is_white_queenside,
};

use super::{
    Board, Move, Piece, BISHOP, BLACK, BLACK_KINGSIDE, BLACK_QUEENSIDE, KING, KNIGHT, PAWN, QUEEN,
    ROOK, WHITE, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};

//directions knight can move
const KNIGHT_DIRECTIONS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (1, -2),
    (2, -1),
    (-1, 2),
    (-2, 1),
    (-1, -2),
    (-2, -1),
];

const KING_DIRECTIONS: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;
const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;

// use to check if there are pieces blocking castling
const WHITE_KINGSIDE_OCCUPANCY_MASK: u64 = 0b110;
const WHITE_QUEENSIDE_OCCUPANCY_MASK: u64 = 0b01110000;
const BLACK_KINGSIDE_OCCUPANCY_MASK: u64 =
    0b00000110_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BLACK_QUEENSIDE_OCCUPANCY_MASK: u64 =
    0b01110000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

const NORTH: u8 = 0;
const NORTHEAST: u8 = 1;
const EAST: u8 = 2;
const SOUTHEAST: u8 = 3;
const SOUTH: u8 = 4;
const SOUTHWEST: u8 = 5;
const WEST: u8 = 6;
const NORTHWEST: u8 = 7;

impl Board {
    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(218);

        let mut current_square: (u8, u8) = (0, 0);
        let mut king_position: (u8, u8) = (0, 0);
        for rank in self.board {
            for piece in rank {
                if (get_piece_color!(piece) == WHITE) != self.turn {
                    current_square.1 += 1;
                    continue;
                }

                if get_piece_type!(piece) == KNIGHT {
                    self.generate_knight_moves(&mut moves, current_square);
                }

                if get_piece_type!(piece) == KING {
                    king_position = current_square;
                    self.generate_king_moves(&mut moves, current_square);
                }

                if get_piece_type!(piece) == BISHOP {
                    self.generate_bishop_moves(&mut moves, current_square);
                }

                if get_piece_type!(piece) == ROOK {
                    self.generate_rook_moves(&mut moves, current_square);
                }

                if get_piece_type!(piece) == QUEEN {
                    self.generate_queen_moves(&mut moves, current_square);
                }

                if (current_square.0 == 1 || current_square.0 == 6)
                    && get_piece_type!(piece) == PAWN
                {
                    self.generate_pawn_promotions(&mut moves, current_square);
                }

                current_square.1 += 1;
            }
            current_square.0 += 1;
            current_square.1 = 0;
        }

        self.generate_pawn_pushes(&mut moves);
        self.generate_pawn_captures(&mut moves);

        moves.retain(|&m| {
            let undo = self.make_move(m);
            let in_check;
            //self.turn appears to be opposite because make_move changes the turn
            if get_piece_type!(m.piece) == KING {
                in_check = self.is_in_check(if !self.turn { WHITE } else { BLACK }, m.to);
            } else {
                in_check = self.is_in_check(if !self.turn { WHITE } else { BLACK }, king_position);
            }

            undo(self);
            !in_check
        });
        moves.shrink_to_fit();

        moves
    }

    // this implementation is horrific and probably bad but im too lazy to code a good one
    pub fn generate_legal_captures(&mut self) -> Vec<Move> {
        let mut moves = self.generate_legal_moves();

        moves.retain(|&m| match m.capture_piece {
            Some(_) => true,
            None => false,
        });

        moves
    }

    // use the BLACK / WHITE constants
    pub fn is_in_check(&self, color: u8, king_position: (u8, u8)) -> bool {
        let diagonal_attacks = self.get_ray_attacks(king_position, NORTHEAST)
            | self.get_ray_attacks(king_position, NORTHWEST)
            | self.get_ray_attacks(king_position, SOUTHEAST)
            | self.get_ray_attacks(king_position, SOUTHWEST);

        let orthogonal_attacks = self.get_ray_attacks(king_position, NORTH)
            | self.get_ray_attacks(king_position, EAST)
            | self.get_ray_attacks(king_position, SOUTH)
            | self.get_ray_attacks(king_position, WEST);

        let knight_attacks = self.knight_masks[king_position.0 as usize][king_position.1 as usize];
        let king_attacks = self.king_masks[king_position.0 as usize][king_position.1 as usize];
        let mut east = 0u64;
        let mut west = 0u64;

        if color == WHITE {
            if king_position.1 > 0 {
                west = (1 << get_bit_index!(king_position)) << 9;
            }

            if king_position.1 < 7 {
                east = (1 << get_bit_index!(king_position)) << 7
            }
        } else {
            if king_position.1 > 0 {
                west = (1 << get_bit_index!(king_position)) >> 7
            }

            if king_position.1 < 7 {
                east = (1 << get_bit_index!(king_position)) >> 9
            }
        }

        let pawn_attacks = if color == WHITE && king_position.0 > 0 {
            west | east
        } else if color == BLACK && king_position.0 < 7 {
            west | east
        } else {
            0
        };

        if color == WHITE {
            return diagonal_attacks & self.black.bishops > 0
                || diagonal_attacks & self.black.queens > 0
                || orthogonal_attacks & self.black.rooks > 0
                || orthogonal_attacks & self.black.queens > 0
                || knight_attacks & self.black.knights > 0
                || pawn_attacks & self.black.pawns > 0
                || king_attacks & self.black.king > 0;
        } else if color == BLACK {
            return diagonal_attacks & self.white.bishops > 0
                || diagonal_attacks & self.white.queens > 0
                || orthogonal_attacks & self.white.rooks > 0
                || orthogonal_attacks & self.white.queens > 0
                || knight_attacks & self.white.knights > 0
                || pawn_attacks & self.white.pawns > 0
                || king_attacks & self.white.king > 0;
        } else {
            false
        }
    }

    pub fn generate_knight_masks(&mut self) {
        for rank in 0u8..8 {
            for file in 0u8..8 {
                let mut mask: u64 = 0;
                for direction in KNIGHT_DIRECTIONS {
                    let to = (rank as i8 + direction.0, file as i8 + direction.1);
                    if to.0 >= 0 && to.0 < 8 && to.1 >= 0 && to.1 < 8 {
                        mask |= 1 << get_bit_index!(to);
                    }
                }
                self.knight_masks[rank as usize][file as usize] = mask;
            }
        }
    }

    pub fn generate_king_masks(&mut self) {
        for rank in 0u8..8 {
            for file in 0u8..8 {
                let mut mask: u64 = 0;

                for direction in KING_DIRECTIONS {
                    let to = (rank as i8 + direction.0, file as i8 + direction.1);
                    if to.0 >= 0 && to.0 < 8 && to.1 >= 0 && to.1 < 8 {
                        mask |= 1 << get_bit_index!(to);
                    }
                }

                self.king_masks[rank as usize][file as usize] = mask;
            }
        }
    }

    pub fn generate_ray_attacks(&mut self) {
        for direction in [
            NORTH, NORTHEAST, EAST, SOUTHEAST, SOUTH, SOUTHWEST, WEST, NORTHWEST,
        ] {
            for rank in 0u8..8 {
                for file in 0u8..8 {
                    let mut r = rank;
                    let mut f = file;
                    let mut bit_index = get_bit_index!(rank, file);
                    match direction {
                        NORTH => {
                            while r > 0 {
                                r -= 1;
                                bit_index += 8;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        NORTHEAST => {
                            while r > 0 && f < 7 {
                                r -= 1;
                                f += 1;
                                bit_index += 7;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        EAST => {
                            while f < 7 {
                                f += 1;
                                bit_index -= 1;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        SOUTHEAST => {
                            while r < 7 && f < 7 {
                                r += 1;
                                f += 1;
                                bit_index -= 9;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        SOUTH => {
                            while r < 7 {
                                r += 1;
                                bit_index -= 8;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        SOUTHWEST => {
                            while r < 7 && f > 0 {
                                r += 1;
                                f -= 1;
                                bit_index -= 7;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        WEST => {
                            while f > 0 {
                                f -= 1;
                                bit_index += 1;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        NORTHWEST => {
                            while r > 0 && f > 0 {
                                r -= 1;
                                f -= 1;
                                bit_index += 9;

                                self.ray_attacks[direction as usize][rank as usize]
                                    [file as usize] |= 1 << bit_index;
                            }
                        }
                        _ => {
                            panic!("WRONG DIRECTION :skull:")
                        }
                    }
                }
            }
        }
    }

    // generates list of moves from a target bitboard, does not check for move legality
    // does not handle promotions and en passant
    fn generate_moves_from_targets(
        &self,
        moves: &mut Vec<Move>,
        current_square: (u8, u8),
        piece: Piece,
        targets: u64,
    ) {
        for i in 0u8..64 {
            if targets & (1 << i) > 0 {
                moves.push(Move::new(
                    current_square,
                    (7 - i / 8, 7 - i % 8),
                    piece,
                    match self.board[7 - i as usize / 8][7 - i as usize % 8] {
                        0 => None,
                        piece => Some(piece),
                    },
                    None,
                    false,
                ));
            }
        }
    }

    fn generate_knight_moves(&self, moves: &mut Vec<Move>, current_square: (u8, u8)) {
        // bitwise move generation (i hope it works)
        let mask = self.knight_masks[current_square.0 as usize][current_square.1 as usize];

        self.generate_moves_from_targets(
            moves,
            current_square,
            if self.turn { WHITE } else { BLACK } | KNIGHT,
            mask & if self.turn {
                !self.white.all
            } else {
                !self.black.all
            },
        )
    }

    fn generate_king_moves(&self, moves: &mut Vec<Move>, current_square: (u8, u8)) {
        let mask = self.king_masks[current_square.0 as usize][current_square.1 as usize];
        self.generate_moves_from_targets(
            moves,
            current_square,
            if self.turn { WHITE } else { BLACK } | KING,
            mask & if self.turn {
                !self.white.all
            } else {
                !self.black.all
            },
        );

        // castling
        if self.turn {
            if is_white_kingside!(self.castle_state)
                && (self.white.all | self.black.all) & WHITE_KINGSIDE_OCCUPANCY_MASK == 0
                && !self.is_in_check(WHITE, current_square) // disallow castling when in check
                && !self.is_in_check(WHITE, (current_square.0, current_square.1 + 1))
            // disallow castling through check
            {
                moves.push(Move::new(
                    current_square,
                    (7, 6),
                    WHITE | KING,
                    None,
                    None,
                    false,
                ));
            }

            if is_white_queenside!(self.castle_state)
                && (self.white.all | self.black.all) & WHITE_QUEENSIDE_OCCUPANCY_MASK == 0
                && !self.is_in_check(WHITE, current_square)
                && !self.is_in_check(WHITE, (current_square.0, current_square.1 - 1))
                && !self.is_in_check(WHITE, (current_square.0, current_square.1 - 2))
            {
                moves.push(Move::new(
                    current_square,
                    (7, 2),
                    WHITE | KING,
                    None,
                    None,
                    false,
                ));
            }
        } else {
            if is_black_kingside!(self.castle_state)
                && (self.white.all | self.black.all) & BLACK_KINGSIDE_OCCUPANCY_MASK == 0
                && !self.is_in_check(BLACK, current_square)
                && !self.is_in_check(BLACK, (current_square.0, current_square.1 + 1))
            {
                moves.push(Move::new(
                    current_square,
                    (0, 6),
                    BLACK | KING,
                    None,
                    None,
                    false,
                ));
            }

            if is_black_queenside!(self.castle_state)
                && (self.white.all | self.black.all) & BLACK_QUEENSIDE_OCCUPANCY_MASK == 0
                && !self.is_in_check(BLACK, current_square)
                && !self.is_in_check(BLACK, (current_square.0, current_square.1 - 1))
                && !self.is_in_check(BLACK, (current_square.0, current_square.1 - 2))
            {
                moves.push(Move::new(
                    current_square,
                    (0, 2),
                    BLACK | KING,
                    None,
                    None,
                    false,
                ));
            }
        }
    }

    // not promotions
    fn generate_pawn_pushes(&self, moves: &mut Vec<Move>) {
        if self.turn {
            // handle promotions elsewhere
            for i in 8u8..48 {
                let mut targets: u64 = 0;
                let current_pawn: u64 = self.white.pawns & (1 << i);

                if current_pawn == 0 {
                    continue;
                };

                targets |= (current_pawn << 8) & !self.black.all & !self.white.all;

                if i < 16 && targets > 0 {
                    targets |= (current_pawn << 16) & !self.black.all & !self.white.all;
                }

                self.generate_moves_from_targets(
                    moves,
                    (7 - i / 8, 7 - i % 8),
                    WHITE | PAWN,
                    targets,
                )
            }
        } else {
            // handle promotions elsewhere
            for i in 16u8..56 {
                let mut targets: u64 = 0;
                let current_pawn: u64 = self.black.pawns & (1 << i);

                if current_pawn == 0 {
                    continue;
                };

                targets |= (current_pawn >> 8) & !self.white.all & !self.black.all;

                if i >= 48 && targets > 0 {
                    targets |= (current_pawn >> 16) & !self.white.all & !self.black.all;
                }

                self.generate_moves_from_targets(
                    moves,
                    (7 - i / 8, 7 - i % 8),
                    BLACK | PAWN,
                    targets,
                )
            }
        }
    }

    fn generate_pawn_captures(&self, moves: &mut Vec<Move>) {
        if self.turn {
            for i in 8u8..48 {
                let current_pawn: u64 = self.white.pawns & (1 << i);

                if current_pawn == 0 {
                    continue;
                }

                let mut targets = 0;
                match self.en_passant_square {
                    Some(square) => {
                        if 7 - i % 8 > 0 && (6 - i / 8, 6 - i % 8) == square
                            || 7 - i % 8 < 7 && (6 - i / 8, 8 - i % 8) == square
                        {
                            moves.push(Move::new(
                                (7 - i / 8, 7 - i % 8),
                                square,
                                WHITE | PAWN,
                                Some(BLACK | PAWN),
                                None,
                                true,
                            ));
                        }
                    }
                    None => {}
                };

                // capturing to the right
                targets |= ((current_pawn & NOT_A_FILE) << 7) & !self.white.all & self.black.all;
                // capturing to the left
                targets |= ((current_pawn & NOT_H_FILE) << 9) & !self.white.all & self.black.all;

                self.generate_moves_from_targets(
                    moves,
                    (7 - i / 8, 7 - i % 8),
                    WHITE | PAWN,
                    targets,
                );
            }
        } else {
            for i in 16u8..56 {
                let current_pawn: u64 = self.black.pawns & (1 << i);

                if current_pawn == 0 {
                    continue;
                }

                let mut targets = 0;
                match self.en_passant_square {
                    Some(square) => {
                        if 7 - i % 8 > 0 && (8 - i / 8, 6 - i % 8) == square
                            || 7 - i % 8 < 7 && (8 - i / 8, 8 - i % 8) == square
                        {
                            moves.push(Move::new(
                                (7 - i / 8, 7 - i % 8),
                                square,
                                BLACK | PAWN,
                                Some(WHITE | PAWN),
                                None,
                                true,
                            ));
                        }
                    }
                    None => {}
                };

                // capturing to the right
                targets |= ((current_pawn & NOT_A_FILE) >> 9) & !self.black.all & self.white.all;
                // capturing to the left
                targets |= ((current_pawn & NOT_H_FILE) >> 7) & !self.black.all & self.white.all;

                self.generate_moves_from_targets(
                    moves,
                    (7 - i / 8, 7 - i % 8),
                    BLACK | PAWN,
                    targets,
                );
            }
        }
    }

    fn generate_pawn_promotions(&self, moves: &mut Vec<Move>, current_square: (u8, u8)) {
        if self.turn && current_square.0 == 1 {
            if self.board[current_square.0 as usize - 1][current_square.1 as usize] == 0 {
                for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move::new(
                        current_square,
                        (current_square.0 - 1, current_square.1),
                        WHITE | PAWN,
                        None,
                        Some(WHITE | piece),
                        false,
                    ));
                }
            }

            // capture to the left
            if current_square.1 > 0
                && get_piece_color!(
                    self.board[current_square.0 as usize - 1][current_square.1 as usize - 1]
                ) == BLACK
            {
                for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move::new(
                        current_square,
                        (current_square.0 - 1, current_square.1 - 1),
                        WHITE | PAWN,
                        Some(
                            self.board[current_square.0 as usize - 1]
                                [current_square.1 as usize - 1],
                        ),
                        Some(WHITE | piece),
                        false,
                    ));
                }
            }

            // capture to the right
            if current_square.1 < 7
                && get_piece_color!(
                    self.board[current_square.0 as usize - 1][current_square.1 as usize + 1]
                ) == BLACK
            {
                for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move::new(
                        current_square,
                        (current_square.0 - 1, current_square.1 + 1),
                        WHITE | PAWN,
                        Some(
                            self.board[current_square.0 as usize - 1]
                                [current_square.1 as usize + 1],
                        ),
                        Some(WHITE | piece),
                        false,
                    ));
                }
            }
        } else if !self.turn && current_square.0 == 6 {
            if self.board[current_square.0 as usize + 1][current_square.1 as usize] == 0 {
                for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move::new(
                        current_square,
                        (current_square.0 + 1, current_square.1),
                        BLACK | PAWN,
                        None,
                        Some(BLACK | piece),
                        false,
                    ));
                }
            }

            // capture to the left
            if current_square.1 > 0
                && get_piece_color!(
                    self.board[current_square.0 as usize + 1][current_square.1 as usize - 1]
                ) == WHITE
            {
                for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move::new(
                        current_square,
                        (current_square.0 + 1, current_square.1 - 1),
                        BLACK | PAWN,
                        Some(
                            self.board[current_square.0 as usize + 1]
                                [current_square.1 as usize - 1],
                        ),
                        Some(BLACK | piece),
                        false,
                    ));
                }
            }

            // capture to the right
            if current_square.1 < 7
                && get_piece_color!(
                    self.board[current_square.0 as usize + 1][current_square.1 as usize + 1]
                ) == WHITE
            {
                for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move::new(
                        current_square,
                        (current_square.0 + 1, current_square.1 + 1),
                        BLACK | PAWN,
                        Some(
                            self.board[current_square.0 as usize + 1]
                                [current_square.1 as usize + 1],
                        ),
                        Some(BLACK | piece),
                        false,
                    ));
                }
            }
        }
    }

    fn generate_bishop_moves(&self, moves: &mut Vec<Move>, current_square: (u8, u8)) {
        let attacks = self.get_ray_attacks(current_square, NORTHEAST)
            | self.get_ray_attacks(current_square, NORTHWEST)
            | self.get_ray_attacks(current_square, SOUTHEAST)
            | self.get_ray_attacks(current_square, SOUTHWEST);

        self.generate_moves_from_targets(
            moves,
            current_square,
            if self.turn { WHITE } else { BLACK } | BISHOP,
            attacks
                & if self.turn {
                    !self.white.all
                } else {
                    !self.black.all
                },
        );
    }

    fn generate_rook_moves(&self, moves: &mut Vec<Move>, current_square: (u8, u8)) {
        let attacks = self.get_ray_attacks(current_square, NORTH)
            | self.get_ray_attacks(current_square, WEST)
            | self.get_ray_attacks(current_square, SOUTH)
            | self.get_ray_attacks(current_square, EAST);

        self.generate_moves_from_targets(
            moves,
            current_square,
            if self.turn { WHITE } else { BLACK } | ROOK,
            attacks
                & if self.turn {
                    !self.white.all
                } else {
                    !self.black.all
                },
        );
    }

    fn generate_queen_moves(&self, moves: &mut Vec<Move>, current_square: (u8, u8)) {
        let attacks = self.get_ray_attacks(current_square, NORTHEAST)
            | self.get_ray_attacks(current_square, NORTHWEST)
            | self.get_ray_attacks(current_square, SOUTHEAST)
            | self.get_ray_attacks(current_square, SOUTHWEST)
            | self.get_ray_attacks(current_square, NORTH)
            | self.get_ray_attacks(current_square, WEST)
            | self.get_ray_attacks(current_square, SOUTH)
            | self.get_ray_attacks(current_square, EAST);

        self.generate_moves_from_targets(
            moves,
            current_square,
            if self.turn { WHITE } else { BLACK } | QUEEN,
            attacks
                & if self.turn {
                    !self.white.all
                } else {
                    !self.black.all
                },
        );
    }

    // shifted bitboards: https://www.chessprogramming.org/Shifted_Bitboards
    fn get_ray_attacks(&self, current_square: (u8, u8), direction: u8) -> u64 {
        let mut attacks = self.ray_attacks[direction as usize][current_square.0 as usize]
            [current_square.1 as usize];
        let mut blockers = attacks & (self.white.all | self.black.all);

        if blockers > 0 {
            blockers = match direction {
                NORTH => {
                    (blockers << 8)
                        | (blockers << 16)
                        | (blockers << 24)
                        | (blockers << 32)
                        | (blockers << 40)
                        | (blockers << 48)
                }
                NORTHEAST => {
                    (blockers << 7)
                        | (blockers << 14)
                        | (blockers << 21)
                        | (blockers << 28)
                        | (blockers << 35)
                        | (blockers << 42)
                }
                EAST => {
                    (blockers >> 1)
                        | (blockers >> 2)
                        | (blockers >> 3)
                        | (blockers >> 4)
                        | (blockers >> 5)
                        | (blockers >> 6)
                }
                SOUTHEAST => {
                    (blockers >> 9)
                        | (blockers >> 18)
                        | (blockers >> 27)
                        | (blockers >> 36)
                        | (blockers >> 45)
                        | (blockers >> 54)
                }
                SOUTH => {
                    (blockers >> 8)
                        | (blockers >> 16)
                        | (blockers >> 24)
                        | (blockers >> 32)
                        | (blockers >> 40)
                        | (blockers >> 48)
                }
                SOUTHWEST => {
                    (blockers >> 7)
                        | (blockers >> 14)
                        | (blockers >> 21)
                        | (blockers >> 28)
                        | (blockers >> 35)
                        | (blockers >> 42)
                }
                WEST => {
                    (blockers << 1)
                        | (blockers << 2)
                        | (blockers << 3)
                        | (blockers << 4)
                        | (blockers << 5)
                        | (blockers << 6)
                }
                NORTHWEST => {
                    (blockers << 9)
                        | (blockers << 18)
                        | (blockers << 27)
                        | (blockers << 36)
                        | (blockers << 45)
                        | (blockers << 54)
                }
                _ => {
                    panic!("invalid direction :skull:")
                }
            };
            attacks ^= blockers & attacks;
        }

        attacks
    }
}
