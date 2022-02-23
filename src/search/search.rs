#![allow(unused)]
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use threadpool::ThreadPool;

use crate::chess::movegen;
use crate::chess::Board;
use crate::chess::Move;
use crate::search::eval;
use crate::search::{NodeType, TEntry, TTable};

const TIME_LIM: u64 = 5;
// time limit the iterative deepening search will start searches untill
const BEGIN_SEARCH_TIMEOUT: u64 = 5;
// time limit the negamax + root search functions will run untill
const END_SEARCH_TIMEOUT: u64 = 15;
const MAX_SEARCH_DEPTH: usize = 20;

pub struct Search {
    pub board: Board,
    pub prev_moves: HashMap<[u64; 12], usize>,
}

impl Search {
    pub fn new(board: Board, prev_moves: HashMap<[u64; 12], usize>) -> Search {
        Search { board, prev_moves }
    }

    pub fn iterative_deepening_search(&mut self, tt: &mut TTable) -> Option<Move> {
        let mut best_score = i32::MIN;
        let mut best_move: Option<Move> = None;

        let mut iter_score = i32::MIN;
        let mut iter_move: Option<Move> = None;

        let start_time = Instant::now();
        for depth in 1..MAX_SEARCH_DEPTH {
            let mut iter_score = i32::MIN;

            let (curr_score, curr_move) = self.root_search(best_move, depth, &start_time, tt);

            if Instant::now().duration_since(start_time) >= Duration::from_secs(TIME_LIM) {
                break;
            }

            best_score = curr_score;
            best_move = curr_move;
        }

        best_move
    }

    pub fn root_search(
        &mut self,
        last_best: Option<Move>,
        depth: usize,
        start_time: &Instant,
        tt: &mut TTable,
    ) -> (i32, Option<Move>) {
        let mut best_move = None;
        let mut best_score = i32::MIN;
        let player = if self.board.colour == 0 { 1 } else { -1 };

        // search the best move from the previous iteration first
        if let Some(m) = last_best {
            self.board.make(&m, tt);

            if movegen::check_check(
                &self.board,
                &movegen::bitscn_fw(&self.board.pieces[11 - self.board.colour]),
                &(1 - self.board.colour),
            ) == 0
            {
                best_score = -negamax(
                    &mut self.board,
                    i32::MIN + 1,
                    i32::MAX,
                    depth - 1,
                    MAX_SEARCH_DEPTH as i32,
                    -player,
                    tt,
                    &start_time,
                );
                println!(
                    "info cp {}, depth {} currmove {}",
                    best_score,
                    depth,
                    m.as_uci_string()
                );
                best_move = Some(m);
            }

            self.board.unmake(&m, tt);
        }

        let mut score;
        let moves = movegen::gen_moves(&self.board);
        for m in moves {
            if Instant::now().duration_since(*start_time) >= Duration::from_secs(TIME_LIM) {
                return (best_score, best_move);
            }

            self.board.make(&m, tt);

            if movegen::check_check(
                &self.board,
                &movegen::bitscn_fw(&self.board.pieces[11 - self.board.colour]),
                &(1 - self.board.colour),
            ) > 0
            {
                self.board.unmake(&m, tt);
                continue;
            }

            score = -negamax(
                &mut self.board,
                i32::MIN + 1,
                i32::MAX,
                depth - 1,
                MAX_SEARCH_DEPTH as i32,
                -player,
                tt,
                &start_time,
            );

            //println!("info cp {}, depth {} currmove {}", score, depth, m.as_uci_string());
            if score > best_score {
                if self.prev_moves.get(&self.board.pieces).unwrap_or(&0) < &2 {
                    best_move = Some(m);
                    best_score = score;
                    println!(
                        "info cp {}, depth {} currmove {}",
                        best_score,
                        depth,
                        m.as_uci_string()
                    );
                }
            }

            self.board.unmake(&m, tt);
        }
        (best_score, best_move)
    }
}

fn negamax(
    b: &mut Board,
    mut alpha: i32,
    beta: i32,
    depth: usize,
    mate_dist: i32,
    player: i32,
    tt: &mut TTable,
    start_time: &Instant,
) -> i32 {
    //dbg!(b.hash);

    if let Some(hash_score) = tt.get(b.hash, depth as u8, mate_dist, alpha, beta) {
        // account for the change in depth of checkmate within transposition table
        //println!("info string hash move");
        return hash_score;
    }

    if depth == 0 {
        //dbg!(depth);
        let eval = eval::quiesce(b, alpha, beta, player);
        // let eval = eval::evaluate(b, player);
        //tt.insert(TEntry::new(b.hash, None, 0, eval, NodeType::Pv));
        return eval;
    }

    let mut score;
    let moves = movegen::gen_moves(b);

    let mut no_moves = true;
    let mut checkmate = false;
    let mut node_type = NodeType::Alpha;
    let mut best_move = None;

    for m in moves {
        if Instant::now().duration_since(*start_time) >= Duration::from_secs(TIME_LIM) {
            break;
        }

        b.make(&m, tt);

        if movegen::check_check(
            b,
            &movegen::bitscn_fw(&b.pieces[11 - b.colour]),
            &(1 - b.colour),
        ) > 0
        {
            b.unmake(&m, tt);
            checkmate = true;
            continue;
        } else {
            no_moves = false;
        }

        score = -negamax(
            b,
            -beta,
            -alpha,
            depth - 1,
            mate_dist - 1,
            -player,
            tt,
            start_time,
        );
        b.unmake(&m, tt);

        if score >= beta {
            tt.insert(TEntry::new(b.hash, None, depth as u8, beta, NodeType::Beta));
            return beta;
        }

        if score > alpha {
            node_type = NodeType::Pv;
            best_move = Some(m);
            alpha = score;
        }
    }

    if no_moves {
        // if checkmate/stalemate
        if checkmate {
            tt.insert(TEntry::new(
                b.hash,
                None,
                depth as u8,
                eval::CHECKMATE * mate_dist,
                NodeType::Pv,
            ));
            eval::CHECKMATE * mate_dist
        } else {
            tt.insert(TEntry::new(b.hash, None, depth as u8, 0, NodeType::Pv));
            println!("info string stalemate found at {}", depth);
            0
        }
    } else {
        tt.insert(TEntry::new(
            b.hash,
            best_move,
            depth as u8,
            alpha,
            node_type,
        ));
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
    let mvs: Arc<Mutex<(Option<Move>, i32, Board)>> =
        Arc::new(Mutex::new((None, i32::MIN, search.board)));
    let prev_moves = Arc::new(RwLock::new(search.prev_moves));

    let moves = moves;

    for m in moves {
        let mv = Arc::clone(&mvs);
        let prev_m = Arc::clone(&prev_moves);

        pool.execute(move || {
            search.board.make_no_hashing(&m);

            if movegen::check_check(
                &search.board,
                &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]),
                &(1 - search.board.colour),
            ) > 0
            {
                search.board.unmake_no_hashing(&m);
                return;
            }

            let score = 0; // -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player);

            let mut best_m = mv.lock().unwrap();

            if score > best_m.1
                && *prev_m
                    .read()
                    .unwrap()
                    .get(&search.board.pieces)
                    .unwrap_or(&0)
                    < 2
            {
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
    let mvs: Arc<Mutex<(Option<Move>, i32, Board)>> =
        Arc::new(Mutex::new((None, i32::MIN, search.board)));
    let prev_moves = Arc::new(RwLock::new(search.prev_moves));

    let moves = moves;
    //let pv = search_pv(&mut search.board, &moves[0], i32::MIN+1, i32::MAX, depth, player);

    for m in moves {
        let mv = Arc::clone(&mvs);
        let prev_m = Arc::clone(&prev_moves);

        pool.execute(move || {
            search.board.make_no_hashing(&m);

            if movegen::check_check(
                &search.board,
                &movegen::bitscn_fw(&search.board.pieces[11 - search.board.colour]),
                &(1 - search.board.colour),
            ) > 0
            {
                search.board.unmake_no_hashing(&m);
                return;
            }

            let score = 0; // -negamax(&mut search.board, &m, i32::MIN+1, i32::MAX, depth-1, -player);

            let mut best_m = mv.lock().unwrap();
            if score > best_m.1
                && *prev_m
                    .read()
                    .unwrap()
                    .get(&search.board.pieces)
                    .unwrap_or(&0)
                    < 2
            {
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

fn search_pv(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, depth: usize, player: i32) -> i32 {
    if depth == 0 {
        let eval = eval::quiesce(b, alpha, beta, player);
        return eval;
    }

    let moves = movegen::gen_moves(b);
    let mut checkmate = true;

    for m in moves {
        b.make_no_hashing(&m);
        if movegen::check_check(
            b,
            &movegen::bitscn_fw(&b.pieces[11 - b.colour]),
            &(1 - b.colour),
        ) > 0
        {
            b.unmake_no_hashing(&m);
            continue;
        } else {
            checkmate = false;
        }

        let score = -search_pv(b, &m, -beta, -alpha, depth - 1, -player);

        if score >= beta {
            b.unmake_no_hashing(&m);
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
        b.unmake_no_hashing(&m);

        if checkmate {
            return eval::CHECKMATE * depth as i32;
        } else {
            return alpha;
        }
    }

    if checkmate {
        eval::CHECKMATE * depth as i32
    } else {
        alpha
    }
}

fn pvs(b: &mut Board, m: &Move, mut alpha: i32, beta: i32, depth: usize, player: i32) -> i32 {
    if depth == 0 {
        let eval = eval::quiesce(b, alpha, beta, player);
        return eval;
    }

    let moves = movegen::gen_moves(b);
    let mut checkmate = true;

    for m in moves {
        b.make_no_hashing(&m);
        if movegen::check_check(
            b,
            &movegen::bitscn_fw(&b.pieces[11 - b.colour]),
            &(1 - b.colour),
        ) > 0
        {
            b.unmake_no_hashing(&m);
            continue;
        } else {
            checkmate = false;
        }

        let score = -pvs(b, &m, -beta, -alpha, depth - 1, -player);

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
        eval::CHECKMATE * depth as i32
    } else {
        alpha
    }
}
