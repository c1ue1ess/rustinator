use crate::chess::Board;
use crate::chess::Move;
use crate::chess::movegen;

const PAWN: i32 = 100;
const KNIGHT: i32 = 350;
const ROOK: i32 = 525;
const BISHOP: i32 = 325;
const QUEEN: i32 = 1000;
const KING: i32 = 25000;

pub const CHECKMATE: i32 = 10000000;


// piece tables based off of https://www.chessprogramming.org/Simplified_Evaluation_Function as i know nothing about chess

const WPAWN_PT: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];
const BPAWN_PT: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10,-20,-20, 10, 10,  5,
    5, -5,-10,  0,  0,-10, -5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
    0,  0,  0,  0,  0,  0,  0,  0,
];

const WKNIGHT_PT: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];
const BKNIGHT_PT: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

const WROOK_PT: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0  
];
const BROOK_PT: [i32; 64] = [
    0,  0,  0,  5,  5,  0,  0,  0,  
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    5, 10, 10, 10, 10, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0,

];

const WBISHOP_PT: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];
const BBISHOP_PT: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,

];

const WQUEEN_PT: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];
const BQUEEN_PT: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
    0,  0,  5,  5,  5,  5,  0, -5,
    -5,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

const WKING_MID_PT: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];
const BKING_MID_PT: [i32; 64] = [
    20, 30, 10,  0,  0, 10, 30, 20,
    20, 20,  0,  0,  0,  0, 20, 20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
];

const WKING_END_PT: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
];
const BKING_END_PT: [i32; 64] = [
    -50,-30,-30,-30,-30,-30,-30,-50,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -50,-40,-30,-20,-20,-30,-40,-50,
];

const PST: [[i32; 64]; 14] = [
    WPAWN_PT, BPAWN_PT, 
    WKNIGHT_PT, BKNIGHT_PT, 
    WROOK_PT, BROOK_PT, 
    WBISHOP_PT, BBISHOP_PT,
    WQUEEN_PT, BQUEEN_PT, 
    WKING_MID_PT, BKING_MID_PT,
    WKING_END_PT, BKING_END_PT
];

pub fn quiesce( b: &mut Board, m: &Move, mut alpha: i32, beta: i32, player: i32) -> i32 {
    let eval = evaluate(b, m, player);
    
    if eval >= beta {
        return beta;
    }
    if alpha < eval {
        alpha = eval;
    }

    let mut captures = Vec::with_capacity(300);
    movegen::all_attk(&mut captures, b);
    // sort captures
    captures.sort_unstable_by(|a, b| a.xpiece.cmp(&b.xpiece).reverse());
    
    let mut no_moves = true;
    
    let mut score;
    for cap in captures {
        b.make_no_hashing(&cap);
        
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[11 - b.colour]), &(1 - b.colour),) > 0 {
            b.unmake_no_hashing(&cap);
            continue;
        } else { 
            no_moves = false
        }
        
        score = -quiesce(b, &cap, -beta, -alpha, -player);
        b.unmake_no_hashing(&cap);
        
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }   
    }

    if no_moves {
        //if checkmate or stalemate
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[10 + b.colour]), &(b.colour)) > 0 {
            -CHECKMATE
        } else {
            0
        }
    } else {
        alpha
    }
}

pub fn evaluate(b: &Board, _m: &Move, player: i32) -> i32 {
    (mat_balance(b) + pos_balance(b)) * player //+ movegen::gen_moves(b).len() as i32 * 25
}

fn mat_balance(b: &Board) -> i32 {	
	let pawns   = PAWN * (b.pieces[0].count_ones() as i32 - b.pieces[1].count_ones() as i32);
	let knights = KNIGHT * (b.pieces[2].count_ones() as i32 - b.pieces[3].count_ones() as i32);
	let rooks   = ROOK * (b.pieces[4].count_ones() as i32 - b.pieces[5].count_ones() as i32);
	let bishops = BISHOP * (b.pieces[6].count_ones() as i32 - b.pieces[7].count_ones() as i32);
	let queens  = QUEEN * (b.pieces[8].count_ones() as i32 - b.pieces[9].count_ones() as i32);
	let kings   = KING * (b.pieces[10].count_ones() as i32 - b.pieces[11].count_ones() as i32);

	pawns + knights + rooks + bishops + queens + kings
}

fn pos_balance(b: &Board) -> i32 {
    let mut pos = 0;
    
    // For now only king midgame pst is used until i decide when endgame starts
    for p in 0..6 {
        //white
        let mut pieces = b.pieces[p*2];
        while pieces > 0 {
            let sq = movegen::bitscn_fw(&pieces);
            pos += PST[p][sq];
            pieces &= pieces-1;
        }

        // vs black
        let mut pieces = b.pieces[p*2+1];
        while pieces > 0 {
            let sq = movegen::bitscn_fw(&pieces);
            pos -= PST[p][sq];
            pieces &= pieces-1;
        }
    }

    pos
}