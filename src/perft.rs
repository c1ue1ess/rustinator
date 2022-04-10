#![allow(unused)]
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
    let moves = movegen::gen_moves(b);
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
    let moves = movegen::gen_moves(b);
    for m in moves {
        b.make_no_hashing(&m);
        
        if movegen::in_check_next(b) > 0 {
            b.unmake_no_hashing(&m);
            continue;
        }
        
        move_count += perft(b, depth - 1);
        b.unmake_no_hashing(&m);
    }

    move_count
}