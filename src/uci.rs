use std::{io, process::exit};

use rustchess2::game::{BoardBuilder, STARTPOS, Move};

use crate::engine::{
    search::{MAX, MIN},
    Engine, PvNode,
};

pub struct UciEngine {
    pub engine: Engine,
}

impl UciEngine {
    pub fn uci() {
        let mut uciengine = UciEngine {
            engine: Engine {
                best_move: None,
                board: BoardBuilder::new().build(),
            },
        };

        println!("uciok");
        loop {
            let mut input: String = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Engine couldn't read from stdin");
            input = input.trim().to_owned();

            match &*input.split(" ").nth(0).unwrap() {
                "isready" => println!("readyok"),
                "ucinewgame" => {}
                "position" => uciengine.position_command(&input),
                "go" => uciengine.go_command(&input),
                "quit" => exit(0),
                _ => {}
            }
        }
    }

    fn position_command(&mut self, command: &String) {
        let mut builder = BoardBuilder::new();
        // ill fully implement later
        if command.contains("startpos") {
            builder.set_position(STARTPOS.to_owned());
        } else if command.contains("fen") {
            builder.set_position(command.split(" ").nth(2).unwrap().to_owned());
        }
        self.engine.board = builder.build();

        if command.contains("moves") {
            let mut found = false;
            for tok in command.split(" ") {
                if tok == "moves" {
                    found = true;
                    continue;
                }

                if found {
                    let _ = self.engine.board.make_move(Move::from_uci(tok, self.engine.board));
                }
            }
        }
    }

    fn go_command(&mut self, _command: &String) {
        // ill fully implement later
        self.engine.search(4, 0, MIN, MAX, PvNode::new(None));

        println!("bestmove {}", self.engine.best_move.unwrap().to_uci());
    }
}
