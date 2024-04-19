pub mod magics;
pub mod perft;
pub mod san;

use game::precomputed::{EAST, NORTH, NORTHEAST, NORTHWEST, SOUTH, SOUTHEAST, SOUTHWEST, WEST};
use game::{
    get_bit_index, get_piece_color, get_piece_type, is_black_kingside, is_black_queenside,
    is_white_kingside, is_white_queenside,
};

use game::{
    Board, Move, Piece, BISHOP, BLACK, BLACK_KINGSIDE, BLACK_QUEENSIDE, KING, KNIGHT, PAWN, QUEEN,
    ROOK, WHITE, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};

const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;
const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;

// use to check if there are pieces blocking castling
const WHITE_KINGSIDE_OCCUPANCY_MASK: u64 = 0b110;
const WHITE_QUEENSIDE_OCCUPANCY_MASK: u64 = 0b01110000;
const BLACK_KINGSIDE_OCCUPANCY_MASK: u64 =
    0b00000110_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
const BLACK_QUEENSIDE_OCCUPANCY_MASK: u64 =
    0b01110000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

pub fn generate_legal_moves(board: &mut Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(218);

    let mut current_square: (u8, u8) = (0, 0);
    let mut king_position: (u8, u8) = (0, 0);
    for rank in board.board {
        for piece in rank {
            if (get_piece_color!(piece) == WHITE) != board.turn {
                current_square.1 += 1;
                continue;
            }

            if get_piece_type!(piece) == KNIGHT {
                generate_knight_moves(board, &mut moves, current_square);
            }

            if get_piece_type!(piece) == KING {
                king_position = current_square;
                generate_king_moves(board, &mut moves, current_square);
            }

            if get_piece_type!(piece) == BISHOP {
                generate_bishop_moves(board, &mut moves, current_square);
            }

            if get_piece_type!(piece) == ROOK {
                generate_rook_moves(board, &mut moves, current_square);
            }

            if get_piece_type!(piece) == QUEEN {
                generate_queen_moves(board, &mut moves, current_square);
            }

            if (current_square.0 == 1 || current_square.0 == 6) && get_piece_type!(piece) == PAWN {
                generate_pawn_promotions(board, &mut moves, current_square);
            }

            current_square.1 += 1;
        }
        current_square.0 += 1;
        current_square.1 = 0;
    }

    generate_pawn_pushes(board, &mut moves);
    generate_pawn_captures(board, &mut moves);

    moves.retain(|&m| {
        let undo = board.make_move(m);
        let in_check;
        //board.turn appears to be opposite because make_move changes the turn
        if get_piece_type!(m.piece) == KING {
            in_check = is_in_check(board, if !board.turn { WHITE } else { BLACK }, m.to);
        } else {
            in_check = is_in_check(
                board,
                if !board.turn { WHITE } else { BLACK },
                king_position,
            );
        }

        undo(board);
        !in_check
    });
    moves.shrink_to_fit();

    moves
}

// this implementation is horrific and probably bad but im too lazy to code a good one
pub fn generate_legal_captures(board: &mut Board) -> Vec<Move> {
    let mut moves = generate_legal_moves(board);

    moves.retain(|&m| match m.capture_piece {
        Some(_) => true,
        None => false,
    });

    moves
}

// use the BLACK / WHITE constants
pub fn is_in_check(board: &Board, color: u8, king_position: (u8, u8)) -> bool {
    let diagonal_attacks = get_ray_attacks(board, king_position, NORTHEAST)
        | get_ray_attacks(board, king_position, NORTHWEST)
        | get_ray_attacks(board, king_position, SOUTHEAST)
        | get_ray_attacks(board, king_position, SOUTHWEST);

    let orthogonal_attacks = get_ray_attacks(board, king_position, NORTH)
        | get_ray_attacks(board, king_position, EAST)
        | get_ray_attacks(board, king_position, SOUTH)
        | get_ray_attacks(board, king_position, WEST);

    let knight_attacks = board.knight_masks[king_position.0 as usize][king_position.1 as usize];
    let king_attacks = board.king_masks[king_position.0 as usize][king_position.1 as usize];
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
        return diagonal_attacks & board.black.bishops > 0
            || diagonal_attacks & board.black.queens > 0
            || orthogonal_attacks & board.black.rooks > 0
            || orthogonal_attacks & board.black.queens > 0
            || knight_attacks & board.black.knights > 0
            || pawn_attacks & board.black.pawns > 0
            || king_attacks & board.black.king > 0;
    } else if color == BLACK {
        return diagonal_attacks & board.white.bishops > 0
            || diagonal_attacks & board.white.queens > 0
            || orthogonal_attacks & board.white.rooks > 0
            || orthogonal_attacks & board.white.queens > 0
            || knight_attacks & board.white.knights > 0
            || pawn_attacks & board.white.pawns > 0
            || king_attacks & board.white.king > 0;
    } else {
        false
    }
}

// generates list of moves from a target bitboard, does not check for move legality
// does not handle promotions and en passant
fn generate_moves_from_targets(
    board: &Board,
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
                match board.board[7 - i as usize / 8][7 - i as usize % 8] {
                    0 => None,
                    piece => Some(piece),
                },
                None,
                false,
            ));
        }
    }
}

fn generate_knight_moves(board: &Board, moves: &mut Vec<Move>, current_square: (u8, u8)) {
    // bitwise move generation (i hope it works)
    let mask = board.knight_masks[current_square.0 as usize][current_square.1 as usize];

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | KNIGHT,
        mask & if board.turn {
            !board.white.all
        } else {
            !board.black.all
        },
    )
}

fn generate_king_moves(board: &Board, moves: &mut Vec<Move>, current_square: (u8, u8)) {
    let mask = board.king_masks[current_square.0 as usize][current_square.1 as usize];
    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | KING,
        mask & if board.turn {
            !board.white.all
        } else {
            !board.black.all
        },
    );

    // castling
    if board.turn {
        if is_white_kingside!(board.castle_state)
                && (board.white.all | board.black.all) & WHITE_KINGSIDE_OCCUPANCY_MASK == 0
                && !is_in_check(board, WHITE, current_square) // disallow castling when in check
                && !is_in_check(board, WHITE, (current_square.0, current_square.1 + 1))
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

        if is_white_queenside!(board.castle_state)
            && (board.white.all | board.black.all) & WHITE_QUEENSIDE_OCCUPANCY_MASK == 0
            && !is_in_check(board, WHITE, current_square)
            && !is_in_check(board, WHITE, (current_square.0, current_square.1 - 1))
            && !is_in_check(board, WHITE, (current_square.0, current_square.1 - 2))
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
        if is_black_kingside!(board.castle_state)
            && (board.white.all | board.black.all) & BLACK_KINGSIDE_OCCUPANCY_MASK == 0
            && !is_in_check(board, BLACK, current_square)
            && !is_in_check(board, BLACK, (current_square.0, current_square.1 + 1))
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

        if is_black_queenside!(board.castle_state)
            && (board.white.all | board.black.all) & BLACK_QUEENSIDE_OCCUPANCY_MASK == 0
            && !is_in_check(board, BLACK, current_square)
            && !is_in_check(board, BLACK, (current_square.0, current_square.1 - 1))
            && !is_in_check(board, BLACK, (current_square.0, current_square.1 - 2))
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
fn generate_pawn_pushes(board: &Board, moves: &mut Vec<Move>) {
    if board.turn {
        // handle promotions elsewhere
        for i in 8u8..48 {
            let mut targets: u64 = 0;
            let current_pawn: u64 = board.white.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            };

            targets |= (current_pawn << 8) & !board.black.all & !board.white.all;

            if i < 16 && targets > 0 {
                targets |= (current_pawn << 16) & !board.black.all & !board.white.all;
            }

            generate_moves_from_targets(board, moves, (7 - i / 8, 7 - i % 8), WHITE | PAWN, targets)
        }
    } else {
        // handle promotions elsewhere
        for i in 16u8..56 {
            let mut targets: u64 = 0;
            let current_pawn: u64 = board.black.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            };

            targets |= (current_pawn >> 8) & !board.white.all & !board.black.all;

            if i >= 48 && targets > 0 {
                targets |= (current_pawn >> 16) & !board.white.all & !board.black.all;
            }

            generate_moves_from_targets(board, moves, (7 - i / 8, 7 - i % 8), BLACK | PAWN, targets)
        }
    }
}

fn generate_pawn_captures(board: &Board, moves: &mut Vec<Move>) {
    if board.turn {
        for i in 8u8..48 {
            let current_pawn: u64 = board.white.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            }

            let mut targets = 0;
            match board.en_passant_square {
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
            targets |= ((current_pawn & NOT_A_FILE) << 7) & !board.white.all & board.black.all;
            // capturing to the left
            targets |= ((current_pawn & NOT_H_FILE) << 9) & !board.white.all & board.black.all;

            generate_moves_from_targets(
                board,
                moves,
                (7 - i / 8, 7 - i % 8),
                WHITE | PAWN,
                targets,
            );
        }
    } else {
        for i in 16u8..56 {
            let current_pawn: u64 = board.black.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            }

            let mut targets = 0;
            match board.en_passant_square {
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
            targets |= ((current_pawn & NOT_A_FILE) >> 9) & !board.black.all & board.white.all;
            // capturing to the left
            targets |= ((current_pawn & NOT_H_FILE) >> 7) & !board.black.all & board.white.all;

            generate_moves_from_targets(
                board,
                moves,
                (7 - i / 8, 7 - i % 8),
                BLACK | PAWN,
                targets,
            );
        }
    }
}

fn generate_pawn_promotions(board: &Board, moves: &mut Vec<Move>, current_square: (u8, u8)) {
    if board.turn && current_square.0 == 1 {
        if board.board[current_square.0 as usize - 1][current_square.1 as usize] == 0 {
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
                board.board[current_square.0 as usize - 1][current_square.1 as usize - 1]
            ) == BLACK
        {
            for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                moves.push(Move::new(
                    current_square,
                    (current_square.0 - 1, current_square.1 - 1),
                    WHITE | PAWN,
                    Some(board.board[current_square.0 as usize - 1][current_square.1 as usize - 1]),
                    Some(WHITE | piece),
                    false,
                ));
            }
        }

        // capture to the right
        if current_square.1 < 7
            && get_piece_color!(
                board.board[current_square.0 as usize - 1][current_square.1 as usize + 1]
            ) == BLACK
        {
            for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                moves.push(Move::new(
                    current_square,
                    (current_square.0 - 1, current_square.1 + 1),
                    WHITE | PAWN,
                    Some(board.board[current_square.0 as usize - 1][current_square.1 as usize + 1]),
                    Some(WHITE | piece),
                    false,
                ));
            }
        }
    } else if !board.turn && current_square.0 == 6 {
        if board.board[current_square.0 as usize + 1][current_square.1 as usize] == 0 {
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
                board.board[current_square.0 as usize + 1][current_square.1 as usize - 1]
            ) == WHITE
        {
            for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                moves.push(Move::new(
                    current_square,
                    (current_square.0 + 1, current_square.1 - 1),
                    BLACK | PAWN,
                    Some(board.board[current_square.0 as usize + 1][current_square.1 as usize - 1]),
                    Some(BLACK | piece),
                    false,
                ));
            }
        }

        // capture to the right
        if current_square.1 < 7
            && get_piece_color!(
                board.board[current_square.0 as usize + 1][current_square.1 as usize + 1]
            ) == WHITE
        {
            for piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                moves.push(Move::new(
                    current_square,
                    (current_square.0 + 1, current_square.1 + 1),
                    BLACK | PAWN,
                    Some(board.board[current_square.0 as usize + 1][current_square.1 as usize + 1]),
                    Some(BLACK | piece),
                    false,
                ));
            }
        }
    }
}

fn generate_bishop_moves(board: &Board, moves: &mut Vec<Move>, current_square: (u8, u8)) {
    let blocker_mask =
        board.bishop_blocker_masks[current_square.0 as usize][current_square.1 as usize];
    let blockers = blocker_mask & (board.white.all | board.black.all);
    let entry = &magics::BISHOP_MAGICS[current_square.0 as usize][current_square.1 as usize];
    let legal_moves = magics::BISHOP_MOVES[magics::magic_index(entry, blockers)];

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | BISHOP,
        legal_moves
            & if board.turn {
                !board.white.all
            } else {
                !board.black.all
            },
    )
}

fn generate_rook_moves(board: &Board, moves: &mut Vec<Move>, current_square: (u8, u8)) {
    let blocker_mask =
        board.rook_blocker_masks[current_square.0 as usize][current_square.1 as usize];
    let blockers = blocker_mask & (board.white.all | board.black.all);
    let entry = &magics::ROOK_MAGICS[current_square.0 as usize][current_square.1 as usize];
    let legal_moves = magics::ROOK_MOVES[magics::magic_index(entry, blockers)];

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | ROOK,
        legal_moves
            & if board.turn {
                !board.white.all
            } else {
                !board.black.all
            },
    )
}

fn generate_queen_moves(board: &Board, moves: &mut Vec<Move>, current_square: (u8, u8)) {
    let ortho_blocker_mask =
        board.rook_blocker_masks[current_square.0 as usize][current_square.1 as usize];
    let ortho_blockers = ortho_blocker_mask & (board.white.all | board.black.all);
    let ortho_entry = &magics::ROOK_MAGICS[current_square.0 as usize][current_square.1 as usize];
    let ortho_legal_moves = magics::ROOK_MOVES[magics::magic_index(ortho_entry, ortho_blockers)];

    let diag_blocker_mask =
        board.bishop_blocker_masks[current_square.0 as usize][current_square.1 as usize];
    let diag_blockers = diag_blocker_mask & (board.white.all | board.black.all);
    let diag_entry = &magics::BISHOP_MAGICS[current_square.0 as usize][current_square.1 as usize];
    let diag_legal_moves = magics::BISHOP_MOVES[magics::magic_index(diag_entry, diag_blockers)];

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | QUEEN,
        (ortho_legal_moves | diag_legal_moves)
            & if board.turn {
                !board.white.all
            } else {
                !board.black.all
            },
    );
}

// shifted bitboards: https://www.chessprogramming.org/Shifted_Bitboards
fn get_ray_attacks(board: &Board, current_square: (u8, u8), direction: u8) -> u64 {
    let mut attacks =
        board.ray_attacks[direction as usize][current_square.0 as usize][current_square.1 as usize];
    let mut blockers = attacks & (board.white.all | board.black.all);

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
