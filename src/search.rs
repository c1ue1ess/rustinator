use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use rayon::prelude::*;

use crate::movegen::{self, MoveOrderList, gen_attk};
use crate::{ Board, Move, TTable };
use crate::eval::{self, STALEMATE};
use crate::transposition_table::{ NodeType, TEntry };

const MAX_SEARCH_DEPTH: usize = 20;

const TIME_LIM_MS: u64 = 5000;
const ALPHA_TIME_BONUS: u64 = 250;

pub struct Search<'a> {
    pub board: Board,
    pub tt: &'a mut TTable,
    pub tc: TimeControl
}
#[derive(Clone, Copy)]
pub struct TimeControl {
    pub start_time: Instant,
    pub additional: u64,
}

impl TimeControl {
    pub fn new(start_time: Instant) -> TimeControl {
        TimeControl { start_time, additional: 0 }
    }

    pub fn new_now() -> TimeControl {
        TimeControl { start_time: Instant::now(), additional: 0 }
    }
    pub fn add_time(&mut self, milli: u64){
        self.additional += milli;
    }
}

// pub fn iterative_deepening_search_mt(mut search: Search) -> Option<Move> {
//     let mut best_score = i32::MIN;
//     let mut best_move: Option<Move> = None;


//     let mut curr = (i32::MIN, None);
//     search.tt.hit_count = 0;
//     search.tt.miss_count = 0;

//     for depth in 2..MAX_SEARCH_DEPTH {
//         let mut complete = false;
//         if search.tc.start_time.elapsed() <= Duration::from_millis(TIME_LIM_MS / 2){
//             MoveOrderList::new_root(&search.board, search.tt, best_move).move_scores.par_iter()
//                 .map(| x | x.0)
//                 .for_each(|m| {
//                     search.tt
//                     let mut new_board = search.board.clone();
//                     new_board.make(&m, search.tt);
//                     let mut new_search;
//                     unsafe {
//                         new_search = Search::new(new_board, search.tt, search.tc.clone());
//                     }                   

//                     new_search.root_search(best_move, depth-1);
//                 });
//             complete = true;
//         }
        
        
//         // cut off any potentially unstable moves
//         if search.tc.start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS + search.tc.additional) {
//             println!("info string cut early");
//             break;
//         }
        
        
//         best_score = curr.0;
//         best_move = curr.1;
//         if complete {
//             println!("info string completed iteration");
//             println!("info string {}ms left", (TIME_LIM_MS + search.tc.additional) as u128 - search.tc.start_time.elapsed().as_millis());
//             search.tc.add_time(500*depth as u64);
//         } 
            
//     }

//     println!("info string hitcount={}, miss_count={}", search.tt.hit_count, search.tt.miss_count);
//     best_move
// }

impl <'a> Search<'a> {
    pub fn new(board: Board, tt: &'a mut TTable, tc: TimeControl) -> Search<'a> {
        Search { board, tt, tc }
    }

    pub fn iterative_deepening_search(&mut self) -> Option<Move> {
        let mut best_score = i32::MIN;
        let mut best_move: Option<Move> = None;


        let mut curr = (i32::MIN, None);
        self.tt.hit_count = 0;
        self.tt.miss_count = 0;

        for depth in 1..MAX_SEARCH_DEPTH {
            let mut complete = false;
            if self.tc.start_time.elapsed() <= Duration::from_millis(TIME_LIM_MS / 2){
                curr = self.root_search(best_move, depth);
                complete = true;
            }
            
            
            // cut off any potentially unstable moves
            if self.tc.start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS + self.tc.additional) {
                println!("info string cut early");
                break;
            }
            
            
            best_score = curr.0;
            best_move = curr.1;
            if complete {
                println!("info string completed iteration");
                println!("info string {}ms left", (TIME_LIM_MS + self.tc.additional) as u128 - self.tc.start_time.elapsed().as_millis());
                self.tc.add_time(500*depth as u64);
            } 
                
        }

        println!("info string hitcount={}, miss_count={}", self.tt.hit_count, self.tt.miss_count);
        best_move
    }


    pub fn root_search( &mut self, last_best: Option<Move>, depth: usize) -> (i32, Option<Move>) {
        let mut best_move = last_best;
        let mut best_score = i32::MIN+1;
        let player = if self.board.colour == 0 { 1 } else { -1 };
        
        for m in MoveOrderList::new_root(&self.board, self.tt, last_best) {
            if self.tc.start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS + self.tc.additional) {
                println!("info string incomplete search");
                return (best_score, best_move);
            }

            self.board.make(&m, self.tt);
            let og_hash = self.board.hash;

            if movegen::in_check_next(&self.board) > 0 {
                self.board.unmake(&m, self.tt);
                continue;
            }

            let score = -self.negamax(i32::MIN + 1, -best_score, depth-1, MAX_SEARCH_DEPTH as i32, -player);

            if score > best_score {
                best_move = Some(m);
                best_score = score;
                
                println!(
                    "info cp {}, depth {} currmove {}",
                    best_score,
                    depth,
                    m.as_uci_string()
                );
            } 

            self.board.unmake(&m, self.tt);
        }
        
        (best_score, best_move)
    }

    pub fn negamax( &mut self, mut alpha: i32, beta: i32, depth: usize, mate_dist: i32, player: i32 ) -> i32 {

        if let Some(hash_score) = self.tt.get(self.board.hash, depth as u8, mate_dist, alpha, beta) {
            return hash_score;
        } else if self.board.is_bad_pos() {
            // prefer mates further down the line to potentially find more favourable positions later on
            return STALEMATE; 
        }

        if depth == 0 {
            let eval = eval::quiesce(self, alpha, beta, player);
            self.tt.insert(TEntry::new(self.board.hash, None, 0, eval, NodeType::Pv));
            return eval;
        }

        let mut best_move = None;
        let mut no_moves = true;
        let mut checkmate = false;
        let mut node_type = NodeType::Alpha;


        // staged move ordering - generates the pv and captures first and then afterwards the quiet moves 
        let captures = |b:&mut Board, tt: &mut TTable|  MoveOrderList::new_pv_attacks(b, movegen::gen_attk(b), tt);
        let quiet = |b:&mut Board, tt: &mut TTable| MoveOrderList::new_quiet(b, movegen::gen_quiet(b), tt);

        for moveset in [captures, quiet]{
            let moves = moveset(&mut self.board, self.tt);
            for m in moves {
                if self.tc.start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS + self.tc.additional) {
                    println!("info string incomplete search");
                    break;
                }

                self.board.make(&m, self.tt);

                if movegen::in_check_next(&self.board) > 0 {
                    self.board.unmake(&m, self.tt);
                    checkmate = true;
                    continue;
                } else {
                    no_moves = false;
                }

                let score = -self.negamax(-beta,-alpha,depth - 1, mate_dist - 1, -player);

                self.board.unmake(&m, self.tt);

                if score >= beta {
                    self.tt.insert(TEntry::new(self.board.hash, None, depth as u8, beta, NodeType::Beta));
                    if m.xpiece == 12 {
                        self.tt.inc_hh(m.piece, m.to, depth as i32);
                    }
                    return beta;
                }

                if score > alpha {
                    node_type = NodeType::Pv;
                    best_move = Some(m);
                    alpha = score;
                }

            }
        }
        // if checkmate/stalemate
        if no_moves {
            if checkmate {
                self.tt.insert(TEntry::new(
                    self.board.hash,
                    None,
                    depth as u8,
                    eval::CHECKMATE * mate_dist,
                    NodeType::Pv,
                ));

                eval::CHECKMATE * mate_dist
            
            } else {
                self.tt.insert(TEntry::new(
                    self.board.hash,
                    None, 
                    depth as u8, 
                    0, 
                    NodeType::Pv
                ));
            
                STALEMATE
            }
        } else {
            self.tt.insert(TEntry::new(
                self.board.hash,
                best_move,
                depth as u8,
                alpha,
                node_type,
            ));

            alpha
        }
    }
}
