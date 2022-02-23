use std::collections::HashMap;
use std::io;

use crate::chess::moves::Move;
use crate::chess::Board;
use crate::search;
use crate::search::Search;
use crate::search::TTable;
use crate::uci::opening_book;

pub fn uci() {
    let mut search = None;
    let mut buffer = String::new();
    let mut last_pos_cmd = String::new();
    let mut use_book = true;
    let mut tt = TTable::new();

    io::stdin()
        .read_line(&mut buffer)
        .expect("First input to uci failed");

    while !buffer.trim().contains("quit") {
        if buffer.starts_with("ucinewgame") {
            use_book = true;
            tt = TTable::new();
        } else if buffer.starts_with("uci") {
            uciok();
        } else if buffer.starts_with("isready") {
            isready();
        } else if buffer.starts_with("position") {
            last_pos_cmd = String::from(&buffer);
            search = Some(position(&buffer, &tt));
        } else if buffer.starts_with("go") {
            use_book = go(&last_pos_cmd.trim(), search.take(), use_book, &mut tt);
        }

        buffer.clear();

        io::stdin()
            .read_line(&mut buffer)
            .expect("Uci input failed");
    }
}

fn uciok() {
    println!("id name rustinator\nid author George\nuciok");
}

fn isready() {
    println!("readyok");
}

fn position(buffer: &str, tt: &TTable) -> Search {
    let mut board = Board::new_with_hash(&tt);
    let mut prev_moves: HashMap<[u64; 12], usize> = HashMap::new();

    // add initial position to previous moves
    let entry = prev_moves.entry(board.pieces).or_insert(0);
    *entry += 1;

    // make any extra moves and add them to prev_moves
    let pos: Vec<&str> = buffer.trim().split(' ').collect();
    if pos.len() > 2 {
        for m in &pos[3..] {
            board.make_no_hashing(&Move::new_from_text(&m, &board));

            // add board to prev_moves hashmap
            let entry = prev_moves.entry(board.pieces).or_insert(0);
            *entry += 1;
        }
    }

    // write last pos to file for debugging
    //std::fs::write("last_pos.txt", buffer.as_bytes()).unwrap();

    Search::new(board, prev_moves)
}

fn go(buffer: &str, s: Option<Search>, use_book: bool, tt: &mut TTable) -> bool {
    if use_book {
        if opening_book::get_opening_move(buffer) {
            return true;
        }
    }

    // let mut s = s.expect("No board found in go");

    let best_move = s
        .expect("No board found in go")
        .iterative_deepening_search(tt);

    println!("bestmove {}", best_move.unwrap().as_uci_string());

    false
}
