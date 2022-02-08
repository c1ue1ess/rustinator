use crate::chess::Board;
use crate::chess::Move;
use crate::chess::movegen;
use crate::chess::moves::MoveType;

const PAWN: i32 = 100;
const KNIGHT: i32 = 350;
const ROOK: i32 = 525;
const BISHOP: i32 = 325;
const QUEEN: i32 = 1000;
const KING: i32 = 1000000;

pub const CHECKMATE: i32 = 10000000;


pub fn quiesce( b: &mut Board, m: &Move, mut alpha: i32, beta: i32, player: i32) -> i32 {

    // if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[10 + b.colour]), &(b.colour),) > 0 {
    //     return i32::MIN + 1;
    // }

    let eval = evaluate(b, m, player);
    
    if eval >= beta {
        return beta;
    }
    if alpha < eval {
        alpha = eval;
    }

    let mut captures = Vec::new();
    movegen::all_attk(&mut captures, b);
    
    let mut checkmate = true;
    
    let mut score;
    for cap in captures {
        //println!("{}\n{}\n", b, cap);
        b.make(&cap);
        
        if movegen::check_check(b, &movegen::bitscn_fw(&b.pieces[11 - b.colour]), &(1 - b.colour),) > 0 {
            b.unmake(&cap);
            continue;
        } else { 
            checkmate = false
        }
        
        score = -quiesce(b, &cap, -beta, -alpha, -player);
        
        if score >= beta {
            b.unmake(&cap);
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
        
        b.unmake(&cap);
    }

    if checkmate {
        -CHECKMATE
    } else {
        alpha
    }
}

pub fn evaluate(b: &Board, m: &Move, player: i32) -> i32 {
    let mut eval = 0;
    eval += mat_balance(b);
    //eval += pos_balance(b);
    eval *= player;
 

    eval 
}

fn mat_balance(b: &Board) -> i32 {
    let mut balance = 0;
	let wp: i32 = b.pieces[0].count_ones() as i32;
	let bp: i32 = b.pieces[1].count_ones() as i32;
	let wn: i32 = b.pieces[2].count_ones() as i32;
	let bn: i32 = b.pieces[3].count_ones() as i32;
	let wr: i32 = b.pieces[4].count_ones() as i32;
	let br: i32 = b.pieces[5].count_ones() as i32;
	let wb: i32 = b.pieces[6].count_ones() as i32;
	let bb: i32 = b.pieces[7].count_ones() as i32;
	let wq: i32 = b.pieces[8].count_ones() as i32;
	let bq: i32 = b.pieces[9].count_ones() as i32;
	let wk: i32 = b.pieces[10].count_ones() as i32;
	let bk: i32 = b.pieces[11].count_ones() as i32;

	balance += (wp - bp) * PAWN;
	balance += (wn - bn) * KNIGHT;
	balance += (wr - br) * ROOK;
	balance += (wb - bb) * BISHOP;
	balance += (wq - bq) * QUEEN;
	balance += (wk - bk) * KING;

	balance
}

fn pos_balance(b: &Board) -> i32 {
    let mut balance = 0;

    const MID: u64 = 0x8181000000;
	const OUTTER_MID: u64 = 0xB34242B30000;


    let mut wp = (b.pieces[0] & MID).count_ones() as i32 * PAWN * 15;
	let mut bp = (b.pieces[1] & MID).count_ones() as i32 * PAWN * 15;
	let mut wn = (b.pieces[2] & MID).count_ones() as i32 * KNIGHT * 50;
	let mut bn = (b.pieces[3] & MID).count_ones() as i32 * KNIGHT * 50;
	let mut wr = (b.pieces[4] & MID).count_ones() as i32 * ROOK * 25;
	let mut br = (b.pieces[5] & MID).count_ones() as i32 * ROOK * 25;
	let mut wb = (b.pieces[6] & MID).count_ones() as i32 * BISHOP * 40;
	let mut bb = (b.pieces[7] & MID).count_ones() as i32 * BISHOP * 40;
	let mut wq = (b.pieces[8] & MID).count_ones() as i32 * QUEEN * 50;
	let mut bq = (b.pieces[9] & MID).count_ones() as i32 * QUEEN * 50;
	let mut wk = (b.pieces[10] & MID).count_ones() as i32 * KING * 0;
	let mut bk = (b.pieces[11] & MID).count_ones() as i32 * KING * 0;

	balance += (wp - bp) + (wn - bn) + (wr - br) + (wb - bb) + (wq - bq) + (wk - bk);

	wp = (b.pieces[0] & OUTTER_MID).count_ones() as i32 * PAWN * 30;
	bp = (b.pieces[1] & OUTTER_MID).count_ones() as i32 * PAWN * 30;
	wn = (b.pieces[2] & OUTTER_MID).count_ones() as i32 * KNIGHT * 50;
	bn = (b.pieces[3] & OUTTER_MID).count_ones() as i32 * KNIGHT * 50;
	wr = (b.pieces[4] & OUTTER_MID).count_ones() as i32 * ROOK * 40;
	br = (b.pieces[5] & OUTTER_MID).count_ones() as i32 * ROOK * 40;
	wb = (b.pieces[6] & OUTTER_MID).count_ones() as i32 * BISHOP * 40;
	bb = (b.pieces[7] & OUTTER_MID).count_ones() as i32 * BISHOP * 40;
	wq = (b.pieces[8] & OUTTER_MID).count_ones() as i32 * QUEEN * 50;
	bq = (b.pieces[9] & OUTTER_MID).count_ones() as i32 * QUEEN * 50;
	wk = (b.pieces[10] & OUTTER_MID).count_ones() as i32 * KING * 5;
	bk = (b.pieces[11] & OUTTER_MID).count_ones() as i32 * KING * 5;

	balance + (wp - bp) + (wn - bn) + (wr - br) + (wb - bb) + (wq - bq) + (wk - bk)
}