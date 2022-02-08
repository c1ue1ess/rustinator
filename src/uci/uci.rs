use std::io;
use std::collections::HashMap;
use std::fs;

use crate::chess::Board;
use crate::chess::moves::Move;
use crate::search;
use crate::search::Search;

pub fn uci() {
    let mut search = None;
    
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer).expect("First input to uci failed");

    while !buffer.trim().contains("quit") {
        if buffer.starts_with("ucinewgame"){
            
        } else if buffer.starts_with("uci") {
            uciok();
        } else if buffer.starts_with("isready") {
            isready();
        } else if buffer.starts_with("position") {
            search = Some(position(&buffer));
        } else if buffer.starts_with("go") {
            go(&buffer, search.take());
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
            dbg!(m);
            board.make(&Move::new_from_text(&m, &board));
            let entry = prev_moves.entry(board.pieces).or_insert(0);
            *entry += 1;
        }
    }

    //fs::write("last_pos.txt", buffer.as_bytes()).unwrap();

    Search { board, prev_moves }
}

fn go(_buffer: &str, s: Option<Search>) {
    let s = s.expect("No board found in go");
    let depth = 6;
    let best_move = search::root_search(s, depth);
    println!("bestmove {}", best_move.as_uci_string())

}

