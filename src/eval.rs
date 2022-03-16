use crate::search::Search;
use crate::{ Board, Move, TTable };
use crate::moves::MoveType;
use crate::movegen::{ self, MoveOrderList };

const PAWN: i32 = 100;
const KNIGHT: i32 = 400;
const ROOK: i32 = 525;
const BISHOP: i32 = 350;
const QUEEN: i32 = 1200;
const KING: i32 = 25000;

pub const PIECE_VALUE: [i32; 12] = [
    PAWN, -PAWN,
    KNIGHT, -KNIGHT,
    ROOK, -ROOK,
    BISHOP, -BISHOP,
    QUEEN, -QUEEN,
    KING, -KING
];

const BISHOP_PAIR_BONUS: i32 = 100;

pub const CHECKMATE: i32 = -10000000;
pub const STALEMATE: i32 = 0;

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
const KNIGHT_MOBILITY: i32 = 20;
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

pub fn quiesce(search: &mut Search, mut alpha: i32, beta: i32, player: i32) -> i32 {
    let eval = evaluate(&mut search.board, player);

    if eval >= beta {
        return beta;
    }
    if alpha < eval {
        alpha = eval;
    }

    // delta pruning
    if eval < alpha - QUEEN {
        return alpha;
    }


    // sort captures
    let captures = MoveOrderList::new_pv_attacks(&search.board, movegen::gen_attk(&search.board), search.tt); 
    
    let mut no_moves = true;
    let mut checkmate = false;
    let mut score;
    
    
    for cap in captures {
        search.board.make_no_hashing(&cap);

        if movegen::in_check_next(&search.board) > 0
        {
            search.board.unmake_no_hashing(&cap);
            checkmate = true;
            continue;
        } else {
            no_moves = false;
        }

        score = -quiesce(search, -beta, -alpha, -player);
        
        search.board.unmake_no_hashing(&cap);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }

    }

    
    if no_moves {
        // if checkmate/stalemate
        if checkmate && movegen::in_check_now(&search.board) > 0 {
            CHECKMATE
        } else {
            STALEMATE
        }
    } else {
        alpha
    }
}



pub fn evaluate(b: &mut Board, player: i32) -> i32 {
    let mut eval = mat_balance(b);
    eval += pos_balance(b); 
    eval += mobility(b);
    
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
    let kings = KING * (b.pieces[10].count_ones() as i32 - b.pieces[11].count_ones() as i32);

    pawns + knights + rooks + bishops + queens + kings
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
        mob += PIECE_MOBILITY[m.piece];
        mob += if m.move_type ==  MoveType::Capture { CAPTURE_BONUS  } else { 0 };
    }
    
    b.colour = 1;
    for m in movegen::gen_moves(b) {
        mob -= PIECE_MOBILITY[m.piece];
        mob -= if m.move_type ==  MoveType::Capture { CAPTURE_BONUS } else { 0 };
    }
    
    b.colour = actual_colour;

    mob
}

// #[test]
// fn ev_score() {
//     // let mut w = Board::new_from_fen("pnrbqk2/8/8/8/8/8/PNRBQK2/PNRBQK2 w - - 0 10");
//     // let mut b = Board::new_from_fen("PNRBQK2/8/8/8/8/8/pnrbqk2/pnrbqk2 b - - 0 10");
//     // let mut b = Board::new_from_fen("pnrbqk2/pnrbqk2/8/8/8/8/8/PNRBQK2 b - - 0 10");

//     let mut w = Board::new();
//     let mut b = Board::new();
//     b.colour = 1;

//     assert_eq!(quiesce(&mut w, i32::MIN+1, i32::MAX, 1), quiesce(&mut b, i32::MIN+1, i32::MAX, -1));
//     let mut w_search = crate::search::Search { board: w, prev_moves: std::collections::HashMap::new() };
//     let mut b_search = crate::search::Search { board: b, prev_moves: std::collections::HashMap::new() };

//     let depth = 11;

//     let w_score = crate::search::root_search(&mut w_search, None, depth, &std::time::Instant::now(), &mut crate::search::TTable::new()).0;
//     let b_score = crate::search::root_search(&mut b_search, None, depth, &std::time::Instant::now(), &mut crate::search::TTable::new()).0;

//     assert_eq!(w_score, b_score)
// }
