use crate::get_piece_type;

// standard algebraic notation
use super::{Board, Move, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};
// does not modify board, however generate_legal_moves requrires a mutable board
// this function is probably expensive to call
// also why is algebraic notation so stupid
pub fn to_san(m: &Move, board: &mut Board) -> Result<String, String> {
    let mut s = String::new();

    s += match get_piece_type!(m.piece) {
        PAWN => match m.capture_piece {
            Some(_) => ["a", "b", "c", "d", "e", "f", "g", "h"][m.from.1 as usize],
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

    for mv in super::generate_legal_moves(board, false) {
        if m.to == mv.to && m.piece == mv.piece {
            possible_origins.push(mv);
        }
    }

    if possible_origins.len() == 2 {
        if possible_origins[0].from.1 != possible_origins[1].from.1 {
            s += ["a", "b", "c", "d", "e", "f", "g", "h"][m.from.1 as usize];
        } else {
            s += ["8", "7", "6", "5", "4", "3", "2", "1"][m.to.0 as usize];
        }
    }

    s += match m.capture_piece {
        Some(_) => "x",
        None => "",
    };

    s += ["a", "b", "c", "d", "e", "f", "g", "h"][m.to.1 as usize];
    s += ["8", "7", "6", "5", "4", "3", "2", "1"][m.to.0 as usize];

    let undo = board.make_move(*m);

    if super::is_in_check(
        board,
        if board.turn { WHITE } else { BLACK },
        if board.turn {
            board.white_king_position
        } else {
            board.black_king_position
        },
    ) {
        s += if super::generate_legal_moves(board, false).is_empty() {
            "#"
        } else {
            "+"
        }
    }

    undo(board);

    Ok(s)
}
