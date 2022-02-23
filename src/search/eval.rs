use crate::chess::movegen;
use crate::chess::Board;
use crate::chess::Move;

const PAWN: i32 = 100;
const KNIGHT: i32 = 350;
const ROOK: i32 = 525;
const BISHOP: i32 = 325;
const QUEEN: i32 = 1000;
const KING: i32 = 25000;

pub const CHECKMATE: i32 = -10000000;

// piece tables based off of https://www.chessprogramming.org/Simplified_Evaluation_Function as i know nothing about chess

const BPAWN_PT: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];
const WPAWN_PT: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, -20, -20, 10, 10, 5, 5, -5, -10, 0, 0, -10, -5, 5, 0, 0, 0,
    20, 20, 0, 0, 0, 5, 5, 10, 25, 25, 10, 5, 5, 10, 10, 20, 30, 30, 20, 10, 10, 50, 50, 50, 50,
    50, 50, 50, 50, 0, 0, 0, 0, 0, 0, 0, 0,
];

const BKNIGHT_PT: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];
const WKNIGHT_PT: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15, 15, 10,
    5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 10, 15, 15, 10,
    0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

const BROOK_PT: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];
const WROOK_PT: [i32; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0,
    0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 5, 10, 10, 10, 10, 10, 10, 5, 0, 0,
    0, 0, 0, 0, 0, 0,
];

const BBISHOP_PT: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];
const WBISHOP_PT: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 5, 0, 0, 0, 0, 5, -10, -10, 10, 10, 10, 10, 10,
    10, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

const WQUEEN_PT: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, -5, 0, 5, 5, 5, 5, 0, 0, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];
const BQUEEN_PT: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 5, 0, 0, 0, 0, -10, -10, 5, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, 0, -5, 0, 5, 5, 5, 5, 0, -5, -10, 0, 5, 5, 5, 5, 0, -10, -10, 0, 0, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const BKING_MID_PT: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];
const WKING_MID_PT: [i32; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20, 20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20, -20, -20, -20,
    -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
    -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50,
    -40, -40, -30,
];

const WKING_END_PT: [i32; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30,
    30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30,
    -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30,
    -30, -50,
];
const BKING_END_PT: [i32; 64] = [
    -50, -30, -30, -30, -30, -30, -30, -50, -30, -30, 0, 0, 0, 0, -30, -30, -30, -10, 20, 30, 30,
    20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10,
    20, 30, 30, 20, -10, -30, -30, -20, -10, 0, 0, -10, -20, -30, -50, -40, -30, -20, -20, -30,
    -40, -50,
];

const PST: [[i32; 64]; 14] = [
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

pub fn quiesce(b: &mut Board, mut alpha: i32, beta: i32, player: i32) -> i32 {
    let eval = evaluate(b, player);

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
    let mut checkmate = false;
    let mut score;
    for cap in captures {
        b.make_no_hashing(&cap);

        if movegen::check_check(
            b,
            &movegen::bitscn_fw(&b.pieces[11 - b.colour]),
            &(1 - b.colour),
        ) > 0
        {
            b.unmake_no_hashing(&cap);
            checkmate = true;
            continue;
        } else {
            no_moves = false;
        }

        score = -quiesce(b, -beta, -alpha, -player);
        b.unmake_no_hashing(&cap);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    if no_moves {
        // if checkmate/stalemate
        if checkmate
            && movegen::check_check(
                b,
                &movegen::bitscn_fw(&b.pieces[10 + b.colour]),
                &(b.colour),
            ) > 0
        {
            CHECKMATE
        } else {
            0
        }
    } else {
        alpha
    }
}

pub fn evaluate(b: &Board, player: i32) -> i32 {
    let mut eval = (mat_balance(b) + pos_balance(b)); // + movegen::gen_moves(b).len() as i32 * 50;
    eval * player
}

fn mat_balance(b: &Board) -> i32 {
    let pawns = PAWN * (b.pieces[0].count_ones() as i32 - b.pieces[1].count_ones() as i32);
    let knights = KNIGHT * (b.pieces[2].count_ones() as i32 - b.pieces[3].count_ones() as i32);
    let rooks = ROOK * (b.pieces[4].count_ones() as i32 - b.pieces[5].count_ones() as i32);
    let bishops = BISHOP * (b.pieces[6].count_ones() as i32 - b.pieces[7].count_ones() as i32);
    let queens = QUEEN * (b.pieces[8].count_ones() as i32 - b.pieces[9].count_ones() as i32);
    let kings = KING * (b.pieces[10].count_ones() as i32 - b.pieces[11].count_ones() as i32);

    // println!("pawns {} knights {} rooks {} bishops {} queens {} kings {}",
    //     pawns/PAWN, knights/KNIGHT, rooks/ROOK, bishops/BISHOP, queens/QUEEN, kings/KING);
    pawns + knights + rooks + bishops + queens + kings
}

fn pos_balance(b: &Board) -> i32 {
    let mut pos = 0;

    for p in 0..5 {
        //white
        let mut pieces = b.pieces[p * 2];
        while pieces > 0 {
            let sq = movegen::bitscn_fw(&pieces);
            pos += PST[p][sq];
            pieces &= pieces - 1;
        }

        // vs black
        let mut pieces = b.pieces[p * 2 + 1];
        while pieces > 0 {
            let sq = movegen::bitscn_fw(&pieces);
            pos -= PST[p][sq];
            pieces &= pieces - 1;
        }
    }

    // if endgame, use endgame king pst
    if b.pieces[8] | b.pieces[9] == 0 {
        pos += PST[12][movegen::bitscn_fw(&b.pieces[10])];
        pos -= PST[13][movegen::bitscn_fw(&b.pieces[11])];
    } else {
        pos += PST[10][movegen::bitscn_fw(&b.pieces[10])];
        pos -= PST[11][movegen::bitscn_fw(&b.pieces[11])];
    }

    pos
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
