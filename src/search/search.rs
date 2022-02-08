use std::collections::HashMap;

use crate::chess::Board;
use crate::chess::Move;
use crate::movegen;
use crate::search::eval;

pub struct Search {
    pub board: Board,
    pub prev_moves: HashMap<[u64; 12], usize>,
}

pub fn root_search(mut search: Search, depth: usize) -> Move {
    if depth == 0 {
        println!("what");
    }

    
    let moves = movegen::gen_moves(&search.board);
    let mut best_move = None;
    let mut best_move_no_tfr = None;
    let mut best_score = i32::MIN;
    let player = if search.board.colour == 0 { 1 } else { -1 };
    let mut score;
    for m in moves {
        search.board.make(&m);
        
        if movegen::check_check(&search.board, 
            &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]), &(1 - search.board.colour),) > 0 {
                search.board.unmake(&m);
                continue;
        }

        score = -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player);
    
        if score > best_score {
            best_move_no_tfr = Some(m);
            if search.prev_moves.get(&search.board.pieces).unwrap_or(&0) < &2{
                best_move = Some(m);
                best_score = score;
            }
        }
        
        search.board.unmake(&m);
    }

    match best_move {
        Some(bm) => bm,
        None => best_move_no_tfr.unwrap()
    }
}

fn negamax(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, depth: usize, player: i32) -> i32{
    if depth == 0 { 
        let eval = eval::quiesce(b, m, alpha, beta, player);
        return eval;
    }
        
    let mut score;
    let moves = movegen::gen_moves(b);
    
    let mut checkmate = true;

    for m in moves {
        b.make(&m);
        
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[11 - b.colour]), &(1 - b.colour),) > 0 {
            b.unmake(&m);
            continue;
        } else {
            checkmate = false;
        }
            
        score = -negamax(b, &m, -beta, -alpha, depth-1, -player);
        
        if score >= beta {
            b.unmake(&m);
            return beta;
        }
        
        if score > alpha {
            alpha = score;
        }
        b.unmake(&m);
    }

    if checkmate {
        -eval::CHECKMATE * depth as i32
    } else {
        alpha
    }
}