use std::fmt::Display;
use std::fs::{self};
use std::io::Write;

use game::BoardBuilder;
use game::{
    precomputed::{EAST, NORTH, NORTHEAST, NORTHWEST, SOUTH, SOUTHEAST, SOUTHWEST, WEST},
    rand, Board,
};

#[derive(Debug, Clone, Copy)]
pub struct MagicEntry {
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

pub enum SlidingPieces {
    Bishop,
    Rook,
}

impl Display for SlidingPieces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SlidingPieces::Bishop => "Bishop",
                SlidingPieces::Rook => "Rook",
            }
        )?;
        Ok(())
    }
}

fn get_ray_attacks(
    board: &Board,
    current_square: (u8, u8),
    direction: u8,
    mut blockers: u64,
) -> u64 {
    let mut attacks =
        board.ray_attacks[direction as usize][current_square.0 as usize][current_square.1 as usize];
    blockers = attacks & blockers;

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

pub fn find_magic(
    board: &Board,
    piece: SlidingPieces,
    square: (u8, u8),
    index_bits: u8,
) -> (Vec<u64>, MagicEntry) {
    let shift = 64 - index_bits;

    loop {
        let magic = rand::random() & rand::random() & rand::random();
        let entry = MagicEntry {
            magic,
            shift,
            offset: 0u32,
        };
        if let Ok(table) = try_magic(board, square, &piece, &entry) {
            return (table, entry);
        }
    }
}

// returns array for looking up legal moves
fn try_magic(
    board: &Board,
    square: (u8, u8),
    piece: &SlidingPieces,
    entry: &MagicEntry,
) -> Result<Vec<u64>, ()> {
    let mut blockers = 0;
    let mask = match piece {
        SlidingPieces::Bishop => board.bishop_blocker_masks[square.0 as usize][square.1 as usize],
        SlidingPieces::Rook => board.rook_blocker_masks[square.0 as usize][square.1 as usize],
    };
    let index_bits: u8 = 64 - entry.shift;
    let mut table = vec![0; 1 << index_bits];
    loop {
        let moves = match piece {
            SlidingPieces::Bishop => {
                get_ray_attacks(board, square, NORTHEAST, blockers)
                    | get_ray_attacks(board, square, NORTHWEST, blockers)
                    | get_ray_attacks(board, square, SOUTHEAST, blockers)
                    | get_ray_attacks(board, square, SOUTHWEST, blockers)
            }
            SlidingPieces::Rook => {
                get_ray_attacks(board, square, NORTH, blockers)
                    | get_ray_attacks(board, square, EAST, blockers)
                    | get_ray_attacks(board, square, SOUTH, blockers)
                    | get_ray_attacks(board, square, WEST, blockers)
            }
        };

        if table[get_magic_index(entry, blockers)] == 0 {
            table[get_magic_index(entry, blockers)] = moves;
        } else if table[get_magic_index(entry, blockers)] != moves {
            return Err(());
        }

        blockers = blockers.wrapping_sub(mask) & mask;
        if blockers == 0 {
            break;
        }
    }
    Ok(table)
}

// does not calculate offset, offset is calculated when writing magics to file
fn get_magic_index(entry: &MagicEntry, blockers: u64) -> usize {
    let hash = blockers.wrapping_mul(entry.magic);
    (hash >> entry.shift) as usize
}

pub fn format_moves_table(table: [[Vec<u64>; 8]; 8]) -> String {
    let mut ret: String = String::new();

    ret += "[";
    for rank in table {
        ret += "[";
        for file in rank {
            ret += &format!("vec!{:?},", file);
        }
        ret += "],";
    }
    ret += "]";

    ret
}

pub fn write_magics_to_file(filename: String) {
    let mut out_file = if let Ok(file) = fs::File::create(&filename) {
        file
    } else {
        return;
    };
    let mut rook_moves: [[Vec<u64>; 8]; 8] = Default::default();
    let mut bishop_moves: [[Vec<u64>; 8]; 8] = Default::default();
    let mut rook_magics: [[MagicEntry; 8]; 8] = [[MagicEntry {
        magic: 0,
        shift: 0,
        offset: 0,
    }; 8]; 8];
    let mut bishop_magics: [[MagicEntry; 8]; 8] = [[MagicEntry {
        magic: 0,
        shift: 0,
        offset: 0,
    }; 8]; 8];
    let mut _rook_table_size = 0usize;
    let mut _bishop_table_size = 0usize;
    let board = BoardBuilder::new().build();

    for rank in 0u8..8 {
        for file in 0u8..8 {
            let index_bits: u8 =
                board.rook_blocker_masks[rank as usize][file as usize].count_ones() as u8;
            (
                rook_moves[rank as usize][file as usize],
                rook_magics[rank as usize][file as usize],
            ) = find_magic(&board, SlidingPieces::Rook, (rank, file), index_bits);
            _rook_table_size += rook_moves[rank as usize][file as usize].len();
        }
    }

    for rank in 0u8..8 {
        for file in 0u8..8 {
            let index_bits =
                board.bishop_blocker_masks[rank as usize][file as usize].count_ones() as u8;
            (
                bishop_moves[rank as usize][file as usize],
                bishop_magics[rank as usize][file as usize],
            ) = find_magic(&board, SlidingPieces::Bishop, (rank, file), index_bits);
            _bishop_table_size += bishop_moves[rank as usize][file as usize].len();
        }
    }

    // write flattened version of rook and bishop moves
    let mut current_offset = 0u32;
    let _ = out_file.write(b"pub const ROOK_MOVES: &[u64] = &[");
    for rank in 0..8usize {
        for file in 0..8usize {
            rook_magics[rank][file].offset = current_offset;
            for move_bb in &rook_moves[rank][file] {
                current_offset += 1;
                let _ = out_file.write(format!("{},", move_bb).as_bytes());
            }
        }
    }
    current_offset = 0;
    let _ = out_file.write(b"];\npub const BISHOP_MOVES: &[u64] = &[");
    for rank in 0..8usize {
        for file in 0..8usize {
            bishop_magics[rank][file].offset = current_offset;
            for move_bb in &bishop_moves[rank][file] {
                current_offset += 1;
                let _ = out_file.write(format!("{},", move_bb).as_bytes());
            }
        }
    }
    let _ =
        out_file.write(format!("];\npub const ROOK_MAGICS: [[MagicEntry; 8]; 8] = [").as_bytes());
    for rank in 0..8usize {
        let _ = out_file.write(b"[");
        for file in 0..8usize {
            let _ = out_file.write(format!("{:?},", rook_magics[rank][file]).as_bytes());
        }
        let _ = out_file.write(b"],");
    }
    let _ =
        out_file.write(format!("];\npub const BISHOP_MAGICS: [[MagicEntry; 8]; 8] = [").as_bytes());
    for rank in 0..8usize {
        let _ = out_file.write(b"[");
        for file in 0..8usize {
            let _ = out_file.write(format!("{:?},", bishop_magics[rank][file]).as_bytes());
        }
        let _ = out_file.write(b"],");
    }
    let _ = out_file.write(b"];");
}
