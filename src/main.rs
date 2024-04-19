mod tests;
mod uci;

use std::{error::Error, io, time::Duration};

use engine::Engine;
use rustchess2::game::{Board, BoardBuilder, Move};

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut input: String = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Engine couldn't read from stdin");

    match &*input.trim() {
        "uci" => uci::UciEngine::uci()?,
        "cli" => cli(),
        _ => println!("{} is not supported.", input),
    }

    Ok(())
}

fn cli() {
    // for debugging stuff
    let board: Board = BoardBuilder::new()
        .set_position(rustchess2::game::STARTPOS.to_owned())
        //.set_position(rustchess2::game::KIWIPETE.to_owned())
        //.set_position("8/8/8/8/8/1k1K4/2R5/8 w - - 0 1".to_owned())
        // .set_position("8/8/8/8/2R5/1k1K4/8/8 b - - 1 1".to_owned())
        // .set_position("8/8/8/8/2R5/3K4/1k6/8 w - - 2 2".to_owned())
        //.set_position("8/3k4/5B2/8/4K1R1/2RB4/8/8 w - - 0 1".to_owned())
        .build();
    let mut engine = Engine::new(board);
    loop {
        println!("{}", engine.board);
        let moves: Vec<Move>;
        measure!({
            moves = movegen::generate_legal_moves(&mut engine.board);
        });
        for m in &moves {
            print!("{} ", movegen::san::to_san(m, &mut engine.board).unwrap());
        }
        println!("\nLength: {}", moves.len());
        if moves.len() == 0 {
            break;
        }
        let best: Option<Move>;
        measure!({
            best = engine.iterative_deepening_search(
                6,
                true,
                Instant::now(),
                Duration::from_secs(1),
                None,
            );
        });
        println!(
            "Move: {} / {}",
            best.unwrap().to_uci(),
            movegen::san::to_san(&best.unwrap(), &mut engine.board).unwrap()
        );
        measure!({
            println!("Static Eval: {}", engine.evaluate());
        });

        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input: &str = &*input.trim();
        let m = Move::from_uci(input, engine.board);
        let _ = engine.board.make_move(m);
        engine.transposition_table.clear();
    }
}
