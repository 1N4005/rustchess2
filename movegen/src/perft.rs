use super::Board;

pub fn perft(board: &mut Board, depth: u8, depth_from_root: u8) -> u128 {
    let legal_moves = super::generate_legal_moves(board, false);
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
        // println!("{}", board);
        let undo = board.make_move(m);
        // println!("{}", board);

        let c = perft(board, depth - 1, depth_from_root + 1);
        if depth_from_root == 0 {
            println!("{}: {}", m.to_uci(), c);
        }
        count += c;

        undo(board);
    }

    if depth_from_root == 0 {
        println!("Nodes Searched: {}", count);
    }

    count
}
