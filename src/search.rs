use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use rayon::prelude::*;

use crate::movegen::{self, gen_attk, in_check_now};
use crate::move_ordering::{MoveOrderList, RootOrderList, KillerMoves};
use crate::moves::MoveType;
use crate::{ Board, Move, TTable };
use crate::eval::{self, STALEMATE, CHECKMATE};
use crate::transposition_table::{ NodeType, TEntry };

pub const MAX_SEARCH_DEPTH: usize = 50;

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

impl <'a> Search<'a> {
    pub fn new(board: Board, tt: &'a mut TTable, tc: TimeControl) -> Search<'a> {
        Search { board, tt, tc }
    }

    pub fn iterative_deepening_search(&mut self) -> Option<Move> {
        self.tt.hit_count = 0;
        self.tt.miss_count = 0;

        let mut root = RootOrderList::new(&self.board);
        let mut km = KillerMoves::new();

        for depth in 1..MAX_SEARCH_DEPTH {
            let mut complete = false;
            if self.tc.start_time.elapsed() <= Duration::from_millis(TIME_LIM_MS / 2){
                self.root_search(&mut root, &mut km, depth);
                complete = true;
            }
            
            // cut off any potentially unstable moves
            if self.tc.start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS + self.tc.additional) {
                println!("info string cut early");
                break;
            }
            
            if complete {
                println!("info string completed iteration");
                println!("info string {}ms left", (TIME_LIM_MS + self.tc.additional) as u128 - self.tc.start_time.elapsed().as_millis());
                self.tc.add_time(500*depth as u64);
            } 
                
        }

        println!("info string hitcount={}, miss_count={}", self.tt.hit_count, self.tt.miss_count);
        root.get_bestmove()
    }


    pub fn root_search( &mut self, root_moves: &mut RootOrderList, km: &mut KillerMoves, depth: usize) -> i32 {
        let mut best_move = None;
        let mut best_score = i32::MIN+1;
        let player = if self.board.colour == 0 { 1 } else { -1 };

        root_moves.sort();

        for (index, (m, curr_score)) in root_moves.now.iter().enumerate() {
            if self.tc.start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS + self.tc.additional) {
                println!("info string incomplete search");
                return best_score;
            }

            self.board.make(&m, self.tt);

            if movegen::in_check_next(&self.board) > 0 {
                self.board.unmake(&m, self.tt);
                continue;
            }

            let score = -self.pvs(i32::MIN + 1, -best_score, depth-1, MAX_SEARCH_DEPTH as i32, -player, km);

            root_moves.next[index] = (*m,score);

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
        
        root_moves.done_iteration();
        best_score
    }

    pub fn pvs( &mut self, mut alpha: i32, beta: i32, mut depth: usize, mate_dist: i32, player: i32, km: &mut KillerMoves ) -> i32 {

        if let Some(hash_score) = self.tt.get(self.board.hash, depth as u8, mate_dist, alpha, beta) {
            return hash_score;
        } else if self.board.is_bad_pos() {
            // prefer mates further down the line to potentially find more favourable positions later on
            return STALEMATE; 
        }

        if depth == 0 {
            let eval = eval::quiesce(self, alpha, beta, mate_dist-1, player);
            if eval < CHECKMATE{
                self.tt.insert(TEntry::new(self.board.hash, None, 0, eval, NodeType::Pv));
            } else {
                self.tt.insert(TEntry::new(self.board.hash, None, 0, eval, NodeType::Beta))
            }
            return eval;
        }

        let mut best_move = None;
        let mut no_moves = true;
        let mut checkmate = false;
        let mut node_type = NodeType::Alpha;

        let mut sub_km = KillerMoves::new();

        let og_hash = self.board.hash; 
        // staged move ordering - generates the pv and captures first and then afterwards the quiet moves 
        let captures = |b:&mut Board, _km: &KillerMoves, tt: &TTable|  MoveOrderList::new_pv_attacks(b, &movegen::gen_attk(b), tt);
        let quiet = |b:&mut Board, km: &KillerMoves, tt: &TTable| MoveOrderList::new_quiet(b, &movegen::gen_quiet(b), km, tt);

        let mut is_pv = true;
        
        for moveset in [captures, quiet]{
            let moves = moveset(&mut self.board, km, self.tt);
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

                let mut score: i32;

                if is_pv {
                    score = -self.pvs(-beta,-alpha,depth - 1, mate_dist - 1, -player, &mut sub_km);
                    is_pv = false;
                } else {
                    score = -self.pvs(-alpha-1,-alpha,depth - 1, mate_dist - 1, -player, &mut sub_km);
                    if score > alpha {
                        score = -self.pvs(-beta,-alpha,depth - 1, mate_dist - 1, -player, &mut sub_km);
                    }
                }

                self.board.unmake(&m, self.tt);

                if score >= beta {
                    match m.move_type {
                       MoveType::Capture | MoveType::EpCapture | MoveType::PromoCapture => {},
                       _  => km.push(m)
                    }

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