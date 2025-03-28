const canvas = document.getElementById("board");

const pieces = new Image();
//pieces.src = "img/Chess_Pieces_Sprite.svg";
pieces.src = "img/Chess_Pieces_Sprite_alt.png";

const grid_width = canvas.clientWidth / 8;
const grid_height = canvas.clientHeight / 8;

const PAWN = 0b00100;
const BISHOP = 0b01000;
const KNIGHT = 0b01100;
const ROOK = 0b10000;
const QUEEN = 0b10100;
const KING = 0b11000;

const WHITE = 0b01;
const BLACK = 0b10;

let game_board;
let game_id = -1n;
let selected_row = -1;
let selected_col = -1;
let legal_moves = [];

function square_to_row_col(square) {
    let col = -1;
    let row = 8 - Number(square[1]);
    switch(square[0]) {
        case "a":
        col = 0;
        break;
        case "b":
        col = 1;
        break;
        case "c":
        col = 2;
        break;
        case "d":
        col = 3;
        break;
        case "e":
        col = 4;
        break;
        case "f":
        col = 5;
        break;
        case "g":
        col = 6;
        break;
        case "h":
        col = 7;
        break;
    }
    return [row, col];
}

function draw() {
    const ctx = canvas.getContext("2d");
    
    const dark = "rgb(99 79 62)";
    const light = "rgb(191 153 120)";
    const selected = "rgb(255, 255, 0)";
    const legal_move_color = "rgb(150, 150, 0)";

    document.getElementById("idbox").innerHTML = game_id;
    
    for (let r=0; r<8; r++) {
        for (let c=0; c<8; c++) {
            if (r == selected_row && c == selected_col) {
                ctx.fillStyle = selected;
            } else if ((r + c) % 2 == 0) {
                ctx.fillStyle = light;
            } else {
                ctx.fillStyle = dark;
            }

            legal_moves.forEach((move) => {
                let f = square_to_row_col(move.slice(0, 2));
                let t = square_to_row_col(move.slice(2, 4));

                if (f[0] == selected_row && f[1] == selected_col && t[0] == r && t[1] == c) {
                    ctx.fillStyle = legal_move_color;
                }
            })

            ctx.fillRect(c * grid_width, r * grid_height, (1 + c) * grid_width, (1 + r) * grid_height);

            switch (game_board[r][c]) {
                case WHITE | PAWN:
                ctx.drawImage(pieces, 225, 0, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case WHITE | BISHOP:
                ctx.drawImage(pieces, 90, 0, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case WHITE | KNIGHT:
                ctx.drawImage(pieces, 135, 0, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case WHITE | ROOK:
                ctx.drawImage(pieces, 180, 0, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case WHITE | QUEEN:
                ctx.drawImage(pieces, 45, 0, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case WHITE | KING:
                ctx.drawImage(pieces, 0, 0, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                    
                case BLACK | PAWN:
                ctx.drawImage(pieces, 225, 45, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case BLACK | BISHOP:
                ctx.drawImage(pieces, 90, 45, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case BLACK | KNIGHT:
                ctx.drawImage(pieces, 135, 45, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case BLACK | ROOK:
                ctx.drawImage(pieces, 180, 45, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case BLACK | QUEEN:
                ctx.drawImage(pieces, 45, 45, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                break;
                case BLACK | KING:
                ctx.drawImage(pieces, 0, 45, 45, 45, c * grid_width, r * grid_height, grid_width, grid_height);
                default:
            }
        }
    }
}

async function fetch_new_board() {
    const response = await fetch("/board");

    const res = await response.json();
    res[0] = BigInt(res[0]);
    console.log(res);
    return res;
}

async function fetch_board() {
    const response = await fetch("/retboard/" + game_id);
    let res = await response.json();
    console.log(res);
    return res;
}

async function fetch_legal_moves() {
    const response = await fetch("/legalmoves/" + game_id);
    let res = await response.json();
    console.log(res);
    return res;
}

async function print_legal_moves() {
    let moves = await fetch_legal_moves();
    document.getElementById("legalmoves").innerHTML = moves;
}

async function fetch_best_move() {
    const response = await fetch("/bestmove/" + game_id);
    let res = await response.json();
    console.log(res);
}

async function play_engine_move() {
    let response = await fetch("/bestmove/" + game_id);
    let res = await response.json();
    console.log(res);
    response = await fetch("/makemove/" + game_id + "/" + res);
    res = await response.json();
    console.log(res);
    game_board = res;
    draw();
}

window.addEventListener("click", async (event) => {
    console.log(event.clientX, event.clientY);
    const rect = canvas.getBoundingClientRect();
    let row_clicked = Math.floor(8 * (event.clientY - rect.top) / canvas.clientHeight);
    let col_clicked = Math.floor(8 * (event.clientX - rect.left) / canvas.clientWidth);

    legal_moves = await fetch_legal_moves(); 
    
    if (row_clicked >= 0 && row_clicked < 8 && col_clicked >= 0 && col_clicked < 8) {
        const engine_turn = document.getElementById("botcolorchoose");
        console.log(engine_turn.value);
        const engine_to_move = engine_turn.value != "none";
        const player_turn_bool = engine_turn.value == "white";
        const engine_turn_bool = !player_turn_bool;
        let player_can_move = true;

        if (engine_to_move) {
            let from = square_to_row_col(legal_moves[0].slice(0, 2));
            if (((game_board[from[0]][from[1]] & 0b11) == WHITE) == engine_turn_bool) {
                player_can_move = false;

                play_engine_move(); 
            } else {
                player_can_move = true;
            }

            if (player_can_move) {
                legal_moves.forEach(async (move) => {
                    let f = square_to_row_col(move.slice(0, 2));
                    let t = square_to_row_col(move.slice(2, 4));
                        
                    if (f[0] == selected_row && f[1] == selected_col && t[0] == row_clicked && t[1] == col_clicked) {
                        const response = await fetch("/makemove/" + game_id + "/" + move);
                        let res = await response.json();
                        console.log(res);
                        game_board = res;
                        draw();
                    }
                });
            }
        } else {
            legal_moves.forEach(async (move) => {
                let f = square_to_row_col(move.slice(0, 2));
                let t = square_to_row_col(move.slice(2, 4));
                    
                if (f[0] == selected_row && f[1] == selected_col && t[0] == row_clicked && t[1] == col_clicked) {
                    const response = await fetch("/makemove/" + game_id + "/" + move);
                    let res = await response.json();
                    console.log(res);
                    game_board = res;
                    draw();
                }
            });
        }
    }

    if (row_clicked == selected_row && col_clicked == selected_col) {
        selected_row = -1;
        selected_col = -1;
    } else {
        selected_row = row_clicked;
        selected_col = col_clicked;
    } 
    
    draw();
});

window.addEventListener("load", async () => {
    if (game_id === -1n) {
        let res = await fetch_new_board();
        game_board = res[1];
        game_id = res[0];
    } else {
        let res = await fetch_board();
        game_board = res;
    }

    draw();
});
