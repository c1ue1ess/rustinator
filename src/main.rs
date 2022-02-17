#![allow(unused)]
use std::collections::HashMap;
use std::fs;

use crate::chess::{ Move, Board };
use crate::search::{ TTable };

mod chess;
mod search;
mod uci;

fn main() {
    let debugger = false;
    
    if debugger {
        debug();
    } else {
        uci::uci();
    }
}


fn debug() {
    let mut b = Board::new(); 
    println!("single thread = {}", search::perft::perft(&mut b, 6));
    return;
    // let b = crate::chess::Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    // dbg!(b);
    // println!("{}", b);

    // let f = fs::read("target/debug/last_pos.txt").unwrap();
    let buffer = crate::uci::WHITE_OPENS[0];
    // let buffer = String::from_utf8_lossy(&f);


    let mut prev_moves: HashMap<[u64; 12], usize> = HashMap::new();
    let mut board = Board::new();
    let mut tt = TTable::new();
    let entry = prev_moves.entry(board.pieces).or_insert(0);
    *entry += 1; 
    
    let pos: Vec<&str> = buffer.trim().split(' ').collect();
    
    if pos.len() > 2 {
        for m in &pos[3..] {
            dbg!(m);
            let mv = &Move::new_from_text(&m, &board);
            board.make(mv, &tt);
            println!("{board}\n{mv}");
            let entry = prev_moves.entry(board.pieces).or_insert(0);
            *entry += 1;
        }
    }

    
    search::iterative_deepening_search(search::Search { board, prev_moves }, &mut tt);

    // for m in movegen::gen_moves(&board) {
    //     board.make(&m);

    //     println!("{}\n{}", board, m);
    //     board.unmake(&m);
    // }


    //println!("single thread = {}", search::perft::perft(&mut b, depth));
    // search::perft_multi_thread(&mut b, depth);
}