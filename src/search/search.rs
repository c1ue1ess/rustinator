use std::collections::HashMap;
use threadpool::ThreadPool;
use std::sync::{Arc, Mutex, RwLock};


use crate::chess::Board;
use crate::chess::Move;
use crate::chess::movegen;
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

// WORK IN PROGRESS
#[allow(dead_code)]
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
            search.board.make(&m);
            
            if movegen::check_check(&search.board, &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]), 
                &(1 - search.board.colour),) > 0 {
                    search.board.unmake(&m);
                    return;
                }
                
            let score = -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player);
            
            let mut best_m = mv.lock().unwrap();
        
            if score > best_m.1 && *prev_m.read().unwrap().get(&search.board.pieces).unwrap_or(&0) < 2 {
                best_m.0 = Some(m);
                best_m.1 = score;
            }
            
            search.board.unmake(&m);
        });
    }
    
    pool.join();
    let best_move = mvs.lock().unwrap().0.unwrap();
    
    best_move
}
    



#[allow(dead_code)]
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
            search.board.make(&m);
            
            if movegen::check_check(&search.board, 
                &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]), &(1 - search.board.colour),) > 0 {
                    search.board.unmake(&m);
                    return;
                }
                
                let score = -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player);
                
                let mut best_m = mv.lock().unwrap();
                if score > best_m.1 && *prev_m.read().unwrap().get(&search.board.pieces).unwrap_or(&0) < 2 {
                    best_m.0 = Some(m);
                    best_m.1 = score;
                }
                search.board.unmake(&m);
            });
        }
        
    pool.join();
    let best_move = mvs.lock().unwrap().0.unwrap();

    best_move
}
        
    

#[allow(dead_code)]
fn search_pv(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, depth: usize, player: i32) -> i32{
    if depth == 0 { 
        let eval = eval::quiesce(b, m, alpha, beta, player);
        return eval;
    }
    
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
            
        let score = -search_pv(b, &m, -beta, -alpha, depth-1, -player);
        
        if score >= beta {
            b.unmake(&m);
            return beta;
        }
        
        if score > alpha {
            alpha = score;
        }
        b.unmake(&m);
        
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

#[allow(dead_code)]
fn pvs(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, depth: usize, player: i32) -> i32{
    if depth == 0 { 
        let eval = eval::quiesce(b, m, alpha, beta, player);
        return eval;
    }
    
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
            
        let score = -pvs(b, &m, -beta, -alpha, depth-1, -player);
        
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