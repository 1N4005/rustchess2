mod uci;

use std::{error::Error, io};

use engine::Engine;
use movegen::generate_legal_moves;
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

    match input.trim() {
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
        // .set_position("r1b1kbnr/ppq1pppp/2n5/2p1P3/8/2BP1N2/PPPQBPPP/R3K2R b KQkq - 2 10".to_owned())
        // .set_position("1k6/4q3/8/8/8/2Q2R2/1r2K3/8 w - - 0 1".to_owned())
        //.set_position("6Q1/1k3N2/8/3b4/2R5/1N3B2/8/2K5 w - - 0 1".to_owned())
        //.set_position("rnb1k1nr/pppp1ppp/8/4P3/Pb6/2P5/1P2PqPP/RNBQKBNR w - - 0 1".to_owned())
        //.set_position("rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 0 1".to_owned())
        .build();
    let mut engine = Engine::new(board);
    loop {
        println!("{}", engine.board);

        let moves = generate_legal_moves(&mut engine.board, false);
        print!("legal moves: ");
        for m in moves {
            print!("{} ", m.to_uci());
        }
        println!();
        measure!({
            println!("Static Eval: {}", engine.evaluate());
        });

        let mut input: String = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input: &str = input.trim();
        let m = Move::from_uci(input, engine.board);
        let _ = engine.board.make_move(m);
        engine.transposition_table.clear();
    }
}
