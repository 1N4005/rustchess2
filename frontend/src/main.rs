use std::{collections::HashMap, sync::Mutex, time::{Duration, Instant}};

use engine::Engine;
use game::{rand, Board, BoardBuilder, Move, Piece, STARTPOS};
use movegen::generate_legal_moves;
use rocket::{fs::FileServer, response::Redirect, serde::json::Json, State};

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/index.html")
}

#[get("/board")]
fn board(active_boards: &State<Mutex<HashMap<u64, Board>>>) -> Json<(String, [[Piece; 8]; 8])> {
    let b = BoardBuilder::new().set_position(STARTPOS.to_string()).build();
    let ret = b.board;
    let id = rand::random();
    active_boards.lock().unwrap().insert(id, b);
    println!("new board id {}\n{}", id, b);
    let idstr = id.to_string();
    Json((idstr, ret))
}

#[get("/retboard/<id>")]
fn retboard(id: u64, active_boards: &State<Mutex<HashMap<u64, Board>>>) -> Json<[[Piece; 8]; 8]> {
    Json(active_boards.lock().unwrap().get(&id).unwrap().board)
}

#[get("/legalmoves/<id>")]
fn legalmoves(id: u64, active_boards: &State<Mutex<HashMap<u64, Board>>>) -> Json<Vec<String>> {
    let mut b = active_boards.lock().unwrap().get(&id).unwrap().clone();
    let moves = generate_legal_moves(&mut b, false);
    let mut moves_vec = Vec::new();
    for m in moves {
        moves_vec.push(m.to_uci());
    }

    Json(moves_vec)
}

#[get("/makemove/<id>/<uci>")]
fn makemove(id: u64, uci: String, active_boards: &State<Mutex<HashMap<u64, Board>>>) -> Json<[[Piece; 8]; 8]> {
    let mut b = active_boards.lock().unwrap().get(&id).unwrap().clone();
    let _ = b.make_move(Move::from_uci(&uci, b));
    active_boards.lock().unwrap().insert(id, b);

    Json(b.board)
}

#[get("/removegame/<id>")]
fn removegame(id: u64, active_boards: &State<Mutex<HashMap<u64, Board>>>) {
    active_boards.lock().unwrap().remove(&id);
    println!("removed game {}", id);
}

#[get("/turn/<id>")]
fn turn(id: u64, active_boards: &State<Mutex<HashMap<u64, Board>>>) -> Json<bool> {
    Json(active_boards.lock().unwrap().get(&id).unwrap().turn)
}

#[get("/bestmove/<id>")]
fn bestmove(id: u64, active_boards: &State<Mutex<HashMap<u64, Board>>>) -> Json<String> {
    let b = active_boards.lock().unwrap().get(&id).unwrap().clone();
    let mut engine = Engine::new(b);
    engine.iterative_deepening_search(200, true, Instant::now(), Duration::from_millis(1000), None);
    Json(engine.best_move.unwrap().to_uci())
}

#[launch]
fn rocket() -> _ {
    let active_boards: Mutex<HashMap<u64, Board>> = Mutex::new(HashMap::new());
    rocket::build()
        .mount("/", FileServer::from("./static"))
        .mount("/", routes![index, board, retboard, legalmoves, makemove, removegame, turn, bestmove])
        .manage(active_boards)
}
