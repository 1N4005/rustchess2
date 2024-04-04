use std::{io, process::exit, time::{Instant, Duration}};

use rustchess2::game::{BoardBuilder, Move, STARTPOS};

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
            engine: Engine::new(BoardBuilder::new().build()),
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
                    let _ = self
                        .engine
                        .board
                        .make_move(Move::from_uci(tok, self.engine.board));
                }
            }
        }
    }

    fn go_command(&mut self, command: &String) {
        // ill fully implement later
        // self.engine.minimax(4, 0, MIN, MAX, PvNode::new(None), &mut Vec::new());
        //self.engine.negamax(6, 0, MIN, MAX);
        let tokens: Vec<&str> = command.split(" ").collect();

        let mut btime: Option<i32> = None;
        let mut wtime: Option<i32> = None;
        for i in 0..tokens.len() {
            if tokens[i] == "btime" {
                btime = Some(tokens[i + 1].parse().expect("failed to parse"));
            }

            if tokens[i] == "wtime" {
                wtime = Some(tokens[i + 1].parse().expect("failed to parse"));
            }
        }

        if let Some(b) = btime {
            if let Some(w) = wtime {
                let curr_time = Instant::now();
                let alloted_time = if self.engine.board.turn {
                    Duration::from_millis(w.try_into().unwrap()) / 40
                } else {
                    Duration::from_millis(b.try_into().unwrap()) / 40
                };

                self.engine.iterative_deepening_search(20, true, curr_time, alloted_time);
            } else {
                self.engine.iterative_deepening_search(6, false, Instant::now(), Duration::from_secs(0));
            }
        } else {
            self.engine.iterative_deepening_search(6, false, Instant::now(), Duration::from_secs(0));
        } 

        println!("bestmove {}", self.engine.best_move.unwrap().to_uci());
        self.engine.transposition_table.clear();
    }
}
