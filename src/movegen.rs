use std::ops::Index;
use std::slice::SliceIndex;

use crate::{ Board, Move, TTable };
use crate::moves::MoveType;
use crate::eval;
use crate::board_info::*;



pub struct MoveOrderList {
    pub move_scores: Vec<(Move, i32)>,
}

impl MoveOrderList {
    pub fn new(b: &Board, moves: Vec<Move>, tt: &TTable) -> MoveOrderList {
        let mut move_scores: Vec<(Move, i32)> = Vec::with_capacity(moves.len());
        
        for m in moves {
            move_scores.push((m, score_move(b, &m, tt)));
            //move_scores.push((m, 0));

        }
        
        MoveOrderList { move_scores }
    }

    

    pub fn len(&self) -> usize {
        self.move_scores.len()
    }

    pub fn new_pv_attacks(b: &Board, mut moves: Vec<Move>, tt: &TTable) -> MoveOrderList {
        let mut move_scores: Vec<(Move, i32)> = Vec::with_capacity(moves.len());
        
        let bestmove = tt.get_bestmove(b.hash);

        for m in moves {
            //add pv to hash if one exists
            if bestmove == Some(m){
                move_scores.push((m, i32::MAX));
                continue;
            }
             
            move_scores.push((m, score_attacks(b, &m, tt)));
            // move_scores.push((m, 0));

        }


        MoveOrderList { move_scores }
    }

    pub fn new_quiet(b: &Board, moves: Vec<Move>, tt: &TTable) -> MoveOrderList {
        let mut move_scores: Vec<(Move, i32)> = Vec::with_capacity(moves.len());
        
        let bestmove = tt.get_bestmove(b.hash);

        for m in moves {
            //add pv to hash if one exists
            if bestmove == Some(m){
                move_scores.push((m, i32::MAX));
                continue;
            }
             
            move_scores.push((m, score_quiet(b, &m, tt)));
            // move_scores.push((m, 0));

        }


        MoveOrderList { move_scores }
    }

    pub fn new_root(b: &Board, tt: &TTable, last_best: Option<Move>) -> MoveOrderList {
        let captures = gen_attk(b);
        let quiet = gen_quiet(b);
        
        let mut move_scores: Vec<(Move, i32)> = Vec::with_capacity(captures.len() + quiet.len());

        let bestmove = tt.get_bestmove(b.hash);
        
        for m in captures {
            if bestmove == Some(m){
                move_scores.push((m, i32::MAX-1));
                continue;
            } else if last_best == Some(m) {
                move_scores.push((m, i32::MAX));
                continue;
            }

            move_scores.push((m, score_attacks(b, &m, tt)));
            // move_scores.push((m, 0));
        }
        
        for m in quiet {
            if bestmove == Some(m){
                move_scores.push((m, i32::MAX-1));
                continue;
            } else if last_best == Some(m) {
                move_scores.push((m, i32::MAX));
                continue;
            }

            move_scores.push((m, score_quiet(b, &m, tt)));
            // move_scores.push((m, 0));

        }


        MoveOrderList { move_scores }
    }

    pub fn new_quiesce(b: &Board, mut moves: Vec<Move>, tt: &TTable) -> MoveOrderList {
        let mut move_scores: Vec<(Move, i32)> = Vec::with_capacity(moves.len());

        for m in moves {
            // add pv to hash if one exists
            move_scores.push((m, score_attacks(b, &m, tt)));    
            // move_scores.push((m, 0));
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

fn score_attacks(b: &Board, m: &Move, tt: &TTable) -> i32 {
    //mvv-lva
    m.xpiece as i32 - m.piece as i32 + 100000 // add offset so no overlap with history heuristics
}

fn score_quiet(b: &Board, m: &Move, tt: &TTable) -> i32 {
    tt.get_hh(m.piece, m.to)
}


pub fn gen_attk(b: &Board) -> Vec<Move> {
    let mut attk = Vec::with_capacity(256);
    all_attk(&mut attk, b);
    attk
}

pub fn gen_quiet(b: &Board) -> Vec<Move> {
    let mut quiet = Vec::with_capacity(256);
    all_quiet(&mut quiet, b);
    quiet
}

pub fn gen_moves(b: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(300);
    all_attk(&mut moves, b);

    // sort attacks by most valuable capture
    //moves.sort_unstable_by(|a, b| a.xpiece.cmp(&b.xpiece).reverse());
    
    all_quiet(&mut moves, b);
    
    moves
}

fn all_quiet(moves: &mut Vec<Move>, b: &Board) {
    if b.colour == 0 {
        add_wp_quiet(moves, b);
    } else {
        add_bp_quiet(moves, b);
    }

    add_knight_quiet(moves, b);
    add_rook_quiet(moves, b);
    add_bishop_quiet(moves, b);
    add_queen_quiet(moves, b);
    add_king_quiet(moves, b);
}

fn all_attk(moves: &mut Vec<Move>, b: &Board) {
    if b.colour == 0 {
        add_wp_attk(moves, b);
    } else {
        add_bp_attk(moves, b);
    }

    add_knight_attk(moves, b);
    add_rook_attk(moves, b);
    add_bishop_attk(moves, b);
    add_queen_attk(moves, b);
    add_king_attk(moves, b);
}

fn add_wp_quiet(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut quiet;
    

    let mut pawns = b.pieces[0];
    while pawns > 0 {
        from = bitscn_fw(&pawns);
        quiet = wpawn_moves(from, b);
        if quiet > 0 {
            to = bitscn_fw(&quiet);
            if to > 55 {
                for i in [8, 4, 2, 6] {
                    moves.push(Move::new_promo(from, to, 0, b.ep, b, i));
                }
            } else {
                moves.push(Move::new_quiet(from, to, 0, b.ep, b));
            }

            quiet &= quiet - 1;
            if quiet > 0 {
                to = bitscn_fw(&quiet);
                moves.push(Move::new_double_push(from, to, 0, b.ep, b));
            }
        }

        pawns &= pawns - 1;
    }
}

fn add_wp_attk(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut attk;
    
    let mut pawns = b.pieces[0];
    while pawns > 0 {
        from = bitscn_fw(&pawns);
        attk = wpawn_attk(from, b);
        while attk > 0 {
            to = bitscn_fw(&attk);
            if to > 55 {
                for i in [8, 4, 2, 6] {
                    moves.push(Move::new_promo_capture( from, to, 0, get_xpiece(to, b), b.ep, b, i));
                }
            } else if to as u8 == b.ep {
                moves.push(Move::new_ep_capture(from, to, 0, 1, b.ep, b));
            } else {
                moves.push(Move::new_capture( from, to, 0, get_xpiece(to, b), b.ep, b));
            }
            
            attk &= attk - 1;
        }
        pawns &= pawns - 1;
    }
}

fn add_bp_quiet(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut quiet;

    let mut pawns = b.pieces[1];
    while pawns > 0 {
        from = bitscn_fw(&pawns);
        quiet = bpawn_moves(from, b);

        if quiet > 0 {
            to = bitscn_rv(&quiet);
            if to < 8 {
                for i in [9, 5, 3, 7] {
                    moves.push(Move::new_promo(from, to, 1, b.ep, b, i));
                }
            } else {
                moves.push(Move::new_quiet(from, to, 1, b.ep, b));
            }

            quiet ^= SQUARES[to];
            if quiet > 0 {
                to = bitscn_fw(&quiet);
                moves.push(Move::new_double_push(from, to, 1, b.ep, b));
            }
        }

        pawns &= pawns - 1;
    }
}

fn add_bp_attk(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut attk;

    let mut pawns = b.pieces[1];
    while pawns > 0 {
        from = bitscn_fw(&pawns);
        attk = bpawn_attk(from, b);

        while attk > 0 {
            to = bitscn_fw(&attk);
            if to < 8 {
                for i in [9, 5, 3, 7] {
                    moves.push(Move::new_promo_capture( from, to, 1, get_xpiece(to, b), b.ep, b, i));
                }
            } else if to as u8 == b.ep {
                moves.push(Move::new_ep_capture(from, to, 1, 0, b.ep, b));
            } else {
                moves.push(Move::new_capture( from, to, 1, get_xpiece(to, b), b.ep, b));
            }

            attk &= attk - 1;
        }
        pawns &= pawns - 1;
    }
}

fn add_knight_quiet(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut quiet;

    let mut knights = b.pieces[2 + b.colour];
    while knights > 0 {
        from = bitscn_fw(&knights);
        let m = knight_moves(from);
        quiet = m & !b.util[2];
        
        while quiet > 0 {
            to = bitscn_fw(&quiet);
            moves.push(Move::new_quiet(from,to,2 + b.colour,b.ep,b));
            quiet &= quiet - 1;
        }
        knights &= knights - 1;
    }
}

fn add_knight_attk(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut attk;

    let mut knights = b.pieces[2 + b.colour];
    while knights > 0 {
        from = bitscn_fw(&knights);
        let m = knight_moves(from);
        attk = m & b.util[1 - b.colour];

        while attk > 0 {
            to = bitscn_fw(&attk);
            moves.push(Move::new_capture( from, to, 2 + b.colour, get_xpiece(to, b), b.ep, b));
            attk &= attk - 1;
        }
        knights &= knights - 1;
    }
}

fn add_rook_quiet(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut quiet;

    let mut rooks = b.pieces[4 + b.colour];
    while rooks > 0 {
        from = bitscn_fw(&rooks);
        let m = rook_moves(from, b);
        quiet = m & !b.util[2];

        while quiet > 0 {
            to = bitscn_fw(&quiet);
            moves.push(Move::new_quiet( from, to, 4 + b.colour, b.ep, b));
            quiet &= quiet - 1;
        }
        rooks &= rooks - 1;
    }
}

fn add_rook_attk(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut attk;

    let mut rooks = b.pieces[4 + b.colour];
    while rooks > 0 {
        from = bitscn_fw(&rooks);
        let m = rook_moves(from, b);
        attk = m & b.util[1 - b.colour];

        while attk > 0 {
            to = bitscn_fw(&attk);
            moves.push(Move::new_capture( from, to, 4 + b.colour, get_xpiece(to, b), b.ep, b));
            attk &= attk - 1;
        }
        rooks &= rooks - 1;
    }
}

fn add_bishop_quiet(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut quiet;

    let mut bishops = b.pieces[6 + b.colour];
    while bishops > 0 {
        from = bitscn_fw(&bishops);
        let m = bishop_moves(from, b);
        quiet = m & !b.util[2];

        while quiet > 0 {
            to = bitscn_fw(&quiet);
            moves.push(Move::new_quiet( from, to, 6 + b.colour, b.ep, b));
            quiet &= quiet - 1;
        }
        bishops &= bishops - 1;
    }
}

fn add_bishop_attk(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut attk;

    let mut bishops = b.pieces[6 + b.colour];
    while bishops > 0 {
        from = bitscn_fw(&bishops);
        let m = bishop_moves(from, b);
        attk = m & b.util[1 - b.colour];
        while attk > 0 {
            to = bitscn_fw(&attk);
            moves.push(Move::new_capture( from, to, 6 + b.colour, get_xpiece(to, b), b.ep, b));
            attk &= attk - 1;
        }
        bishops &= bishops - 1;
    }
}

fn add_queen_quiet(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut quiet;

    let mut queens = b.pieces[8 + b.colour];
    while queens > 0 {
        from = bitscn_fw(&queens);
        let m = queen_moves(from, b);
        quiet = m & !b.util[2];

        while quiet > 0 {
            to = bitscn_fw(&quiet);
            moves.push(Move::new_quiet( from, to, 8 + b.colour, b.ep, b));
            quiet &= quiet - 1;
        }
        queens &= queens - 1;
    }
}

fn add_queen_attk(moves: &mut Vec<Move>, b: &Board) {
    let mut from;
    let mut to;
    let mut attk;

    let mut queens = b.pieces[8 + b.colour];
    while queens > 0 {
        from = bitscn_fw(&queens);
        let m = queen_moves(from, b);
        attk = m & b.util[1 - b.colour];

        while attk > 0 {
            to = bitscn_fw(&attk);
            moves.push(Move::new_capture( from, to, 8 + b.colour, get_xpiece(to, b), b.ep, b));
            attk &= attk - 1;
        }
        queens &= queens - 1;
    }
}

pub fn add_king_quiet(moves: &mut Vec<Move>, b: &Board) {                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 
    
    let mut to;
    

    let king = b.pieces[10 + b.colour];
    let from = bitscn_fw(&king);
    let m = king_moves(from);
    let mut quiet = m & !b.util[2];
    while quiet > 0 {
        to = bitscn_fw(&quiet);
        if attacks_to(b, to, b.colour) > 0 {
            quiet &= quiet - 1;
            continue;
        }
        moves.push(Move::new_quiet( from, to, 10 + b.colour, b.ep, b));
        quiet &= quiet - 1;
    }

    // no need to check castle moves if king is in check
    if attacks_to(b, from, b.colour) > 0 { return; }
    
    if b.colour == 0
        && (b.castle_state & 0b1000) > 0
        && (b.util[2] & 0x60) == 0
        && (attacks_to(b, 5, 0) | attacks_to(b, 6, 0)) == 0
    {
        moves.push(Move::new_castle(
            4,
            6,
            10,
            b.ep,
            b,
            MoveType::WKingSide,
        ))
    }

    if b.colour == 0
        && (b.castle_state & 0b100) > 0
        && (b.util[2] & 0xE) == 0
        && (attacks_to(b, 3, 0) | attacks_to(b, 2, 0)) == 0
    {
        moves.push(Move::new_castle(
            4,
            2,
            10,
            b.ep,
            b,
            MoveType::WQueenSide,
        ))
    }

    if b.colour == 1
        && (b.castle_state & 0b10) > 0
        && (b.util[2] & 0x6000000000000000) == 0
        && (attacks_to(b, 61, 1) |  attacks_to(b, 62, 1)) == 0
    {
        moves.push(Move::new_castle(
            60,
            62,
            11,
            b.ep,
            b,
            MoveType::BKingSide,
        ))
    }

    if b.colour == 1
        && (b.castle_state & 1) > 0
        && (b.util[2] & 0x0E00000000000000) == 0
        && (attacks_to(b, 59, 1) | attacks_to(b, 58, 1)) == 0
    {
        moves.push(Move::new_castle(
            60,
            58,
            11,
            b.ep,
            b,
            MoveType::BQueenSide,
        ))
    }
}

pub fn add_king_attk(moves: &mut Vec<Move>, b: &Board) {
    
    let mut to;
    

    let king = b.pieces[10 + b.colour];
    let from = bitscn_fw(&king);
    let m = king_moves(from);
    let mut attk = m & b.util[1 - b.colour];

    while attk > 0 {
        to = bitscn_fw(&attk);
        if attacks_to(b, to, b.colour) > 0 {
            attk &= attk - 1;
            continue;
        }
        moves.push(Move::new_capture(
            from,
            to,
            10 + b.colour,
            get_xpiece(to, b),
            b.ep,
            b,
        ));
        attk &= attk - 1;
    }
}

pub fn bitscn_fw(bb: &u64) -> usize {
    bb.trailing_zeros() as usize
}

pub fn bitscn_rv(bb: &u64) -> usize {
    (bb.leading_zeros() ^ 63) as usize
}

fn king_moves(index: usize) -> u64 {
    let k_clear_a = SQUARES[index] & !FA;
    let k_clear_h = SQUARES[index] & !FH;

    let up = SQUARES[index] << 8;
    let down = SQUARES[index] >> 8;

    let up_left = k_clear_a << 7;
    let left = k_clear_a >> 1;
    let down_left = k_clear_a >> 9;

    let up_right = k_clear_h << 9;
    let right = k_clear_h << 1;
    let down_right = k_clear_h >> 7;

    up | down | up_left | left | down_left | up_right | right | down_right
}

fn knight_moves(index: usize) -> u64 {
    let ull = (SQUARES[index] & !FA & !FB) << 6;
    let uul = (SQUARES[index] & !FA) << 15;
    let uur = (SQUARES[index] & !FH) << 17;
    let urr = (SQUARES[index] & !FG & !FH) << 10;

    let drr = (SQUARES[index] & !FH & !FG) >> 6;
    let ddr = (SQUARES[index] & !FH) >> 15;
    let ddl = (SQUARES[index] & !FA) >> 17;
    let dll = (SQUARES[index] & !FA & !FB) >> 10;

    ull | uul | uur | urr | drr | ddr | ddl | dll
}

fn wpawn_moves(index: usize, b: &Board) -> u64 {
    let up = (SQUARES[index] << 8) & !b.util[2];
    let up_up = ((up & R3) << 8) & !b.util[2];

    up | up_up
}

fn wpawn_attk(index: usize, b: &Board) -> u64 {
    let up_left = (SQUARES[index] & !FA) << 7;
    let up_right = (SQUARES[index] & !FH) << 9;

    (up_left | up_right) & (b.util[1] | SQUARES[b.ep as usize])
}

fn bpawn_moves(index: usize, b: &Board) -> u64 {
    let down = (SQUARES[index] >> 8) & !b.util[2];
    let down_down = ((down & R6) >> 8) & !b.util[2];
    down | down_down
}

fn bpawn_attk(index: usize, b: &Board) -> u64 {
    let down_left = (SQUARES[index] & !FH) >> 7;
    let down_right = (SQUARES[index] & !FA) >> 9;
    (down_left | down_right) & (b.util[0] | SQUARES[b.ep as usize])
}

fn pos_ray(dir: usize, sq: usize, b: &Board) -> u64 {
    let mv = RAYS[dir][sq];
    let blk = mv & b.util[2];
    let b_index: usize = bitscn_fw(&blk) as usize;
    mv ^ RAYS[dir][b_index] 
}

fn neg_ray(dir: usize, sq: usize, b: &Board) -> u64 {
    let mv = RAYS[dir][sq];
    let blk = mv & b.util[2] | 1;
    let b_index: usize = bitscn_rv(&blk) as usize;

    mv ^ (RAYS[dir][b_index])
}

fn rook_moves(sq: usize, b: &Board) -> u64 {
    pos_ray(1, sq, b) | pos_ray(3, sq, b) | neg_ray(5, sq, b) | neg_ray(7, sq, b)
}

fn bishop_moves(sq: usize, b: &Board) -> u64 {
    pos_ray(0, sq, b) | pos_ray(2, sq, b) | neg_ray(4, sq, b) | neg_ray(6, sq, b)
}

fn queen_moves(sq: usize, b: &Board) -> u64 {
    bishop_moves(sq, b) | rook_moves(sq, b)
}









#[inline(always)]
pub fn attacks_to(b: &Board, piece_index: usize, colour: usize) -> u64 {
    let pawn: u64 = if colour == 0 {
        wpawn_attk(piece_index, b) & b.pieces[1]
    } else {
        bpawn_attk(piece_index, b) & b.pieces[0]
    };

    pawn
    | knight_moves(piece_index) & b.pieces[3 - colour] 
    | rook_moves(piece_index, b) & (b.pieces[5 - colour] | b.pieces[9 - colour]) 
    | bishop_moves(piece_index, b) & (b.pieces[7 - colour] | b.pieces[9 - colour]) 
    | king_moves(piece_index) & (b.pieces[11 - colour])
}

pub fn in_check_next(b: &Board) -> u64 {
    let k_index = bitscn_fw(&b.pieces[11-b.colour]);

    attacks_to(b, k_index, b.colour ^ 1)
}


pub fn in_check_now(b: &Board) -> u64 {
    let k_index = bitscn_fw(&b.pieces[10+b.colour]);

    attacks_to(b, k_index, b.colour)
}













pub fn get_xpiece(sq: usize, b: &Board) -> usize {
    if (SQUARES[sq] & b.pieces[1 - b.colour]) > 0 {
        1 - b.colour
    } else if (SQUARES[sq] & b.pieces[3 - b.colour]) > 0 {
        3 - b.colour
    } else if (SQUARES[sq] & b.pieces[5 - b.colour]) > 0 {
        5 - b.colour
    } else if (SQUARES[sq] & b.pieces[7 - b.colour]) > 0 {
        7 - b.colour
    } else if (SQUARES[sq] & b.pieces[9 - b.colour]) > 0 {
        9 - b.colour
    } else if (SQUARES[sq] & b.pieces[11 - b.colour]) > 0 {
        11 - b.colour
    } else {
        12
    }
}

pub fn get_piece(sq: usize, b: &Board) -> usize {
    if (SQUARES[sq] & b.pieces[b.colour]) > 0 {
		b.colour
	} else if (SQUARES[sq] & b.pieces[2 + b.colour]) > 0 {
		2 + b.colour
	} else if (SQUARES[sq] & b.pieces[4 + b.colour]) > 0 {
		4 + b.colour
	} else if (SQUARES[sq] & b.pieces[6 + b.colour]) > 0 {
		6 + b.colour
	} else if (SQUARES[sq] & b.pieces[8 + b.colour]) > 0 {
		8 + b.colour
	} else if (SQUARES[sq] & b.pieces[10 + b.colour]) > 0 {
		10 + b.colour
	} else {
        12
    }
}

// #[allow(dead_code)]
// prints a bit board over the top of a board
// pub fn print_bb(m: u64, b: &Board) {
//     let mut out = String::new();

//     for i in (1..9).rev() {
//         let s = i.to_string();
//         out.push_str(&s);
//         out.push_str("   ");

//         for j in i * 8 - 8..i * 8 {
//             if (SQUARES[j] & m) > 0 {
//                 out.push('(');
//             } else {
//                 out.push(' ');
//             }

//             if (SQUARES[j] & b.pieces[0]) > 0 {
//                 out.push('P');
//             } else if (SQUARES[j] & b.pieces[1]) > 0 {
//                 out.push('p');
//             } else if (SQUARES[j] & b.pieces[2]) > 0 {
//                 out.push('N');
//             } else if (SQUARES[j] & b.pieces[3]) > 0 {
//                 out.push('n');
//             } else if (SQUARES[j] & b.pieces[4]) > 0 {
//                 out.push('R');
//             } else if (SQUARES[j] & b.pieces[5]) > 0 {
//                 out.push('r');
//             } else if (SQUARES[j] & b.pieces[6]) > 0 {
//                 out.push('B');
//             } else if (SQUARES[j] & b.pieces[7]) > 0 {
//                 out.push('b');
//             } else if (SQUARES[j] & b.pieces[8]) > 0 {
//                 out.push('Q');
//             } else if (SQUARES[j] & b.pieces[9]) > 0 {
//                 out.push('q');
//             } else if (SQUARES[j] & b.pieces[10]) > 0 {
//                 out.push('K');
//             } else if (SQUARES[j] & b.pieces[11]) > 0 {
//                 out.push('k');
//             } else {
//                 out.push('-');
//             }

//             if (SQUARES[j] & m) > 0 {
//                 out.push(')');
//             } else {
//                 out.push(' ');
//             }
//         }
//         out.push('\n');
//     }
//     out.push_str("\n     A  B  C  D  E  F  G  H\n");

//     println!("{}", out);
// }
