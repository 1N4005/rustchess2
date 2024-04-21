use crate::{get_bit_index, Board};

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

pub const NORTH: u8 = 0;
pub const NORTHEAST: u8 = 1;
pub const EAST: u8 = 2;
pub const SOUTHEAST: u8 = 3;
pub const SOUTH: u8 = 4;
pub const SOUTHWEST: u8 = 5;
pub const WEST: u8 = 6;
pub const NORTHWEST: u8 = 7;

impl Board {
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

    // magic bitselfs goofy ah
    pub fn generate_rook_blocker_masks(&mut self) {
        for rank in 0u8..8 {
            for file in 0u8..8 {
                let mut r = rank;
                let mut f = file;

                while r > 1 {
                    r -= 1;

                    self.rook_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }

                r = rank;

                while r < 6 {
                    r += 1;

                    self.rook_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }

                r = rank;

                while f > 1 {
                    f -= 1;

                    self.rook_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }

                f = file;

                while f < 6 {
                    f += 1;

                    self.rook_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }
            }
        }
    }

    pub fn generate_bishop_blocker_masks(&mut self) {
        for rank in 0u8..8 {
            for file in 0u8..8 {
                let mut r = rank;
                let mut f = file;

                while r > 1 && f > 1 {
                    r -= 1;
                    f -= 1;

                    self.bishop_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }

                r = rank;
                f = file;

                while r > 1 && f < 6 {
                    r -= 1;
                    f += 1;

                    self.bishop_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }

                r = rank;
                f = file;

                while r < 6 && f > 1 {
                    r += 1;
                    f -= 1;

                    self.bishop_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }

                r = rank;
                f = file;

                while r < 6 && f < 6 {
                    r += 1;
                    f += 1;

                    self.bishop_blocker_masks[rank as usize][file as usize] |=
                        1 << get_bit_index!(r, f);
                }
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

    pub fn generate_obstructed(&mut self) {
        for from in 0..64 {
            for to in 0..64 {
                let from_square = (7 - (from / 8), 7 - (from % 8));
                let to_square = (7 - (to / 8), 7 - (to % 8));

                if from != to
                    && (from_square.0 + from_square.1 == to_square.0 + to_square.1
                        || from_square.0 - from_square.1 == to_square.0 - to_square.1)
                {
                    // same diagonal
                    let mut rank = from_square.0;
                    let mut file = from_square.1;

                    let deltas = (
                        (to_square.0 - from_square.0) / i32::abs(to_square.0 - from_square.0),
                        (to_square.1 - from_square.1) / i32::abs(to_square.1 - from_square.1),
                    );

                    loop {
                        rank += deltas.0;
                        file += deltas.1;

                        if rank == to_square.0 && file == to_square.1 {
                            break;
                        }

                        self.obstructed[from as usize][to as usize] |=
                            1 << get_bit_index!(rank, file);
                    }
                }

                if from != to && (from_square.0 == to_square.0 || from_square.1 == to_square.1) {
                    // same row or column
                    let mut rank = from_square.0;
                    let mut file = from_square.1;

                    let deltas = (
                        (to_square.0 - from_square.0)
                            / i32::abs(if to_square.0 - from_square.0 == 0 {
                                1
                            } else {
                                to_square.0 - from_square.0
                            }),
                        (to_square.1 - from_square.1)
                            / i32::abs(if to_square.1 - from_square.1 == 0 {
                                1
                            } else {
                                to_square.1 - from_square.1
                            }),
                    );

                    loop {
                        rank += deltas.0;
                        file += deltas.1;

                        if rank == to_square.0 && file == to_square.1 {
                            break;
                        }

                        self.obstructed[from as usize][to as usize] |=
                            1 << get_bit_index!(rank, file);
                    }
                }
            }
        }
    }
}
