#![allow(unused)]
use std::collections::HashMap;
use threadpool::ThreadPool;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, Duration};

use crate::chess::Board;
use crate::chess::Move;
use crate::chess::movegen;
use crate::search::eval;
use crate::search::{ TTable, TEntry, NodeType };

const TIME_LIM_SEC: u64 = 3;
const MAX_SEARCH_DEPTH: usize = 20;

pub struct Search {
    pub board: Board,
    pub prev_moves: HashMap<[u64; 12], usize>,
}

pub fn iterative_deepening_search(mut search: Search, tt: &mut TTable) -> Option<Move>{
    let start_time = Instant::now();    
    let mut best_score = i32::MIN;
    let mut best_move: Option<Move> = None;
    
    let mut depth = 0;
    
    for depth in 1..MAX_SEARCH_DEPTH {
        if Instant::now().duration_since(start_time) >= Duration::from_secs(TIME_LIM_SEC) {
            break;
        }
        
        let (curr_score, curr_move) = root_search(&mut search, depth, &start_time, tt);
        
        if curr_score > best_score {
            best_score = curr_score;
            best_move = curr_move;
        }
    }

    best_move
}

pub fn root_search(search: &mut Search, depth: usize, start_time: &Instant, tt: &mut TTable) -> (i32, Option<Move>) {
    let mut best_move = None;
    let mut best_score = i32::MIN;
    let player = if search.board.colour == 0 { 1 } else { -1 };
    
    let mut score;
    let moves = movegen::gen_moves(&search.board);
    for m in moves {
        if Instant::now().duration_since(*start_time) >= Duration::from_secs(TIME_LIM_SEC) {
            return (best_score, best_move)
        }
        
        search.board.make(&m, tt);
        
        if movegen::check_check(&search.board, 
            &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]), &(1 - search.board.colour),) > 0 {
                search.board.unmake(&m, tt);
                continue;
        }

        score = -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player, tt, &start_time);
    
        if score > best_score {
            if search.prev_moves.get(&search.board.pieces).unwrap_or(&0) < &2{
                best_move = Some(m);
                best_score = score;
            }
        }
        
        search.board.unmake(&m, tt);
    }

    (best_score, best_move)
    
}

fn negamax(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, 
    depth: usize, player: i32, tt: &mut TTable, start_time: &Instant) -> i32{
    //dbg!(b.hash);
    
    if let Some(hash_score) = tt.get(b.hash, depth as u8, alpha, beta) {
        // account for the change in depth of checkmate within transposition table
        if hash_score <= -eval::CHECKMATE {
            let check_depth = hash_score / -eval::CHECKMATE;
            return -eval::CHECKMATE * (check_depth - depth as i32);
        } else {
            return hash_score;
        }
    } 
    
    if depth == 0 { 
        //dbg!(depth);
        let eval = eval::quiesce(b, m, alpha, beta, player);
        tt.insert(TEntry::new(b.hash, 0, eval, NodeType::Pv));
        return eval;
    }
    
    let mut score;
    let moves = movegen::gen_moves(b);
    
    let mut no_moves = true;
    let mut node_type = NodeType::Alpha;
    
    for m in moves {
        if Instant::now().duration_since(*start_time) >= Duration::from_secs(TIME_LIM_SEC) {
            break;
        }

        b.make(&m, tt);
        
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[11 - b.colour]), &(1 - b.colour),) > 0 {
            b.unmake(&m, tt);
            continue;
        } else {
            no_moves = false;
        }

        score = -negamax(b, &m, -beta, -alpha, depth-1, -player, tt, start_time);
        b.unmake(&m, tt);
        
        if score >= beta {
            tt.insert(TEntry::new(b.hash, depth as u8, beta, NodeType::Beta));
            return beta;
        }
        
        if score > alpha {
            node_type = NodeType::Pv;
            alpha = score;
        }
    }
    
    if no_moves {
        // if checkmate/stalemate
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[10 + b.colour]), &(b.colour)) > 0 {
            //println!("info string checkmate found at {}", depth);
            tt.insert(TEntry::new(b.hash, depth as u8, -eval::CHECKMATE * depth as i32, NodeType::Pv));
            -eval::CHECKMATE * depth as i32
        } else {
            tt.insert(TEntry::new(b.hash, depth as u8, 0, NodeType::Alpha));
            //println!("info string stalemate found at {}", depth);
            0
        }
    } else {
        tt.insert(TEntry::new(b.hash, depth as u8, alpha, node_type));
        alpha
    }
}

// WORK IN PROGRESS
pub fn root_search_mt(mut search: Search, depth: usize) -> Move {
    if depth == 0 {
        println!("what");
    }

    let moves = movegen::gen_moves(&search.board);
    let pool = ThreadPool::new(moves.len());
    
    let player = if search.board.colour == 0 { 1 } else { -1 };
    let mvs: Arc<Mutex<(Option<Move>, i32, Board)>> = Arc::new(Mutex::new((None, i32::MIN, search.board)));
    let prev_moves = Arc::new(RwLock::new(search.prev_moves));
    
    let moves = moves;

    for m in moves {
        let mv = Arc::clone(&mvs);
        let prev_m = Arc::clone(&prev_moves);

        pool.execute(move || {
            search.board.make_no_hashing(&m);
            
            if movegen::check_check(&search.board, &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]), 
                &(1 - search.board.colour),) > 0 {
                    search.board.unmake_no_hashing(&m);
                    return;
                }
                
            let score =0;// -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player);
            
            let mut best_m = mv.lock().unwrap();
        
            if score > best_m.1 && *prev_m.read().unwrap().get(&search.board.pieces).unwrap_or(&0) < 2 {
                best_m.0 = Some(m);
                best_m.1 = score;
            }
            
            search.board.unmake_no_hashing(&m);
        });
    }
    
    pool.join();
    let best_move = mvs.lock().unwrap().0.unwrap();
    
    best_move
}
    
pub fn root_search_mtpvs(mut search: Search, depth: usize) -> Move {
    if depth == 0 {
        println!("what");
    }

    let moves = movegen::gen_moves(&search.board);
    let pool = ThreadPool::new(moves.len());
    


    let player = if search.board.colour == 0 { 1 } else { -1 };
    let mvs: Arc<Mutex<(Option<Move>, i32, Board)>> = Arc::new(Mutex::new((None, i32::MIN, search.board)));
    let prev_moves = Arc::new(RwLock::new(search.prev_moves));
    
    let moves = moves;
    //let pv = search_pv(&mut search.board, &moves[0], i32::MIN+1, i32::MAX, depth, player);

    for m in moves {
        let mv = Arc::clone(&mvs);
        let prev_m = Arc::clone(&prev_moves);

        pool.execute(move || {
            search.board.make_no_hashing(&m);
            
            if movegen::check_check(&search.board, 
                &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]), &(1 - search.board.colour),) > 0 {
                    search.board.unmake_no_hashing(&m);
                    return;
                }
                
                let score =0;// -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player);
                
                let mut best_m = mv.lock().unwrap();
                if score > best_m.1 && *prev_m.read().unwrap().get(&search.board.pieces).unwrap_or(&0) < 2 {
                    best_m.0 = Some(m);
                    best_m.1 = score;
                }
                search.board.unmake_no_hashing(&m);
            });
        }
        
    pool.join();
    let best_move = mvs.lock().unwrap().0.unwrap();

    best_move
}
        
fn search_pv(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, depth: usize, player: i32) -> i32{
    if depth == 0 { 
        let eval = eval::quiesce(b, m, alpha, beta, player);
        return eval;
    }
    
    let moves = movegen::gen_moves(b);
    let mut checkmate = true;

    for m in moves {
        b.make_no_hashing(&m);
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[11 - b.colour]), &(1 - b.colour),) > 0 {
            b.unmake_no_hashing(&m);
            continue;
        } else {
            checkmate = false;
        }
            
        let score = -search_pv(b, &m, -beta, -alpha, depth-1, -player);
        
        if score >= beta {
            b.unmake_no_hashing(&m);
            return beta;
        }
        
        if score > alpha {
            alpha = score;
        }
        b.unmake_no_hashing(&m);
        
        if checkmate {
            return -eval::CHECKMATE * depth as i32
        } else {
            return alpha
        }
    }

    if checkmate {
        -eval::CHECKMATE * depth as i32
    } else {
        alpha
    }
}

fn pvs(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, depth: usize, player: i32) -> i32{
    if depth == 0 { 
        let eval = eval::quiesce(b, m, alpha, beta, player);
        return eval;
    }
    
    let moves = movegen::gen_moves(b);
    let mut checkmate = true;

    for m in moves {
        b.make_no_hashing(&m);
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[11 - b.colour]), &(1 - b.colour),) > 0 {
            b.unmake_no_hashing(&m);
            continue;
        } else {
            checkmate = false;
        }
            
        let score = -pvs(b, &m, -beta, -alpha, depth-1, -player);
        
        if score >= beta {
            b.unmake_no_hashing(&m);
            return beta;
        }
        
        if score > alpha {
            alpha = score;
        }
        b.unmake_no_hashing(&m);
    }

    if checkmate {
        -eval::CHECKMATE * depth as i32
    } else {
        alpha
    }
}