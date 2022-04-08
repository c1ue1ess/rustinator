use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Instant;

use crate::{ Board, Move, TTable, Search, TimeControl };
use crate::opening_book::{self, Book};

pub fn uci(author: String, bot_name: String) {
    let mut book_pos = String::new();
    let mut use_book = true;
    let mut tt = TTable::new(); 
    let mut board: Option<Board> = None;
    let mut buffer = String::new();
    let book = Book::new();
    
    loop {
        io::stdin()
            .read_line(&mut buffer)
            .expect("Uci input failed");

        if buffer.starts_with("ucinewgame") {
            
            ucinewgame(&mut use_book, &mut tt);            
            
        } else if buffer.starts_with("uci") {
            
            uciok(&author, &bot_name);
            
        } else if buffer.starts_with("isready") {
            
            isready();
            
        } else if buffer.starts_with("position") {
            //println!("here");
            board = position(String::from(&buffer), &mut tt, &mut book_pos);
            
        } else if buffer.starts_with("go") {
            
            go(board.take().unwrap(), &book_pos, &mut use_book, &mut tt, &book);
        
        } else if buffer.starts_with("quit") {
            
            break;
            
        }

        buffer.clear();
    }
}

fn ucinewgame(use_book: &mut bool, tt: &mut TTable) {
    *use_book = true;
    *tt = TTable::new();
}

fn uciok(author: &str, bot_name: &str) {
    println!("id name {}\nid author {}\nuciok", bot_name, author); 
}

fn isready() {
    println!("readyok");
}

fn position(buffer: String, tt: &mut TTable, book_pos: &mut String) -> Option<Board> {
    *book_pos = buffer.trim().to_string();
    
    let mut board = Board::new_with_hash(tt);

    let pos: Vec<&str> = buffer.trim().split(' ').collect();
    if pos.len() > 2 {
        for m in &pos[3..] {
            let mv = Move::new_from_text(m, &board);
            board.make(&mv, tt)
        }
    }        

    // write last pos to file for debugging if crashes
    //std::fs::write("last_pos.txt", buffer.as_bytes()).unwrap();

    Some(board)
}

fn go(board: Board, book_pos: &str, mut use_book: &mut bool, tt: &mut TTable, book: &Book) {
    if *use_book && book.get_opening_move(book_pos) {
        *use_book = true;
        return;
    } else {
        *use_book = false;
    }

    let mut search = Search::new(board, tt, TimeControl::new_now());

    let best_move = search.iterative_deepening_search();
    
    println!("bestmove {}", best_move.unwrap().as_uci_string());
}
