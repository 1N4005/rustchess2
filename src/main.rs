mod engine;
mod tests;
mod uci;

use std::io;

use engine::{
    search::{MAX, MIN},
    Engine,
};
use rustchess2::game::{Board, BoardBuilder, Move, BLACK, WHITE};

use crate::engine::PvNode;

macro_rules! measure {
    ($code: block) => {{
        use std::time::Instant;

        println!("====== Measuring Execution Time ======");
        let start = Instant::now();

        $code

        let elapsed = start.elapsed();
        println!("======================================");
        println!("Execution finished in {:.2?}", elapsed);
        println!("======================================");
    }};
}

fn main() {
    let mut input: String = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Engine couldn't read from stdin");

    match &*input.trim() {
        "uci" => uci::UciEngine::uci(),
        "cli" => cli(),
        _ => println!("{} is not supported.", input),
    }
}

fn cli() {
    // for debugging stuff
    let board: Board = BoardBuilder::new()
        // .set_position(rustchess2::game::STARTPOS.to_owned())
        // .set_position(rustchess2::game::KIWIPETE.to_owned())
        .set_position("r3r1k1/pppb3p/7R/4p1P1/2Pp2P1/1P3P2/P7/4RK2 b - - 2 25".to_owned())
        .build();
    // measure!({
    //     println!("{}", board.perft(5, 0))
    // });
    let mut engine = Engine::new(board);
    loop {
        // println!("{:#?}", board);
        println!("{}", engine.board);
        let moves: Vec<Move>;
        measure!({
            moves = engine.board.generate_legal_moves();
        });
        // println!("{:?}", moves);
        for m in &moves {
            print!("{} ", m.to_san(&mut engine.board).unwrap());
        }
        println!("\nLength: {}", moves.len());
        if moves.len() == 0 {
            break;
        }
        let mut result: (i32, PvNode) = (0, PvNode::new(None));
        measure!({
            result = engine.search(4, 0, MIN, MAX, PvNode::new(None));
        });
        println!(
            "Eval: {}, Move: {} / {}",
            result.0,
            engine.best_move.unwrap().to_uci(),
            engine.best_move.unwrap().to_san(&mut engine.board).unwrap()
        );

        let mut pv = result.1;
        while match pv.next {
            Some(_) => true,
            None => false,
        } {
            print!("{} ", pv.best_move.unwrap().to_uci());

            pv = *pv.next.unwrap();
        }
        println!();
        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input: &str = &*input.trim();
        let m = Move::from_uci(input, engine.board);
        let _ = engine.board.make_move(m);
    }

    if board.is_in_check(
        if engine.board.turn { WHITE } else { BLACK },
        if engine.board.turn {
            engine.board.white_king_position
        } else {
            engine.board.black_king_position
        },
    ) {
        if engine.board.turn {
            println!("0-1");
        } else {
            println!("1-0");
        }
    } else {
        println!("1/2-1/2");
    }
}
