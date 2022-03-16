#![allow(unused)]
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

use crate::{ Board, Move };
use crate::moves::MoveType;
use crate::movegen;

#[derive(Debug)]
pub struct PerftCounter {
    moves: usize,
    quiet: usize,
    cap: usize,
    ep_cap: usize,
    castle: usize,
    promo: usize,
    check: usize,
}

impl PerftCounter {
    pub fn new() -> PerftCounter {
        PerftCounter {
            moves: 0,
            quiet: 0,
            cap: 0,
            ep_cap: 0,
            castle: 0,
            promo: 0,
            check: 0,
        }
    }

    fn count_move(&mut self, b: &Board, m: &Move) {
        self.moves += 1;
        match &m.move_type {
            MoveType::Quiet => self.quiet += 1,
            MoveType::DoublePush => self.quiet += 1,
            MoveType::Capture => self.cap += 1,
            MoveType::EpCapture => self.ep_cap += 1,
            MoveType::WKingSide => self.castle += 1,
            MoveType::WQueenSide => self.castle += 1,
            MoveType::BKingSide => self.castle += 1,
            MoveType::BQueenSide => self.castle += 1,
            MoveType::Promo => self.promo += 1,
            MoveType::PromoCapture => {
                self.promo += 1;
                self.cap += 1;
            }
        }

        if movegen::attacks_to(b, movegen::bitscn_fw(&b.pieces[10 + b.colour]), b.colour) > 0 {
            self.check += 1;
        }

        if let MoveType::Capture = &m.move_type {
            // println!("{}", b);
        }

        if m.piece == 1 {
            //println!("{}", b);
        }
    }
}

pub fn perft_counter(
    b: &mut Board,
    depth: usize,
    counter: &mut PerftCounter,
    last_m: Option<&Move>,
) {
    if depth == 0 {
        counter.count_move(b, last_m.unwrap());
        return;
    }
    let moves = movegen::gen_moves(&b);
    for m in moves {
        b.make_no_hashing(&m);
        if movegen::attacks_to(
            b,
            movegen::bitscn_fw(&b.pieces[11 - b.colour]),
            (1 - b.colour),
        ) > 0
        {
            b.unmake_no_hashing(&m);
            continue;
        }

        perft_counter(b, depth - 1, counter, Some(&m));
        b.unmake_no_hashing(&m);
    }
}
pub fn perft(b: &mut Board, depth: usize) -> usize {
    if depth == 0 {
            return 1;
    }

    let mut move_count = 0;
    let moves = movegen::gen_moves(&b);
    for m in moves {
        b.make_no_hashing(&m);
        
        if movegen::attacks_to(b, movegen::bitscn_fw(&b.pieces[11 - b.colour]), (1 - b.colour),) > 0 {
            b.unmake_no_hashing(&m);
            continue;
        }
        
        move_count += perft(b, depth - 1);
        b.unmake_no_hashing(&m);
    }

    move_count
}

pub fn perft_multi_thread(b: &Board, depth: usize) {
    let moves = movegen::gen_moves(&b);
    let pool = ThreadPool::new(moves.len());
    // let pool = ThreadPool::new(12);
    let total_count = Arc::new(Mutex::new(0));
    for m in moves {
        let mut new_b = b.clone();
        let move_count = Arc::clone(&total_count);

        pool.execute(move || {
            new_b.make_no_hashing(&m);

            if movegen::attacks_to(
                &new_b,
                movegen::bitscn_fw(&new_b.pieces[11 - new_b.colour]),
                (1 - new_b.colour),
            ) > 0
            {
                new_b.unmake_no_hashing(&m);
                return;
            }
            let mc = perft(&mut new_b, depth - 1);
            new_b.unmake_no_hashing(&m);
            dbg!(mc);
            let mut moves = move_count.lock().unwrap();
            *moves += mc;
        });
    }

    pool.join();
    println!("Move count = {}", total_count.lock().unwrap());
    println!("Done");
}
