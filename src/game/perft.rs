use super::Board;

impl Board {
    pub fn perft(&mut self, depth: u8, depth_from_root: u8) -> u128 {
        let legal_moves = self.generate_legal_moves();
        if depth == 1 {
            for m in &legal_moves {
                if depth_from_root == 0 {
                    println!("{}: 1", m.to_uci());
                }
            }
            return legal_moves.len() as u128;
        }

        let mut count: u128 = 0;

        for m in legal_moves {
            // println!("{}", self);
            let undo = self.make_move(m);
            // println!("{}", self);

            let c = self.perft(depth - 1, depth_from_root + 1);
            if depth_from_root == 0 {
                println!("{}: {}", m.to_uci(), c);
            }
            count += c;

            undo(self);
        }

        count
    }
}
