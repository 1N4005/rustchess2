use std::{
    io,
    process::exit,
    time::{Duration, Instant},
};

use rustchess2::game::{BoardBuilder, Move, STARTPOS};

use crate::engine::Engine;

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

        let mut i = 0;
        for tok in command.split(" ") {
            if tok == "fen" {
                self.engine.board = BoardBuilder::new()
                    .set_position(format!(
                        "{} {} {} {} {} {}",
                        command.split(" ").nth(i + 1).unwrap(),
                        command.split(" ").nth(i + 2).unwrap(),
                        command.split(" ").nth(i + 3).unwrap(),
                        command.split(" ").nth(i + 4).unwrap(),
                        command.split(" ").nth(i + 5).unwrap(),
                        command.split(" ").nth(i + 6).unwrap(),
                    ))
                    .build();
            }

            i += 1;
        }
    }

    fn go_command(&mut self, command: &String) {
        // ill fully implement later
        // self.engine.minimax(4, 0, MIN, MAX, PvNode::new(None), &mut Vec::new());
        //self.engine.negamax(6, 0, MIN, MAX);
        let tokens: Vec<&str> = command.split(" ").collect();

        let mut btime: Option<i32> = None;
        let mut wtime: Option<i32> = None;
        let mut search_depth = 6;
        for i in 0..tokens.len() {
            if tokens[i] == "btime" {
                btime = Some(tokens[i + 1].parse().expect("failed to parse"));
            }

            if tokens[i] == "wtime" {
                wtime = Some(tokens[i + 1].parse().expect("failed to parse"));
            }

            if tokens[i] == "depth" {
                search_depth = tokens[i + 1].parse().expect("failed to parse");
            }
        }

        let best_move = if let Some(b) = btime {
            if let Some(w) = wtime {
                let curr_time = Instant::now();
                let alloted_time = if self.engine.board.turn {
                    Duration::from_millis(w.try_into().unwrap()) / 40
                } else {
                    Duration::from_millis(b.try_into().unwrap()) / 40
                };

                self.engine
                    .iterative_deepening_search(100, true, curr_time, alloted_time)
            } else {
                self.engine.iterative_deepening_search(
                    6,
                    false,
                    Instant::now(),
                    Duration::from_secs(0),
                )
            }
        } else {
            self.engine.iterative_deepening_search(
                search_depth,
                false,
                Instant::now(),
                Duration::from_secs(0),
            )
        };

        println!("bestmove {}", best_move.unwrap().to_uci());
        self.engine.transposition_table.clear();
    }
}
