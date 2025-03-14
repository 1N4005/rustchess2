pub mod magics;
pub mod perft;
pub mod san;

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

pub fn generate_legal_moves(board: &mut Board, captures_only: bool) -> Vec<Move> {
    // with_capacity so no allocations are needed during movegen
    let mut moves = Vec::with_capacity(218);

    let mut current_square: (u8, u8) = (0, 0);
    let king_position: (u8, u8) = if board.turn {
        board.white_king_position
    } else {
        board.black_king_position
    };

    let (num_checkers, checkers) =
        count_attackers(board, if board.turn { WHITE } else { BLACK }, king_position);
    let double_check = num_checkers > 1;
    if double_check {
        generate_king_moves(board, &mut moves, king_position, captures_only);
    } else {
        let allowed_targets = if num_checkers == 0 {
            u64::MAX
        } else {
            // we are in check (not double)

            checkers
        }; // if we are in check, checkers includes both the checking pieces and the square which
           // block the check

        let pinned = generate_pinned_moves(board, &mut moves, captures_only, allowed_targets);

        for rank in board.board {
            for piece in rank {
                if (get_piece_color!(piece) == WHITE) != board.turn || piece == 0 {
                    current_square.1 += 1;
                    continue;
                }

                // skip if piece is pinned
                if (1 << get_bit_index!(current_square)) & pinned > 0 {
                    current_square.1 += 1;
                    continue;
                }

                if get_piece_type!(piece) == KNIGHT {
                    generate_knight_moves(
                        board,
                        &mut moves,
                        current_square,
                        captures_only,
                        allowed_targets,
                    );
                }

                if get_piece_type!(piece) == KING {
                    generate_king_moves(board, &mut moves, current_square, captures_only);
                }

                if get_piece_type!(piece) == BISHOP {
                    generate_bishop_moves(
                        board,
                        &mut moves,
                        current_square,
                        captures_only,
                        allowed_targets,
                    );
                }

                if get_piece_type!(piece) == ROOK {
                    generate_rook_moves(
                        board,
                        &mut moves,
                        current_square,
                        captures_only,
                        allowed_targets,
                    );
                }

                if get_piece_type!(piece) == QUEEN {
                    generate_queen_moves(
                        board,
                        &mut moves,
                        current_square,
                        captures_only,
                        allowed_targets,
                    );
                }

                if (current_square.0 == 1 || current_square.0 == 6)
                    && get_piece_type!(piece) == PAWN
                {
                    generate_pawn_promotions(board, &mut moves, current_square, allowed_targets);
                }

                current_square.1 += 1;
            }
            current_square.0 += 1;
            current_square.1 = 0;
        }

        if !captures_only {
            generate_pawn_pushes(board, &mut moves, allowed_targets, !pinned);
        }
        generate_pawn_captures(board, &mut moves, allowed_targets, !pinned);
    }

    if captures_only {
        moves.retain(|&m| m.capture_piece.is_some());
    }
    //moves.shrink_to_fit();

    moves
}

// returns bitboard of pinned pieces
pub fn generate_pinned_moves(
    board: &mut Board,
    moves: &mut Vec<Move>,
    captures_only: bool,
    allowed_targets: u64,
) -> u64 {
    let (our_king_position, our_bitboards, opponent_bitboards) = if board.turn {
        (board.white_king_position, board.white, board.black)
    } else {
        (board.black_king_position, board.black, board.white)
    };

    let mut pinned = 0u64;
    let mut pinners = orthogonal_xrays(board, our_bitboards.all, our_king_position)
        & (opponent_bitboards.queens | opponent_bitboards.rooks); //King Xrays opponent's orthogonal sliders
    while pinners != 0 {
        let pinner = pinners.trailing_zeros();
        let obstructed =
            board.obstructed[get_bit_index!(our_king_position) as usize][pinner as usize];

        let pinned_rooks = obstructed & our_bitboards.rooks;
        let pinned_queens = obstructed & our_bitboards.queens;
        let pinned_pawns = obstructed & our_bitboards.pawns;

        if pinned_rooks > 0 {
            let rook = pinned_rooks.trailing_zeros();
            generate_rook_moves(
                board,
                moves,
                (7 - (rook as u8 / 8), 7 - (rook as u8 % 8)),
                captures_only,
                (obstructed | (pinners & (1 << pinner))) & allowed_targets,
            );
        } else if pinned_queens > 0 {
            let queen = pinned_queens.trailing_zeros();
            generate_queen_moves(
                board,
                moves,
                (7 - (queen as u8 / 8), 7 - (queen as u8 % 8)),
                captures_only,
                (obstructed | (pinners & (1 << pinner))) & allowed_targets,
            );
        } else if pinned_pawns > 0 {
            // orthogonally pinned pawns cannot capture or promote at all
            generate_pawn_pushes(
                board,
                moves,
                (obstructed | (pinners & (1 << pinner))) & allowed_targets,
                pinned_pawns,
            );
        }

        pinned |= obstructed & our_bitboards.all;
        pinners ^= 1 << pinner;
    }

    pinners = diagonal_xrays(board, our_bitboards.all, our_king_position)
        & (opponent_bitboards.queens | opponent_bitboards.bishops);
    while pinners != 0 {
        let pinner = pinners.trailing_zeros();
        let obstructed =
            board.obstructed[get_bit_index!(our_king_position) as usize][pinner as usize];

        let pinned_bishops = obstructed & our_bitboards.bishops;
        let pinned_queens = obstructed & our_bitboards.queens;
        let pinned_pawns = obstructed & our_bitboards.pawns;

        if pinned_bishops > 0 {
            let bishop = pinned_bishops.trailing_zeros();
            generate_bishop_moves(
                board,
                moves,
                (7 - (bishop as u8 / 8), 7 - (bishop as u8 % 8)),
                captures_only,
                (obstructed | (pinners & (1 << pinner))) & allowed_targets,
            );
        } else if pinned_queens > 0 {
            let queen = pinned_queens.trailing_zeros();
            generate_queen_moves(
                board,
                moves,
                (7 - (queen as u8 / 8), 7 - (queen as u8 % 8)),
                captures_only,
                (obstructed | (pinners & (1 << pinner))) & allowed_targets,
            );
        } else if pinned_pawns > 0 {
            // pawns cannot be pushed or promoted when pinned diagonally, but can capture the
            // pinning piece

            generate_pawn_captures(board, moves, (1 << pinner) & allowed_targets, pinned_pawns);
        }

        pinned |= obstructed & our_bitboards.all;
        pinners ^= 1 << pinner;
    }

    pinned
}

// use the BLACK / WHITE constants
pub fn count_attackers(board: &Board, color: u8, square: (u8, u8)) -> (u8, u64) {
    let ortho_blocker_mask = board.rook_blocker_masks[square.0 as usize][square.1 as usize];
    let ortho_blockers = ortho_blocker_mask & (board.white.all | board.black.all);
    let ortho_entry = &magics::ROOK_MAGICS[square.0 as usize][square.1 as usize];
    let orthogonal_attacks = magics::ROOK_MOVES[magics::magic_index(ortho_entry, ortho_blockers)];

    let diag_blocker_mask = board.bishop_blocker_masks[square.0 as usize][square.1 as usize];
    let diag_blockers = diag_blocker_mask & (board.white.all | board.black.all);
    let diag_entry = &magics::BISHOP_MAGICS[square.0 as usize][square.1 as usize];
    let diagonal_attacks = magics::BISHOP_MOVES[magics::magic_index(diag_entry, diag_blockers)];

    let knight_attacks = board.knight_masks[square.0 as usize][square.1 as usize];
    let king_attacks = board.king_masks[square.0 as usize][square.1 as usize];
    let mut east = 0u64;
    let mut west = 0u64;

    if color == WHITE {
        if square.1 > 0 {
            west = (1 << get_bit_index!(square)) << 9;
        }

        if square.1 < 7 {
            east = (1 << get_bit_index!(square)) << 7
        }
    } else {
        if square.1 > 0 {
            west = (1 << get_bit_index!(square)) >> 7
        }

        if square.1 < 7 {
            east = (1 << get_bit_index!(square)) >> 9
        }
    }

    let pawn_attacks = if (color == WHITE && square.0 > 0) || (color == BLACK && square.0 < 7) {
        west | east
    } else {
        0
    };

    let mut num_attackers = 0;
    let mut attackers = 0u64;

    let (our_bitboards, opponent_bitboards) = if color == WHITE {
        (board.white, board.black)
    } else if color == BLACK {
        (board.black, board.white)
    } else {
        panic!("wrong color dumbo");
    };

    let mut diagonal_attackers =
        diagonal_attacks & (opponent_bitboards.bishops | opponent_bitboards.queens);
    attackers |= diagonal_attackers;
    num_attackers += diagonal_attackers.count_ones();

    while diagonal_attackers != 0 {
        // add the blocker locations to the mask
        let attacker = diagonal_attackers.trailing_zeros();
        let rank = 7 - (attacker / 8) as usize;
        let file = 7 - (attacker % 8) as usize;
        let attacker_attacks = magics::BISHOP_MOVES[magics::magic_index(
            &magics::BISHOP_MAGICS[rank][file],
            board.bishop_blocker_masks[rank][file] & our_bitboards.all,
        )];

        attackers |= diagonal_attacks & attacker_attacks;
        diagonal_attackers ^= 1 << attacker;
    }

    let mut orthogonal_attackers =
        orthogonal_attacks & (opponent_bitboards.rooks | opponent_bitboards.queens);
    attackers |= orthogonal_attackers;
    num_attackers += orthogonal_attackers.count_ones();

    while orthogonal_attackers != 0 {
        // add blocker locations to mask
        let attacker = orthogonal_attackers.trailing_zeros();
        let rank = 7 - (attacker / 8) as usize;
        let file = 7 - (attacker % 8) as usize;
        let attacker_attacks = magics::ROOK_MOVES[magics::magic_index(
            &magics::ROOK_MAGICS[rank][file],
            board.rook_blocker_masks[rank][file] & our_bitboards.all,
        )];

        attackers |= orthogonal_attacks & attacker_attacks;
        orthogonal_attackers ^= 1 << attacker;
    }

    let knight_attackers = knight_attacks & opponent_bitboards.knights;
    attackers |= knight_attackers;
    num_attackers += knight_attackers.count_ones();

    let pawn_attackers = pawn_attacks & opponent_bitboards.pawns;
    attackers |= pawn_attackers;
    num_attackers += pawn_attackers.count_ones();

    let king_attackers = king_attacks & opponent_bitboards.king;
    attackers |= king_attackers;
    num_attackers += king_attackers.count_ones();

    (num_attackers as u8, attackers)
}

// use the BLACK / WHITE constants
pub fn is_in_check(board: &Board, color: u8, king_position: (u8, u8)) -> bool {
    let ortho_blocker_mask =
        board.rook_blocker_masks[king_position.0 as usize][king_position.1 as usize];
    let ortho_blockers = ortho_blocker_mask & (board.white.all | board.black.all);
    let ortho_entry = &magics::ROOK_MAGICS[king_position.0 as usize][king_position.1 as usize];
    let orthogonal_attacks = magics::ROOK_MOVES[magics::magic_index(ortho_entry, ortho_blockers)];

    let diag_blocker_mask =
        board.bishop_blocker_masks[king_position.0 as usize][king_position.1 as usize];
    let diag_blockers = diag_blocker_mask & (board.white.all | board.black.all);
    let diag_entry = &magics::BISHOP_MAGICS[king_position.0 as usize][king_position.1 as usize];
    let diagonal_attacks = magics::BISHOP_MOVES[magics::magic_index(diag_entry, diag_blockers)];

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

    let pawn_attacks =
        if (color == WHITE && king_position.0 > 0) || (color == BLACK && king_position.0 < 7) {
            west | east
        } else {
            0
        };

    if color == WHITE {
        diagonal_attacks & board.black.bishops > 0
            || diagonal_attacks & board.black.queens > 0
            || orthogonal_attacks & board.black.rooks > 0
            || orthogonal_attacks & board.black.queens > 0
            || knight_attacks & board.black.knights > 0
            || pawn_attacks & board.black.pawns > 0
            || king_attacks & board.black.king > 0
    } else if color == BLACK {
        diagonal_attacks & board.white.bishops > 0
            || diagonal_attacks & board.white.queens > 0
            || orthogonal_attacks & board.white.rooks > 0
            || orthogonal_attacks & board.white.queens > 0
            || knight_attacks & board.white.knights > 0
            || pawn_attacks & board.white.pawns > 0
            || king_attacks & board.white.king > 0
    } else {
        false
    }
}

// https://www.chessprogramming.org/X-ray_Attacks_(Bitboards)#ModifyingOccupancy
// blockers are the pieces to be xrayed through, square is the square of the xraying attacker
fn orthogonal_xrays(board: &Board, mut blockers: u64, square: (u8, u8)) -> u64 {
    let attacks = magics::ROOK_MOVES[magics::magic_index(
        &magics::ROOK_MAGICS[square.0 as usize][square.1 as usize],
        board.rook_blocker_masks[square.0 as usize][square.1 as usize]
            & (board.white.all | board.black.all),
    )];
    blockers &= attacks;
    attacks
        ^ magics::ROOK_MOVES[magics::magic_index(
            &magics::ROOK_MAGICS[square.0 as usize][square.1 as usize],
            board.rook_blocker_masks[square.0 as usize][square.1 as usize]
                & ((board.white.all | board.black.all) ^ blockers),
        )]
}

fn diagonal_xrays(board: &Board, mut blockers: u64, square: (u8, u8)) -> u64 {
    let attacks = magics::BISHOP_MOVES[magics::magic_index(
        &magics::BISHOP_MAGICS[square.0 as usize][square.1 as usize],
        board.bishop_blocker_masks[square.0 as usize][square.1 as usize]
            & (board.white.all | board.black.all),
    )];
    blockers &= attacks;
    attacks
        ^ magics::BISHOP_MOVES[magics::magic_index(
            &magics::BISHOP_MAGICS[square.0 as usize][square.1 as usize],
            board.bishop_blocker_masks[square.0 as usize][square.1 as usize]
                & ((board.white.all | board.black.all) ^ blockers),
        )]
}

// generates list of moves from a target bitboard, does not check for move legality
// does not handle promotions and en passant
fn generate_moves_from_targets(
    board: &Board,
    moves: &mut Vec<Move>,
    current_square: (u8, u8),
    piece: Piece,
    mut targets: u64,
) {
    while targets != 0 {
        let target = targets.trailing_zeros();

        moves.push(Move::new(
            current_square,
            (7 - target as u8 / 8, 7 - target as u8 % 8),
            piece,
            match board.board[7 - target as usize / 8][7 - target as usize % 8] {
                0 => None,
                piece => Some(piece),
            },
            None,
            false,
        ));

        targets ^= 1 << target;
    }
}

fn generate_knight_moves(
    board: &Board,
    moves: &mut Vec<Move>,
    current_square: (u8, u8),
    captures_only: bool,
    allowed_targets: u64,
) {
    // bitwise move generation (i hope it works)
    let mask = board.knight_masks[current_square.0 as usize][current_square.1 as usize];
    let target_mask = if captures_only {
        if board.turn {
            // only generate moves which capture opponent pieces
            board.black.all
        } else {
            board.white.all
        }
    } else if board.turn {
            !board.white.all
        } else {
            !board.black.all
    };

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | KNIGHT,
        mask & target_mask & allowed_targets,
    )
}

fn generate_king_moves(
    board: &mut Board,
    moves: &mut Vec<Move>,
    current_square: (u8, u8),
    captures_only: bool,
) {
    let mut mask = board.king_masks[current_square.0 as usize][current_square.1 as usize];
    let target_mask = if captures_only {
        if board.turn {
            // only generate moves which capture opponent pieces
            board.black.all
        } else {
            board.white.all
        }
    } else  if board.turn {
            !board.white.all
        } else {
            !board.black.all
    };

    let our_prev_bitboards = if board.turn { board.white } else { board.black };

    let our_color = if board.turn { WHITE } else { BLACK };

    let our_king_position = if board.turn {
        get_bit_index!(board.white_king_position)
    } else {
        get_bit_index!(board.black_king_position)
    };

    if board.turn {
        board.white.king = 0;
        board.white.all ^= 1 << our_king_position;
    } else {
        board.black.king = 0;
        board.black.all ^= 1 << our_king_position;
    }

    let mut new_mask = 0;
    mask &= target_mask;
    while mask != 0 {
        let target = mask.trailing_zeros() as u8;

        if !is_in_check(board, our_color, (7 - (target / 8), 7 - (target % 8))) {
            new_mask |= 1 << target;
        }

        mask ^= 1 << target;
    }

    if board.turn {
        board.white.king = our_prev_bitboards.king;
        board.white.all ^= 1 << our_king_position;
    } else {
        board.black.king = our_prev_bitboards.king;
        board.black.all ^= 1 << our_king_position;
    }

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | KING,
        new_mask,
    );

    // castling
    if board.turn {
        if is_white_kingside!(board.castle_state)
                && (board.white.all | board.black.all) & WHITE_KINGSIDE_OCCUPANCY_MASK == 0
                && !is_in_check(board, WHITE, current_square) // disallow castling when in check
                && !is_in_check(board, WHITE, (current_square.0, current_square.1 + 1))
                && !is_in_check(board, WHITE, (current_square.0, current_square.1 + 2))
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
            && !is_in_check(board, BLACK, (current_square.0, current_square.1 + 2))
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

// not promotions, only generates moves for pawns in nonpinned
fn generate_pawn_pushes(
    board: &Board,
    moves: &mut Vec<Move>,
    allowed_targets: u64,
    nonpinned: u64,
) {
    if board.turn {
        // handle promotions elsewhere
        for i in 8u8..48 {
            if (1 << i) & nonpinned == 0 {
                continue;
            }

            let mut targets: u64 = 0;
            let current_pawn: u64 = board.white.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            };

            targets |= (current_pawn << 8) & !board.black.all & !board.white.all;

            if i < 16 && targets > 0 {
                targets |= (current_pawn << 16) & !board.black.all & !board.white.all;
            }

            generate_moves_from_targets(
                board,
                moves,
                (7 - i / 8, 7 - i % 8),
                WHITE | PAWN,
                targets & allowed_targets,
            )
        }
    } else {
        // handle promotions elsewhere
        for i in 16u8..56 {
            if (1 << i) & nonpinned == 0 {
                continue;
            }

            let mut targets: u64 = 0;
            let current_pawn: u64 = board.black.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            };

            targets |= (current_pawn >> 8) & !board.white.all & !board.black.all;

            if i >= 48 && targets > 0 {
                targets |= (current_pawn >> 16) & !board.white.all & !board.black.all;
            }

            generate_moves_from_targets(
                board,
                moves,
                (7 - i / 8, 7 - i % 8),
                BLACK | PAWN,
                targets & allowed_targets,
            )
        }
    }
}

fn generate_pawn_captures(
    board: &mut Board,
    moves: &mut Vec<Move>,
    allowed_targets: u64,
    nonpinned: u64,
) {
    if board.turn {
        for i in 8u8..48 {
            if (1 << i) & nonpinned == 0 {
                continue;
            }

            let current_pawn: u64 = board.white.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            }

            let mut targets = 0;
            if let Some(square) = board.en_passant_square {
                if (7 - i % 8 > 0 && (6 - i / 8, 6 - i % 8) == square
                    || 7 - i % 8 < 7 && (6 - i / 8, 8 - i % 8) == square)
                    && (1 << get_bit_index!(square)) & allowed_targets > 0
                {
                    // make sure that there are no sneaky checks making the capture illegal
                    // en passant should be rare enough that this doesnt impact performance

                    let m = Move::new(
                        (7 - i / 8, 7 - i % 8),
                        square,
                        WHITE | PAWN,
                        Some(BLACK | PAWN),
                        None,
                        true,
                    );

                    let undo = board.make_move(m);

                    if !is_in_check(board, WHITE, board.white_king_position) {
                        moves.push(m);
                    }

                    undo(board);
                }
            }

            // capturing to the right
            targets |= ((current_pawn & NOT_A_FILE) << 7) & !board.white.all & board.black.all;
            // capturing to the left
            targets |= ((current_pawn & NOT_H_FILE) << 9) & !board.white.all & board.black.all;

            generate_moves_from_targets(
                board,
                moves,
                (7 - i / 8, 7 - i % 8),
                WHITE | PAWN,
                targets & allowed_targets,
            );
        }
    } else {
        for i in 16u8..56 {
            if (1 << i) & nonpinned == 0 {
                continue;
            }

            let current_pawn: u64 = board.black.pawns & (1 << i);

            if current_pawn == 0 {
                continue;
            }

            let mut targets = 0;
            if let Some(square) = board.en_passant_square {
                if (7 - i % 8 > 0 && (8 - i / 8, 6 - i % 8) == square
                    || 7 - i % 8 < 7 && (8 - i / 8, 8 - i % 8) == square)
                    && (1 << get_bit_index!(square)) & allowed_targets > 0
                {
                    let m = Move::new(
                        (7 - i / 8, 7 - i % 8),
                        square,
                        BLACK | PAWN,
                        Some(WHITE | PAWN),
                        None,
                        true,
                    );

                    let undo = board.make_move(m);

                    if !is_in_check(board, BLACK, board.black_king_position) {
                        moves.push(m);
                    }

                    undo(board);
                }
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
                targets & allowed_targets,
            );
        }
    }
}

fn generate_pawn_promotions(
    board: &Board,
    moves: &mut Vec<Move>,
    current_square: (u8, u8),
    allowed_targets: u64,
) {
    if board.turn && current_square.0 == 1 {
        if board.board[current_square.0 as usize - 1][current_square.1 as usize] == 0
            && (1 << get_bit_index!(current_square.0 - 1, current_square.1)) & allowed_targets > 0
        {
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
            && (1 << get_bit_index!(current_square.0 - 1, current_square.1 - 1)) & allowed_targets
                > 0
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
            && (1 << get_bit_index!(current_square.0 - 1, current_square.1 + 1)) & allowed_targets
                > 0
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
        if board.board[current_square.0 as usize + 1][current_square.1 as usize] == 0
            && (1 << get_bit_index!(current_square.0 + 1, current_square.1)) & allowed_targets > 0
        {
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
            && (1 << get_bit_index!(current_square.0 + 1, current_square.1 - 1)) & allowed_targets
                > 0
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
            && (1 << get_bit_index!(current_square.0 + 1, current_square.1 + 1)) & allowed_targets
                > 0
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

fn generate_bishop_moves(
    board: &Board,
    moves: &mut Vec<Move>,
    current_square: (u8, u8),
    captures_only: bool,
    allowed_targets: u64,
) {
    let blocker_mask =
        board.bishop_blocker_masks[current_square.0 as usize][current_square.1 as usize];
    let blockers = blocker_mask & (board.white.all | board.black.all);
    let entry = &magics::BISHOP_MAGICS[current_square.0 as usize][current_square.1 as usize];
    let legal_moves = magics::BISHOP_MOVES[magics::magic_index(entry, blockers)];

    let target_mask = if captures_only {
        if board.turn {
            // only generate moves which capture opponent pieces
            board.black.all
        } else {
            board.white.all
        }
    } else if board.turn {
            !board.white.all
        } else {
            !board.black.all
        
    };

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | BISHOP,
        legal_moves & target_mask & allowed_targets,
    )
}

fn generate_rook_moves(
    board: &Board,
    moves: &mut Vec<Move>,
    current_square: (u8, u8),
    captures_only: bool,
    allowed_targets: u64,
) {
    let blocker_mask =
        board.rook_blocker_masks[current_square.0 as usize][current_square.1 as usize];
    let blockers = blocker_mask & (board.white.all | board.black.all);
    let entry = &magics::ROOK_MAGICS[current_square.0 as usize][current_square.1 as usize];
    let legal_moves = magics::ROOK_MOVES[magics::magic_index(entry, blockers)];

    let target_mask = if captures_only {
        if board.turn {
            // only generate moves which capture opponent pieces
            board.black.all
        } else {
            board.white.all
        }
    } else if board.turn {
            !board.white.all
        } else {
            !board.black.all
    };

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | ROOK,
        legal_moves & target_mask & allowed_targets,
    )
}

fn generate_queen_moves(
    board: &Board,
    moves: &mut Vec<Move>,
    current_square: (u8, u8),
    captures_only: bool,
    allowed_targets: u64,
) {
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

    let target_mask = if captures_only {
        if board.turn {
            // only generate moves which capture opponent pieces
            board.black.all
        } else {
            board.white.all
        }
    } else if board.turn {
            !board.white.all
        } else {
            !board.black.all
        
    };

    generate_moves_from_targets(
        board,
        moves,
        current_square,
        if board.turn { WHITE } else { BLACK } | QUEEN,
        (ortho_legal_moves | diag_legal_moves) & target_mask & allowed_targets,
    );
}
