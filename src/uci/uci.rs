use std::io;
use std::collections::HashMap;

use crate::chess::Board;
use crate::chess::moves::Move;
use crate::search;
use crate::search::Search;
use crate::uci::opening_book;
use crate::search::TTable;

pub fn uci() {
    let mut search = None;
    let mut buffer = String::new();
    let mut last_pos_cmd = String::new();
    let mut use_book = true;
    let mut tt = TTable::new();

    io::stdin().read_line(&mut buffer).expect("First input to uci failed");

    while !buffer.trim().contains("quit") {
        if buffer.starts_with("ucinewgame"){
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
        io::stdin().read_line(&mut buffer).expect("Uci input failed");
    }
}

fn uciok() {
    println!("id name Rustinator\nid author George\nuciok");
}

fn isready() {
    println!("readyok");
}

fn position(buffer: &str, tt: &TTable) -> Search {
    let mut prev_moves: HashMap<[u64; 12], usize> = HashMap::new();
    let mut board = Board::new_with_hash(&tt);
    let entry = prev_moves.entry(board.pieces).or_insert(0);
    *entry += 1; 
    
    let pos: Vec<&str> = buffer.trim().split(' ').collect();
    
    if pos.len() > 2 {
        for m in &pos[3..] {
            board.make_no_hashing(&Move::new_from_text(&m, &board));
            
            // add board to prev_moves hashmap
            let entry = prev_moves.entry(board.pieces).or_insert(0);
            *entry += 1;
        }
    }

    // write last post to file if needed
    // std::fs::write("last_pos.txt", buffer.as_bytes()).unwrap();

    Search { board, prev_moves }
}

fn go(buffer: &str, s: Option<Search>, use_book: bool, tt: &mut TTable) -> bool {
    let mut more_book = false;
    if use_book {
        more_book = opening_book::get_opening_move(buffer); 
    }
        
    if more_book { return true; }

    let s = s.expect("No board found in go");
    let depth = 6;
    let best_move = search::iterative_deepening_search(s, tt);
    println!("bestmove {}", best_move.unwrap().as_uci_string());

    false
}
