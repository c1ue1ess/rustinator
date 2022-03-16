use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Instant;

use crate::{ Board, Move, TTable, Search, TimeControl };
use crate::opening_book;

pub fn uci(author: String, bot_name: String) {
    let mut book_pos = String::new();
    let mut use_book = true;
    let mut tt = TTable::new(); 
    let mut board: Option<Board> = None;
    let mut buffer = String::new();
    
    
    // let mut filename = String::from("/home/george/Documents/progs/rustinator/");
    // filename.push_str("input");
    // filename.push_str(rand::random::<u8>().to_string().as_str());
    // filename.push_str(".txt");
    // dbg!(&filename);
    // let mut uci_input = std::fs::File::create(filename).unwrap();
    // // // std::fs::write("uci_input.txt", buffer.as_bytes()).unwrap();
    
    loop {
        io::stdin()
            .read_line(&mut buffer)
            .expect("Uci input failed");
        
        // uci_input.write(buffer.as_bytes());

        if buffer.starts_with("ucinewgame") {
            
            ucinewgame(&mut use_book, &mut tt);            
            
        } else if buffer.starts_with("uci") {
            
            uciok(&author, &bot_name);
            
        } else if buffer.starts_with("isready") {
            
            isready();
            
        } else if buffer.starts_with("position") {
            println!("here");
            board = position(String::from(&buffer), &mut tt, &mut book_pos);
            
        } else if buffer.starts_with("go") {
            
            go(board.take().unwrap(), &book_pos, &mut use_book, &mut tt);
            
        } else if buffer.starts_with("quit") {
            
            break;
            
        }

        buffer.clear();

        // io::stdout().flush().unwrap();
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
    dbg!(&board);

    let pos: Vec<&str> = buffer.trim().split(' ').collect();
    dbg!(&pos); 
    if pos.len() > 2 {
        for m in &pos[3..] {
            let mv = Move::new_from_text(m, &board);
            board.make(&mv, tt)
        }
    }        

    Some(board)
}

fn go(board: Board, book_pos: &String, mut use_book: &mut bool, tt: &mut TTable) {
    dbg!(opening_book::get_opening_move(book_pos));
    if *use_book && opening_book::get_opening_move(book_pos) {
        *use_book = true;
        return;
    } else {
        *use_book = false;
    }

    let mut search = Search::new(board, tt, TimeControl::new_now());

    let best_move = search.iterative_deepening_search();
    
    println!("bestmove {}", best_move.unwrap().as_uci_string());
}








// pub fn uci() {
//     let mut search = None;
//     let mut buffer = String::new();
//     let mut last_pos_cmd = String::new();
//     let mut use_book = true;
//     let mut tt = TTable::new();

//     io::stdin()
//         .read_line(&mut buffer)
//         .expect("First input to uci failed");

//     while !buffer.trim().contains("quit") {
//         if buffer.starts_with("ucinewgame") {
//             use_book = true;
//             tt = TTable::new();
//         } else if buffer.starts_with("uci") {
//             uciok();
//         } else if buffer.starts_with("isready") {
//             isready();
//         } else if buffer.starts_with("position") {
//             last_pos_cmd = String::from(&buffer);
//             search = Some(position(&buffer, &mut tt));
//         } else if buffer.starts_with("go") {
//             use_book = go(&last_pos_cmd.trim(), search.take(), use_book);
//         }

//         buffer.clear();

//         io::stdin()
//             .read_line(&mut buffer)
//             .expect("Uci input failed");
//     }
// }

// fn uciok() {
//     println!("id name rustinator_d\nid author George\nuciok");
// }

// fn isready() {
//     println!("readyok");
// }

// fn position<'a, 'b>(buffer: &'a str, tt: &'a mut TTable) -> Search<'a> {
//     let mut board = Board::new_with_hash(&tt);
//     let mut prev_moves: HashMap<[u64; 12], usize> = HashMap::new();

//     // add initial position to previous moves
//     let entry = prev_moves.entry(board.pieces).or_insert(0);
//     *entry += 1;

//     // make any extra moves and add them to prev_moves
//     let pos: Vec<&str> = buffer.trim().split(' ').collect();
//     if pos.len() > 2 {
//         for m in &pos[3..] {
//             board.make_no_hashing(&Move::new_from_text(&m, &board));

//             // add board to prev_moves hashmap
//             let entry = prev_moves.entry(board.pieces).or_insert(0);
//             *entry += 1;
//         }
//     }

//     // write last pos to file for debugging
//     std::fs::write("last_pos.txt", buffer.as_bytes()).unwrap();


//     let mut tc = TimeControl { start_time: Instant::now(), additional: 0 };
    

//     Search::new(board, tt, tc )
// }

// fn go(buffer: &str, s: Option<Search>, use_book: bool) -> bool {
//     if use_book {
//         if opening_book::get_opening_move(buffer) {
//             return true;
//         }
//     }

//     // let mut s = s.expect("No board found in go");
//     let best_move = s
//         .expect("No board found in go")
//         .iterative_deepening_search();

//     println!("bestmove {}", best_move.unwrap().as_uci_string());

//     false
// }
