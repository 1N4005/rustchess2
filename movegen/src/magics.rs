#[derive(Debug)]
pub struct MagicEntry {
    pub magic: u64,
    pub shift: u8,
    pub offset: u64,
}

include!(concat!(env!("OUT_DIR"), "/magics.rs"));

// assumes blockers are can all be "seen" by the piece
// therefore, it is the responsibility of the caller to
// AND the attack bitboard with the blockers on the board
pub fn magic_index(entry: &MagicEntry, blockers: u64) -> usize {
    let hash = blockers.wrapping_mul(entry.magic);
    ((hash >> entry.shift) + entry.offset) as usize
}
