use std::{
    error::Error,
    fmt::Display,
    io,
    process::exit,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use game::{
    get_piece_color, get_piece_type, BoardBuilder, Move, BISHOP, BLACK, KING, KNIGHT, PAWN, QUEEN,
    ROOK, STARTPOS, WHITE,
};

use engine::Engine;

const NAME: &str = "ThinnGopher";
const AUTHOR: &str = "1ngopher";

pub struct UciEngine {
    pub engine: Engine,
}

impl UciEngine {
    pub fn uci() -> Result<(), Box<dyn Error>> {
        println!("id name {}\nid author {}", NAME, AUTHOR);
        // "fearless concurrency" lmao
        let mut uciengine: Arc<Mutex<UciEngine>> = Arc::new(Mutex::new(UciEngine {
            engine: Engine::new(BoardBuilder::new().build()),
        }));
        let mut prev_input: Option<String> = None;

        println!("uciok");
        loop {
            uciengine.lock().unwrap().engine.canceled = false;
            let (tx, rx) = mpsc::channel();
            let mut input: String = String::new();

            if let Some(input_string) = prev_input {
                input = input_string;
                prev_input = None;
            } else {
                io::stdin().read_line(&mut input)?;
            }
            input = input.trim().to_owned();

            let engine_handle = Arc::clone(&uciengine);

            match input.split(' ').nth(0).unwrap() {
                "isready" => println!("readyok"),
                "ucinewgame" => {
                    uciengine = Arc::new(Mutex::new(UciEngine {
                        engine: Engine::new(BoardBuilder::new().build()),
                    }))
                }
                "position" => uciengine.lock().unwrap().position_command(&input),
                "go" => {
                    thread::spawn(move || {
                        engine_handle.lock().unwrap().go_command(&input.clone(), rx)
                    });

                    let mut buf: String = String::new();
                    io::stdin().read_line(&mut buf)?;
                    if buf.trim() == "stop" {
                        tx.send(true)?;
                    } else {
                        prev_input = Some(buf.trim().to_owned());
                    }
                }
                "quit" => exit(0),
                "eval" => println!(
                    "Static evaluation: {}",
                    uciengine.lock().unwrap().engine.evaluate()
                ),
                "d" => println!("{}", uciengine.lock().unwrap()),
                _ => {}
            }
        }
    }

    fn position_command(&mut self, command: &str) {
        let mut builder = BoardBuilder::new();
        // ill fully implement later
        if command.contains("startpos") {
            builder.set_position(STARTPOS.to_owned());
        }
        self.engine = Engine::new(builder.build());

        for (i, tok) in command.split(' ').enumerate() {
            if tok == "fen" {
                self.engine = Engine::new(
                    BoardBuilder::new()
                        .set_position(format!(
                            "{} {} {} {} {} {}",
                            command.split(' ').nth(i + 1).unwrap(),
                            command.split(' ').nth(i + 2).unwrap(),
                            command.split(' ').nth(i + 3).unwrap(),
                            command.split(' ').nth(i + 4).unwrap(),
                            command.split(' ').nth(i + 5).unwrap(),
                            command.split(' ').nth(i + 6).unwrap(),
                        ))
                        .build(),
                );
            }
        }

        if command.contains("moves") {
            let mut found = false;
            for tok in command.split(' ') {
                if tok == "moves" {
                    found = true;
                    continue;
                }

                if found {
                    let _ = self
                        .engine
                        .board
                        .make_move(Move::from_uci(tok, self.engine.board));

                    self.engine.repetition_table.push(self.engine.board.hash);
                }
            }
        }
    }

    fn go_command(&mut self, command: &str, rx: Receiver<bool>) {
        const MOVES: u32 = 40;
        // ill fully implement later
        // self.engine.minimax(4, 0, MIN, MAX, PvNode::new(None), &mut Vec::new());
        //self.engine.negamax(6, 0, MIN, MAX);
        let tokens: Vec<&str> = command.split(' ').collect();

        let mut btime: Option<i32> = None;
        let mut wtime: Option<i32> = None;
        let mut search_depth = 100; // default depth is infinite
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

            if tokens[i] == "movetime" {
                wtime = Some(tokens[i + 1].parse::<i32>().expect("failed to parse") * MOVES as i32);
                btime = Some(tokens[i + 1].parse::<i32>().expect("failed to parse") * MOVES as i32);
            }

            if tokens[i] == "infinite" {
                search_depth = 100; //lmao
            }

            if tokens[i] == "perft" {
                movegen::perft::perft(
                    &mut self.engine.board,
                    tokens[i + 1].parse::<u8>().expect("failed to parse"),
                    0,
                );
                return;
            }
        }

        let best_move = if let Some(b) = btime {
            if let Some(w) = wtime {
                let curr_time = Instant::now();
                let alloted_time = if self.engine.board.turn {
                    Duration::from_millis(w.try_into().unwrap()) / MOVES
                } else {
                    Duration::from_millis(b.try_into().unwrap()) / MOVES
                };

                self.engine
                    .iterative_deepening_search(100, true, curr_time, alloted_time, Some(rx))
            } else {
                self.engine.iterative_deepening_search(
                    6,
                    false,
                    Instant::now(),
                    Duration::from_secs(0),
                    Some(rx),
                )
            }
        } else {
            self.engine.iterative_deepening_search(
                search_depth,
                false,
                Instant::now(),
                Duration::from_secs(0),
                Some(rx),
            )
        };

        println!("bestmove {}", best_move.unwrap().to_uci());
        self.engine.transposition_table.clear();
    }
}

impl Display for UciEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "+---+---+---+---+---+---+---+---+")?;
        for (index, rank) in self.engine.board.board.into_iter().enumerate() {
            write!(f, "|")?;
            for square in rank {
                write!(
                    f,
                    " {} |",
                    match get_piece_type!(square) {
                        PAWN => match get_piece_color!(square) {
                            WHITE => "P",
                            BLACK => "p",
                            _ => "",
                        },
                        BISHOP => match get_piece_color!(square) {
                            WHITE => "B",
                            BLACK => "b",
                            _ => "",
                        },
                        KNIGHT => match get_piece_color!(square) {
                            WHITE => "N",
                            BLACK => "n",
                            _ => "",
                        },
                        ROOK => match get_piece_color!(square) {
                            WHITE => "R",
                            BLACK => "r",
                            _ => "",
                        },
                        QUEEN => match get_piece_color!(square) {
                            WHITE => "Q",
                            BLACK => "q",
                            _ => "",
                        },
                        KING => match get_piece_color!(square) {
                            WHITE => "K",
                            BLACK => "k",
                            _ => "",
                        },
                        _ => " ",
                    }
                )?;
            }
            writeln!(f, " {}\n+---+---+---+---+---+---+---+---+", 8 - index)?;
        }
        writeln!(f, "  a   b   c   d   e   f   g   h")?;

        writeln!(
            f,
            "\n {} to move.",
            if self.engine.board.turn {
                "White"
            } else {
                "Black"
            }
        )?;

        Ok(())
    }
}
