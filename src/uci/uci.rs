use std::io;
use std::collections::HashMap;

use crate::chess::Board;
use crate::chess::moves::Move;
use crate::search;
use crate::search::Search;
use crate::uci::opening_book;

pub fn uci() {
    let mut search = None;
    let mut buffer = String::new();
    let mut last_pos_cmd = String::new();
    let mut use_book = true;

    io::stdin().read_line(&mut buffer).expect("First input to uci failed");

    while !buffer.trim().contains("quit") {
        if buffer.starts_with("ucinewgame"){
            use_book = true;
        } else if buffer.starts_with("uci") {
            uciok();
        } else if buffer.starts_with("isready") {
            isready();
        } else if buffer.starts_with("position") {
            last_pos_cmd = String::from(&buffer);
            search = Some(position(&buffer));
        } else if buffer.starts_with("go") {
            use_book = go(&last_pos_cmd.trim(), search.take(), use_book);
        }
    
        buffer.clear();
        io::stdin().read_line(&mut buffer).expect("Uci input failed");
    }
}

fn uciok() {
    println!("id name Rustinator\nid author George\nuciok");
}

fn isready() {
    println!("readyok");
}

fn position(buffer: &str) -> Search {
    let mut prev_moves: HashMap<[u64; 12], usize> = HashMap::new();
    let mut board = Board::new();
    let entry = prev_moves.entry(board.pieces).or_insert(0);
    *entry += 1; 
    
    let pos: Vec<&str> = buffer.trim().split(' ').collect();
    
    if pos.len() > 2 {
        for m in &pos[3..] {
            board.make(&Move::new_from_text(&m, &board));
            
            // add board to prev_moves hashmap
            let entry = prev_moves.entry(board.pieces).or_insert(0);
            *entry += 1;
        }
    }

    // write last post to file if needed
    // std::fs::write("last_pos.txt", buffer.as_bytes()).unwrap();

    Search { board, prev_moves }
}

fn go(buffer: &str, s: Option<Search>, use_book: bool) -> bool {
    let mut more_book = false;
    if use_book {
        more_book = opening_book::get_opening_move(buffer); 
    }
        
    if more_book { return true; }

    let s = s.expect("No board found in go");
    let depth = 6;
    let best_move = search::root_search(s, depth);
    println!("bestmove {}", best_move.as_uci_string());

    false
}



#[test]
fn opening() {   
    // for i in 0..28 {
        // let b = position(&opening_book::BLACK_OPENS[24]).board;
        // println!("{i}\n{b}");
        // }
    let s = position("position startpos moves e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 f3g5 d7d5 e4d5 c6a5 c4b5 c7c6 d5c6 b7c6 b5e2 h7h6");
    go("position startpos moves e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 f3g5 d7d5 e4d5 c6a5 c4b5 c7c6 d5c6 b7c6 b5e2 h7h6", Some(s), false);
    //println!("{b}");
}