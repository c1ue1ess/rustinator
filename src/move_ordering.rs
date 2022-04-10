use std::cmp::Reverse;

use crate::eval::PIECE_VALUE;
use crate::moves::MoveType;
use crate::{ Board, Move, TTable };
use crate::movegen::*;

const ATTACK_MOVE_OFFSET: i32 = 100000;
 
#[derive(Clone, Copy)]
pub struct KillerMoves {
    km1: Option<Move>,
    km2: Option<Move>,
}

impl KillerMoves {
    pub fn new() -> KillerMoves {
        KillerMoves { km1: None, km2: None}
    }

    pub fn push(&mut self, m: Move) {
        self.km2 = self.km1;
        self.km1 = Some(m);
    }

    pub fn contains(&self, m: Move) -> i32 {
        if self.km1 == Some(m){
            2
        } else if self.km2 == Some(m){
            1
        } else {
            0
        }

    }
}

pub struct RootOrderList {
    pub now: Vec<(Move, i32)>,
    pub next: Vec<(Move, i32)>,
}

impl RootOrderList {
    pub fn new(b: &Board) -> RootOrderList {
        let mut now = Vec::new();
        
        for m in gen_moves(b){
            now.push((m, i32::MIN + 1));
        }

        let next = now.clone();

        RootOrderList { now, next }
    }

    pub fn done_iteration(&mut self){
        self.now = self.next.clone();
    }

    pub fn rescore_move(&mut self, index: usize, new_score: i32){
        self.next[index].1 = new_score;
    }

    pub fn get_bestmove(&self) -> Option<Move> {
        let mut best_score = i32::MIN;
        let mut best_move = None;
        
        for (m, score) in &self.now {
            if best_score < *score {
                best_score = *score;
                best_move = Some(*m);
            }
        }

        best_move
    }

    pub fn sort(&mut self) {
        self.now.sort_by_key(|ms| Reverse(ms.1))
    }
}

pub struct MoveOrderList {
    pub move_scores: Vec<(Move, i32)>
}

impl MoveOrderList {

    pub fn len(&self) -> usize {
        self.move_scores.len()
    }

    pub fn new_pv_attacks(b: &mut Board, moves: &[Move], tt: &TTable) -> MoveOrderList {
        let mut move_scores = Vec::with_capacity(moves.len());
        
        let bestmove = tt.get_bestmove(b.hash);
        let mut added = false;

        for m in moves {
            //add pv to hash if one exists
            if bestmove == Some(*m){
                move_scores.push((*m, i32::MAX));
                added = true;
                continue;
            }
             
            move_scores.push((*m, score_attacks(b, m, tt)));
            // move_scores.push((m, 0));

        }

        if let Some(bm) = bestmove {
            move_scores.push((bm, i32::MAX));

        }

        MoveOrderList { move_scores }
    }

    pub fn new_quiet(b: &Board, moves: &[Move], km: &KillerMoves, tt: &TTable) -> MoveOrderList {
        let mut move_scores: Vec<(Move, i32)> = Vec::with_capacity(moves.len());
        
        let bestmove = tt.get_bestmove(b.hash);

        for m in moves {
            //add pv to hash if one exists
            if bestmove == Some(*m){
                move_scores.push((*m, i32::MAX));
                continue;
            }
             
            move_scores.push((*m, score_quiet(b, m, km, tt)));
            // move_scores.push((m, 0));

        }

        MoveOrderList { move_scores }
    }


    pub fn new_quiesce(b: &mut Board, moves: &[Move], tt: &TTable) -> MoveOrderList {
        let mut move_scores = Vec::with_capacity(moves.len());

        for m in moves {
            // add pv to hash if one exists
            let see_score = score_attacks(b, m, tt);
            if see_score >= 0{
                move_scores.push((*m, see_score));    
            }
            // move_scores.push((m, 0));
        }

        
        MoveOrderList { move_scores }
    }
    
    pub fn new_quiesce_in_check(b: &mut Board, moves: &[Move], tt: &TTable) -> MoveOrderList {
        let mut move_scores = Vec::with_capacity(moves.len());

        for m in moves {
            // add pv to hash if one exists
            match m.move_type {
                MoveType::Capture | MoveType::EpCapture | MoveType::PromoCapture => {
                    let see_score = score_attacks(b, m, tt);
                    if see_score < 0 {
                        move_scores.push((*m, see_score));
                    }
                },
                
                _ => move_scores.push((*m, tt.get_hh(m.piece, m.to))),
            }
        }
        
        MoveOrderList { move_scores }
    }
}


impl Iterator for MoveOrderList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut highest = i32::MIN;
        let mut index: usize = 0;

        for (i, (m, score)) in self.move_scores.iter().enumerate() {
            if *score > highest {
                highest = *score;
                index = i;
            }
             
        }

        if highest == i32::MIN {
            None
        } else {
            // set score to min so that it never gets picked again
            self.move_scores[index].1 = i32::MIN;
            Some(self.move_scores[index].0)
        }

        
    }
}

fn score_move(b: &Board, m: &Move, tt: &TTable) -> i32 {
    // if hashtable bestmove
    if let Some(bestmove) = tt.get_bestmove(b.hash) {
        if bestmove == *m {
            return i32::MAX;
        }
    } 
    
    // if capture use mvv-lva
    if m.xpiece < 12 {
        m.xpiece as i32 - m.piece as i32 + 100000
    // if quiet move see if the to square is beneficial or not
    } else {
        tt.get_hh(m.piece, m.to)
    }
}

fn score_attacks(b: &mut Board, m: &Move, tt: &TTable) -> i32 {
    static_exchange_eval(b, m) + ATTACK_MOVE_OFFSET
}

fn score_quiet(b: &Board, m: &Move, km: &KillerMoves, tt: &TTable) -> i32 {
    let is_km = km.contains(*m);
    if is_km > 0 {
        // offset killer moves so they score better than bad captures
        ATTACK_MOVE_OFFSET + is_km 
    } else {
        tt.get_hh(m.piece, m.to)
    }
}


fn static_exchange_eval(b: &mut Board, m: &Move) -> i32 {
    let mut value = 0;
    
    b.make_no_hashing(m);
    value = PIECE_VALUE[m.xpiece as usize] - see(b, m.to);
    b.unmake_no_hashing(m);

    value
}

fn see(b: &mut Board, to: u8) -> i32 {
    let mut value = 0;
    let mut smallest: (Option<Move>, i32) = (None, PIECE_VALUE[11]);

    for m in gen_attk(b){
        if m.to == to && PIECE_VALUE[m.xpiece as usize] < smallest.1 {
            smallest = (Some(m), PIECE_VALUE[m.xpiece as usize]);
        }
    }

    if smallest.1 < PIECE_VALUE[11] {
        let m = smallest.0.unwrap();
        b.make_no_hashing(&m);
        value = PIECE_VALUE[m.xpiece as usize] - see(b, to);
        b.unmake_no_hashing(&m);
    }

    value
} 
