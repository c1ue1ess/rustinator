use crate::board_info::{SQUARES, self, FA, FB, FILES, FH, FG, FC, FD, FE, FF};
use crate::search::{Search, MAX_SEARCH_DEPTH};
use crate::{ Board, Move, TTable };
use crate::moves::MoveType;
use crate::movegen::{self, bitscn_fw, in_check_now};
use crate::move_ordering::{MoveOrderList, KillerMoves};

const PAWN: i32 = 100;
const KNIGHT: i32 = 400;
const ROOK: i32 = 525;
const BISHOP: i32 = 350;
const QUEEN: i32 = 1000;
const KING: i32 = 100000;

pub const PIECE_VALUE: [i32; 12] = [
    PAWN, PAWN,
    KNIGHT, KNIGHT,
    ROOK, ROOK,
    BISHOP, BISHOP,
    QUEEN, QUEEN,
    KING, KING
];

const BISHOP_PAIR_BONUS: i32 = 200;

pub const CHECKMATE: i32 = -10000000;
pub const STALEMATE: i32 = 0;

pub const DOUBLED_PAWN_PEN: i32 = 20;
pub const ISOLATED_PAWN_PEN: i32 = 40;
pub const INNER_LEVER_BONUS: i32 = 25;
pub const OUTTER_LEVER_BONUS: i32 = 15;
pub const RAM_PEN: i32 = 20;
pub const CHAIN_BONUS: i32 = 15;
pub const SIDE_BONUS: i32 = 10;

// piece tables based off of https://www.chessprogramming.org/Simplified_Evaluation_Function as i know nothing about chess

const BPAWN_PT: [i8; 64] = [
    0,  0,  0,   0, 0, 0, 0, 0, 
    50, 50, 50,  50,  50, 50, 50, 50, 
    10, 10, 20,  30,  30, 20, 10, 10, 
    5,  5,  10,  25,  25, 10, 5, 5, 
    0,  0,  0,   20,  20, 0, 0, 0, 
    5,  -5, -10,  0,  0, -10, -5, 5, 
    5,  10, 10,  -20, -20, 10, 10, 5, 
    0,  0,  0,   0,   0, 0, 0, 0,
];
const WPAWN_PT: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 
    5, 10, 10, -20, -20, 10, 10, 5, 
    5, -5, -10, 0, 0, -10, -5, 5, 
    0, 0, 0, 20, 20, 0, 0, 0, 
    5, 5, 10, 25, 25, 10, 5, 5, 
    10, 10, 20, 30, 30, 20, 10, 10, 
    50, 50, 50, 50, 50, 50, 50, 50, 
    0, 0, 0, 0, 0, 0, 0, 0,
];

const BKNIGHT_PT: [i8; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, 
    -40, -20, 0, 0, 0, 0, -20, -40, 
    -30, 0, 10, 15, 15, 10, 0, -30, 
    -30, 5, 15, 20, 20, 15, 5, -30, 
    -30, 0, 15, 20, 20, 15, 0, -30, 
    -30, 5, 10, 15, 15, 10, 5, -30, 
    -40, -20, 0, 5, 5, 0, -20, -40, 
    -50, -40, -30, -30, -30, -30, -40, -50,
];
const WKNIGHT_PT: [i8; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, 
    -40, -20, 0, 5, 5, 0, -20, -40, 
    -30, 5, 10, 15, 15, 10, 5, -30, 
    -30, 0, 15, 20, 20, 15, 0, -30, 
    -30, 5, 15, 20, 20, 15, 5, -30, 
    -30, 0, 10, 15, 15, 10, 0, -30, 
    -40, -20, 0, 0, 0, 0, -20, -40, 
    -50, -40, -30, -30, -30, -30, -40, -50,
];

const BROOK_PT: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 
    5, 10, 10, 10, 10, 10, 10, 5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    0, 0, 0, 5, 5, 0, 0, 0,
];
const WROOK_PT: [i8; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    -5, 0, 0, 0, 0, 0, 0, -5, 
    5, 10, 10, 10, 10, 10, 10, 5, 
    0, 0, 0, 0, 0, 0, 0, 0,
];

const BBISHOP_PT: [i8; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, 
    -10, 0, 0, 0, 0, 0, 0, -10, 
    -10, 0, 5, 10, 10, 5, 0, -10, 
    -10, 5, 5, 10, 10, 5, 5, -10, 
    -10, 0, 10, 10, 10, 10, 0, -10, 
    -10, 10, 10, 10, 10, 10, 10, -10, 
    -10, 5, 0, 0, 0, 0, 5, -10, 
    -20, -10, -10, -10, -10, -10, -10, -20,
];
const WBISHOP_PT: [i8; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, 
    -10, 5, 0, 0, 0, 0, 5, -10, 
    -10, 10, 10, 10, 10, 10, 10, -10, 
    -10, 0, 10, 10, 10, 10, 0, -10, 
    -10, 5, 5, 10, 10, 5, 5, -10, 
    -10, 0, 5, 10, 10, 5, 0, -10, 
    -10, 0, 0, 0, 0, 0, 0, -10, 
    -20, -10, -10, -10, -10, -10, -10, -20,
];

const WQUEEN_PT: [i8; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, 
    -10, 0, 0, 0, 0, 0, 0, -10, 
    -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 
    -5, 0, 5, 5, 5, 5, 0, 0, 
    -10, 5, 5, 5, 5, 5, 0, -10, 
    -10, 0, 5, 0, 0, 0, 0, -10, 
    -20, -10, -10, -5, -5, -10, -10, -20,
];
const BQUEEN_PT: [i8; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, 
    -10, 0, 5, 0, 0, 0, 0, -10, 
    -10, 5, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, 0, 
    -5, 0, 5, 5, 5, 5, 0, -5, 
    -10, 0, 5, 5, 5, 5, 0, -10, 
    -10, 0, 0, 0, 0, 0, 0, -10, 
    -20, -10, -10, -5, -5, -10, -10, -20,
];

const BKING_MID_PT: [i8; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, 
    -30, -40, -40, -50, -50, -40, -40, -30, 
    -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];
const WKING_MID_PT: [i8; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20, 
    20, 20, 0, 0, 0, 0, 20, 20, 
    -10, -20, -20, -20, -20, -20, -20,
    -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
    -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50,
    -40, -40, -30,
];

const WKING_END_PT: [i8; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, 
    -30, -20, -10, 0, 0, -10, -20, -30, 
    -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];
const BKING_END_PT: [i8; 64] = [
    -50, -30, -30, -30, -30, -30, -30, -50, 
    -30, -30, 0, 0, 0, 0, -30, -30, 
    -30, -10, 20, 30, 30,
    20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10,
    20, 30, 30, 20, -10, -30, -30, -20, -10, 0, 0, -10, -20, -30, -50, -40, -30, -20, -20, -30,
    -40, -50,
];

pub const PST: [[i8; 64]; 14] = [
    WPAWN_PT,
    BPAWN_PT,
    WKNIGHT_PT,
    BKNIGHT_PT,
    WROOK_PT,
    BROOK_PT,
    WBISHOP_PT,
    BBISHOP_PT,
    WQUEEN_PT,
    BQUEEN_PT,
    WKING_MID_PT,
    BKING_MID_PT,
    WKING_END_PT,
    BKING_END_PT,
];

const CAPTURE_BONUS: i32 = 15;

const PAWN_MOBILITY: i32 = 0;
const KNIGHT_MOBILITY: i32 = 50;
const ROOK_MOBILITY: i32 = 10;
const BISHOP_MOBILITY: i32 = 15;
// queen too agressive so brought this value down from 20
const QUEEN_MOBILITY: i32 = 10;
const KING_MOBILITY: i32 = 5;

const PIECE_MOBILITY: [i32; 12] = [
    PAWN_MOBILITY, PAWN_MOBILITY,
    KNIGHT_MOBILITY, KNIGHT_MOBILITY,
    ROOK_MOBILITY, ROOK_MOBILITY,
    BISHOP_MOBILITY, BISHOP_MOBILITY,
    QUEEN_MOBILITY, QUEEN_MOBILITY,
    KING_MOBILITY, KING_MOBILITY
];

pub fn quiesce(search: &mut Search, mut alpha: i32, beta: i32, mate_dist: i32, player: i32) -> i32 {
    // println!("entered q");
    let eval = evaluate(&mut search.board, player);
    
    if eval >= beta {
        return beta;
    }
    
    if alpha < eval {
        alpha = eval;
    }

    let q_narrow = |search: &mut Search| {
        let attks = movegen::gen_attk(&mut search.board);
        MoveOrderList::new_quiesce(&mut search.board, &attks, search.tt)
    };
    let q_research = |search: &mut Search| {
        let moves = movegen::gen_moves(&mut search.board);
        MoveOrderList::new_quiesce_in_check(&mut search.board, &moves, search.tt)
    };
    
    let moves = [q_narrow, q_research];
    let mut no_moves = true;
    let mut checkmate = false;
    let mut score;
    
    for moveset in moves {
        let move_list = moveset(search);
    
        for m in move_list {
            search.board.make_no_hashing(&m);

            if movegen::in_check_next(&search.board) > 0 {
                search.board.unmake_no_hashing(&m);
                checkmate = true;
                continue;
            } else if search.board.is_bad_pos() {
                search.board.unmake_no_hashing(&m);
                continue;
            } else {
                no_moves = false;
                
            }

            score = -quiesce(search, -beta, -alpha, mate_dist-1, -player);
            
            search.board.unmake_no_hashing(&m);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }

        }

        if checkmate && no_moves {
            //dbg!(no_moves);
            //dbg!(checkmate);
        } else {
            break;
        }
    }

    if no_moves && checkmate {
        // println!("checkmate in q at {mate_dist}");
        CHECKMATE
    } else {
        alpha
    }
}





pub fn evaluate(b: &mut Board, player: i32) -> i32 {
    let mut eval = mat_balance(b);
    eval += pos_balance(b); 
    eval += mobility(b);
    eval += pawn_structure(b);
    eval *= player;
        
    eval

    
}

fn mat_balance(b: &Board) -> i32 {
    let pawns = PAWN * (b.pieces[0].count_ones() as i32 - b.pieces[1].count_ones() as i32);
    let knights = KNIGHT * (b.pieces[2].count_ones() as i32 - b.pieces[3].count_ones() as i32);
    let rooks = ROOK * (b.pieces[4].count_ones() as i32 - b.pieces[5].count_ones() as i32);
    
    // bishop pair bonus
    let w_bishop_count = b.pieces[6].count_ones() as i32;
    let w_bishop_bonus =  BISHOP_PAIR_BONUS * (w_bishop_count/2);
    let b_bishop_count = b.pieces[7].count_ones() as i32;
    let b_bishop_bonus =  BISHOP_PAIR_BONUS * (b_bishop_count/2);
    let bishops = BISHOP * (w_bishop_count - b_bishop_count) + w_bishop_bonus - b_bishop_bonus;

    let queens = QUEEN * (b.pieces[8].count_ones() as i32 - b.pieces[9].count_ones() as i32);
    // dont need kings makes no difference
    //let kings = KING * (b.pieces[10].count_ones() as i32 - b.pieces[11].count_ones() as i32);

    pawns + knights + rooks + bishops + queens //+ kings
}

fn pos_balance(b: &Board) -> i32 {
    let mut pos: i32 = 0;

    for (p, pst) in PST.iter().enumerate().take(5) {
        //white
        let mut pieces = b.pieces[p * 2];
        while pieces > 0 {
            let sq = movegen::bitscn_fw(&pieces);
            pos += pst[sq] as i32;
            pieces &= pieces - 1;
        }

        // vs black
        let mut pieces = b.pieces[p * 2 + 1];
        while pieces > 0 {
            let sq = movegen::bitscn_fw(&pieces);
            pos -= pst[sq] as i32;
            pieces &= pieces - 1;
        }
    }

    // if endgame, use endgame king pst
    if b.pieces[8] | b.pieces[9] == 0 {
        pos += PST[12][movegen::bitscn_fw(&b.pieces[10])] as i32;
        pos -= PST[13][movegen::bitscn_fw(&b.pieces[11])] as i32;
    } else {
        pos += PST[10][movegen::bitscn_fw(&b.pieces[10])] as i32;
        pos -= PST[11][movegen::bitscn_fw(&b.pieces[11])] as i32;
    }

    pos
}

fn mobility(b: &mut Board) -> i32 {
    let mut mob = 0;
    let actual_colour = b.colour;

    b.colour = 0;
    for m in movegen::gen_moves(b) {
        mob += PIECE_MOBILITY[m.piece as usize];
        mob += if m.move_type ==  MoveType::Capture { CAPTURE_BONUS  } else { 0 };
    }
    
    b.colour = 1;
    for m in movegen::gen_moves(b) {
        mob -= PIECE_MOBILITY[m.piece as usize];
        mob -= if m.move_type ==  MoveType::Capture { CAPTURE_BONUS } else { 0 };
    }
    
    b.colour = actual_colour;

    mob
}

fn pawn_structure(b: &Board) -> i32 {
    let mut pawns = 0;
    
    // reward connected pawns and penalising isolated and doubled pawns
    pawns += pawn_chains(b);
    // levers good
    pawns += pawn_levers(b);
    // no rams are bad apparently
    pawns += pawn_rams(b);
    //doubled bad
    pawns += doubled_pawns(b);
    //backwards also bad
    pawns += isolated_pawns(b);

    pawns
}

fn doubled_pawns(b: &Board) -> i32 {
    let mut doubled = 0;
    for rank in board_info::RANKS {
        if (b.pieces[0] & rank).count_ones() > 1 {
            doubled -= DOUBLED_PAWN_PEN;
        }
        if (b.pieces[1] & rank).count_ones() > 1 {
            doubled += DOUBLED_PAWN_PEN;
        }
    }

    doubled
}

fn isolated_pawns(b: &Board) -> i32 {
    let mut iso = 0;

    // file a
    if (b.pieces[0] & FA) > 0 && (b.pieces[0] & FB) == 0 {
        iso -= ISOLATED_PAWN_PEN;
    }
    if (b.pieces[1] & FA) > 0 && (b.pieces[1] & FB) == 0 {
        iso += ISOLATED_PAWN_PEN;
    }
    // middle files
    for i in 1..7 {
        if  (b.pieces[0] & FILES[i]) > 0 && 
            (b.pieces[0] & (FILES[i-1] | FILES[i+1])) == 0 {
            iso -= ISOLATED_PAWN_PEN;
        }
        if (b.pieces[1] & FILES[i]) > 0 && (b.pieces[1] & (FILES[i-1] | FILES[i+1])) == 0 {
            iso += ISOLATED_PAWN_PEN;
        }
    }
    // file h
    if (b.pieces[0] & FH) > 0 && (b.pieces[0] & FG) == 0 {
        iso -= ISOLATED_PAWN_PEN;
    }
    if (b.pieces[1] & FH) > 0 && (b.pieces[1] & FG) == 0 {
        iso += ISOLATED_PAWN_PEN;
    }

    iso
}

fn pawn_levers(b: &Board) -> i32 {
    let mut lever = 0;

    let left = b.pieces[0] & FA & FB & FC & FD;
    let right = b.pieces[0] & FE & FF & FG & FH;
    
    // inner levers
    lever += ((left << 9) & b.pieces[1]).count_ones() as i32 * INNER_LEVER_BONUS;
    lever += ((right << 7) & b.pieces[1]).count_ones() as i32 * INNER_LEVER_BONUS;
    // outter levers
    lever += (((left & !FA) << 7) & b.pieces[1]).count_ones() as i32 * OUTTER_LEVER_BONUS;
    lever += (((right & !FH) << 9) & b.pieces[1]).count_ones() as i32 * OUTTER_LEVER_BONUS;

    let left = b.pieces[1] & FA & FB & FC & FD;
    let right = b.pieces[1] & FE & FF & FG & FH;
    //inner leavers
    lever -= ((left >> 7) & b.pieces[0]).count_ones() as i32 * INNER_LEVER_BONUS;
    lever -= ((right >> 9) & b.pieces[0]).count_ones() as i32 * INNER_LEVER_BONUS;
    // outter leavers
    lever -= (((left & !FA) >> 9) & b.pieces[0]).count_ones() as i32 * OUTTER_LEVER_BONUS;
    lever -= (((right & !FH) >> 7) & b.pieces[0]).count_ones() as i32 * OUTTER_LEVER_BONUS;

    lever
}

fn pawn_rams(b:&Board) -> i32 {
    let mut rams = 0;

    rams -= ((b.pieces[0] << 8) & b.pieces[1]).count_ones() as i32 * RAM_PEN;
    // rams -= ((b.pieces[1] >> 8) & b.pieces[0]).count_ones() as i32 * RAM_BONUS;
    
    rams
}

fn pawn_chains(b: &Board) -> i32 {
    let mut chains = 0;
    // left chains white
    chains += (b.pieces[0] & ((b.pieces[0] & !FH) << 9)).count_ones() as i32 * CHAIN_BONUS;
    // right chains white
    chains += (b.pieces[0] & ((b.pieces[0] & !FA) << 7)).count_ones() as i32 * CHAIN_BONUS;

    // left chains black
    chains -= (b.pieces[1] & ((b.pieces[1] & !FH) >> 7)).count_ones() as i32 * CHAIN_BONUS;
    // right chains black
    chains -= (b.pieces[1] & ((b.pieces[1] & !FA) >> 9)).count_ones() as i32 * CHAIN_BONUS;
    
    chains
}

fn pawn_side_by_side(b: &Board) -> i32 {
    let mut sbs = 0;

    sbs += (b.pieces[0] & ((b.pieces[0] & !FH) << 1)).count_ones() as i32 * SIDE_BONUS;
    sbs -= (b.pieces[1] & ((b.pieces[1] & !FH) << 1)).count_ones() as i32 * SIDE_BONUS;

    sbs
}



#[test]
fn pawn_score() {
    
    let mut b = Board::new_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    // let p = pawn_rams(&b);

    let white = evaluate(&mut b, 1);
    let black = evaluate(&mut b, -1);
    assert_eq!(white, -black);
    // assert_eq!(p, RAM_PEN)
    // assert_eq!(p, 0)
}
