#![allow(unused)]
use std::sync::{ Arc, RwLock };
use std::collections::HashMap;
use std::fs;

use search::{ Search, TimeControl };
use moves::Move;
use board::Board;
use transposition_table::TTable;

mod board;
mod board_info;
mod eval;
mod move_ordering;
mod movegen;
mod moves;
mod opening_book;
mod perft;
mod search;
mod transposition_table;
mod uci;
// mod uci_mt;

fn main() {
    let author = "George";
    let bot_name = "rustinator_pawns";
    let debugger = false;

    if debugger {
        debug();
    } else {
        // uci::uci(String::from(author), String::from(bot_name));
        uci::uci(String::from(author), String::from(bot_name));
    }
}


fn debug() {


    
    let f = fs::read("target/debug/last_pos.txt").unwrap();
    let buffer = String::from_utf8_lossy(&f);
    println!("{buffer}");
    //let buffer = crate::opening_book::WHITE_OPENS[0];
    let mut tt = TTable::new();
    // let mut board = Board::new_from_fen("R7/4kp2/5N2/4P3/8/8/8/6K1 w - - 0 1");
    // board.hash = board.get_hash(&tt);

    let mut board = Board::new();
    // println!("single thread = {}", perft::perft(&mut board, 6));
    // return;

    let mut pos: Vec<&str> = buffer.trim().split(' ').collect();
    let mut pos: Vec<String> = pos.iter().map(|s| String::from(*s)).collect();
    let mut pos = pos[3..pos.len()].to_vec();
    
    if pos.len() > 2 
    {
        for m in &pos 
        {
            let mv = &Move::new_from_text(m, &board);
            board.make(mv, &tt);
        }
    }
    
    // println!("{board}");
    
    // // //board.make(&Move::new_from_text("e7e1", &board), &tt);
    // // let entry = prev_moves.entry(board.pieces).or_insert(0);
    // // *entry += 1;
    
    // // let og_hash = board.hash;

    let bestmove = Search::new(board.clone(), &mut tt, TimeControl::new_now()).iterative_deepening_search().unwrap();
    // // ntln!("\nbestmove: {}\n", bestmove.as_uci_string());
    // // assert_eq!(og_hash, board.hash);
    

    // // for m in movegen::gen_moves(&board) {
    // //     println!("{}", m.as_uci_string());
    // // }

    // // board.make(&Move::new_from_text("b1a1", &board), &tt);
    
    // // dbg!(board.prev_moves);
    // // for rep in board.prev_moves {
    // //     if rep > 
    // // }
    // dbg!(board.hash, board.prev_moves[(board.hash & 0x3FFF) as usize], board.is_bad_pos());

    // // for m in movegen::gen_moves(&board) {
    // //     board.make(&m);

    // //     println!("{}\n{}", board, m);
    // //     board.unmake(&m);
    // // }


    // //println!("single thread = {}", search::perft::perft(&mut b, depth));
    // // search::perft_multi_thread(&mut b, depth);
}