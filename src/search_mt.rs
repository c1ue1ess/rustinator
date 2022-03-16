// #![allow(unused)]
// use std::collections::HashMap;
// use std::sync::{Arc, Mutex, RwLock};
// use std::time::{Duration, Instant};
// use threadpool::ThreadPool;

// use rayon::prelude::*;

// use crate::chess::movegen::{self, gen_attk, MoveOrderList};
// use crate::chess::{Board, Move};
// use crate::search::{eval, NodeType, TEntry, TTable};

// const TIME_LIM_MS: u64 = 5000;
// const MAX_SEARCH_DEPTH: usize = 20;

// pub struct Search {
//     pub board: Board,
//     pub prev_moves: HashMap<[u64; 12], usize>,
// }

// impl Search {
//     pub fn new(board: Board, prev_moves: HashMap<[u64; 12], usize>) -> Search {
//         Search { board, prev_moves }
//     }

//     pub fn iterative_deepening_search(&mut self, tt: &'static mut TTable) -> Option<Move> {
//         let mut best_score = i32::MIN;
//         let mut best_move: Option<Move> = None;

//         let start_time = Instant::now();
//         for depth in 1..MAX_SEARCH_DEPTH {
//             // if start_time.elapsed() <= Duration::from_millis(TIME_LIM_MS / 2){
//             let (curr_score, curr_move) = self.root_search(best_move, depth, &start_time, tt);
//             // }

//             if start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS) {
//                 break;
//             }

//             best_score = curr_score;
//             best_move = curr_move;
//         }

//         best_move
//     }

//     pub fn root_search<'a>(
//         &'a mut self,
//         last_best: Option<Move>,
//         depth: usize,
//         start_time: &Instant,
//         tt: &'static mut TTable,
//     ) -> (i32, Option<Move>) {
//         let mut best_move = None;
//         let mut best_score = i32::MIN;
//         let player = if self.board.colour == 0 { 1 } else { -1 };

//         // search the best move from the previous iteration first
//         if let Some(m) = last_best {
//             self.board.make(&m, tt);

//             if movegen::check_check(
//                 &self.board,
//                 &movegen::bitscn_fw(&self.board.pieces[11 - self.board.colour]),
//                 &(1 - self.board.colour),
//             ) == 0
//             {
//                 best_score = -super::search::negamax(
//                     &mut self.board,
//                     i32::MIN + 1,
//                     i32::MAX,
//                     depth - 1,
//                     MAX_SEARCH_DEPTH as i32,
//                     -player,
//                     tt,
//                     &start_time,
//                 );

//                 println!(
//                     "info cp {}, depth {} currmove {}",
//                     best_score,
//                     depth,
//                     m.as_uci_string()
//                 );

//                 best_move = Some(m);
//             }

//             self.board.unmake(&m, tt);
//         }

//         // let moves = MoveOrderList::new(&self.board, movegen::gen_moves(&self.board), tt);
//         let captures = |b: &mut Board, tt: &mut TTable| {
//             MoveOrderList::new_pv_attacks(b, movegen::gen_attk(b), tt)
//         };
//         let quiet =
//             |b: &mut Board, tt: &mut TTable| MoveOrderList::new_quiet(b, movegen::gen_quiet(b), tt);

//         let tt_rw = RwLock::new(&tt);         
            
//             for moveset in [captures, quiet] {
//                 let moves = moveset(&mut self.board, tt);
//                 let pool = rayon::ThreadPoolBuilder::new().num_threads(moves.len()).build_scoped(wrapper, with_pool)
                
//                 for m in moves {

//                 let start_time_arc = Arc::new(&start_time);
//                 let best_move_arc = Arc::new(&Mutex::new(&best_move));
//                 let best_score_arc = Arc::new(&Mutex::new(&best_score));

//                 // locking the tt when in use may be slow but its a start
//                 let tt_arc = Arc::clone(&Arc::new(tt_rw));
                
//                 let mut selfc = Search { board: self.board, prev_moves: self.prev_moves };

//                 pool.execute(move || {
//                     if start_time_arc.elapsed() >= Duration::from_millis(TIME_LIM_MS) {
//                         return
//                     }

//                     selfc.board.make(&m, &tt_arc.read().unwrap());

//                     if movegen::check_check(
//                         &selfc.board,
//                         &movegen::bitscn_fw(&selfc.board.pieces[11 - selfc.board.colour]),
//                         &(1 - selfc.board.colour),
//                     ) > 0
//                     {
//                         selfc.board.unmake(&m, tt_arc.get_mut().unwrap());
//                         return;
//                     }

//                     let score = -negamax(
//                         &mut selfc.board,
//                         i32::MIN + 1,
//                         i32::MAX,
//                         depth - 1,
//                         MAX_SEARCH_DEPTH as i32,
//                         -player,
//                         &tt_arc,
//                         &start_time,
//                     );

//                     if score > best_score {
//                         if selfc.prev_moves.get(&selfc.board.pieces).unwrap_or(&0) < &2 {
//                             best_move = Some(m);
//                             best_score = score;

//                             println!(
//                                 "info cp {}, depth {} currmove {}",
//                                 best_score,
//                                 depth,
//                                 m.as_uci_string()
//                             );
//                         }
//                     }

//                     selfc.board.unmake(&m, &tt_arc.read().unwrap());
//                 });
//             }
//         }
//         (best_score, best_move)
//     }
// }

// fn negamax(
//     b: &mut Board,
//     mut alpha: i32,
//     beta: i32,
//     depth: usize,
//     mate_dist: i32,
//     player: i32,
//     tt: &Arc<RwLock<&&mut TTable>>,
//     start_time: &Instant,
// ) -> i32 {
//     if let Some(hash_score) = tt.read().unwrap().get(b.hash, depth as u8, mate_dist, alpha, beta) {
//         return hash_score;
//     }

//     if depth == 0 {
//         let eval = eval::quiesce(b, alpha, beta, player, &tt.read().unwrap());
//         tt.write().unwrap().insert(TEntry::new(b.hash, None, 0, eval, NodeType::Pv));
//         return eval;
//     }

//     let mut best_move = None;
//     let mut no_moves = true;
//     let mut checkmate = false;
//     let mut node_type = NodeType::Alpha;

//     // staged move ordering - generates the pv and captures first and then afterwards the quiet moves
//     let captures =
//         |b: &mut Board, tt: &TTable| MoveOrderList::new_pv_attacks(b, movegen::gen_attk(b), tt);
//     let quiet =
//         |b: &mut Board, tt: &TTable| MoveOrderList::new_quiet(b, movegen::gen_quiet(b), tt);

//     for moveset in [captures, quiet] {
//         let moves = moveset(b, &tt.read().unwrap());
//         for m in moves {
//             if start_time.elapsed() >= Duration::from_millis(TIME_LIM_MS) {
//                 break;
//             }

//             b.make(&m, &tt.read().unwrap());

//             if movegen::check_check(
//                 b,
//                 &movegen::bitscn_fw(&b.pieces[11 - b.colour]),
//                 &(1 - b.colour),
//             ) > 0
//             {
//                 b.unmake(&m, &tt.read().unwrap());
//                 checkmate = true;
//                 continue;
//             } else {
//                 no_moves = false;
//             }

//             let score = -negamax(
//                 b,
//                 -beta,
//                 -alpha,
//                 depth - 1,
//                 mate_dist - 1,
//                 -player,
//                 tt,
//                 start_time,
//             );

//             b.unmake(&m, &tt.read().unwrap());

//             if score >= beta {
//                 tt.write().unwrap().insert(TEntry::new(b.hash, None, depth as u8, beta, NodeType::Beta));
//                 if m.xpiece == 12 {
//                     tt.write().unwrap().inc_hh(m.piece, m.to, depth as i32);
//                 }
//                 return beta;
//             }

//             if score > alpha {
//                 node_type = NodeType::Pv;
//                 best_move = Some(m);
//                 alpha = score;
//             }
//         }
//     }
//     // if checkmate/stalemate
//     if no_moves {
//         if checkmate {
//             tt.write().unwrap().insert(TEntry::new(
//                 b.hash,
//                 None,
//                 depth as u8,
//                 eval::CHECKMATE * mate_dist,
//                 NodeType::Pv,
//             ));

//             eval::CHECKMATE * mate_dist
//         } else {
//             tt.write().unwrap().insert(TEntry::new(b.hash, None, depth as u8, 0, NodeType::Pv));

//             0
//         }
//     } else {
//         tt.write().unwrap().insert(TEntry::new(
//             b.hash,
//             best_move,
//             depth as u8,
//             alpha,
//             node_type,
//         ));
//         alpha
//     }
// }
