use rand::prelude::*;

use crate::{ Board, Move};
use crate::eval;

const TTABLE_SIZE: usize = 1048576; // 2^20
const TTABLE_INDEX_MASK: u64 = 0xFFFFF; 


#[derive(Clone)]
pub struct TTable {
    pub zorbist_array: [u64; 781],
    pub ttable: Box<[TEntry]>,
    pub hheuristic: [[i32; 64]; 12],
    pub hit_count: u64,
    pub miss_count: u64,
}

impl TTable {
    pub fn new() -> TTable {
        let tentry = TEntry {
            hash: 0,
            best_move: None,
            depth: 0,
            score: 0,
            node_type: NodeType::Pv,
        };
        
        let hheuristic = [[0; 64]; 12];

        let mut ttable = vec![tentry; TTABLE_SIZE].into_boxed_slice();
        TTable { 
            zorbist_array: TTable::init_zorbist_array(), 
            ttable,
            hheuristic,
            hit_count: 0,
            miss_count: 0
        }
    }


    /*
    --- zorbist array indexing ---
    0-767: piece positions
    768: colour
    769-772: castle rights
    773-780: file of ep square
    */
    fn init_zorbist_array() -> [u64; 781]{
        let mut zorbist_array: [u64; 781] = [0; 781]; 
        let mut prng = rand::thread_rng();

        for z in &mut zorbist_array  {
            *z = prng.gen::<u64>()
        }

        zorbist_array
    }

    pub fn get(&mut self, hash: u64, curr_depth: u8, mate_dist: i32, alpha: i32, beta: i32) -> Option<i32> {
        let entry = self.ttable[(hash & TTABLE_INDEX_MASK) as usize]; 
        
        if entry.hash == 0 || entry.depth < curr_depth {
            return None;
        } else if entry.hash != hash {
            self.miss_count += 1;
            return None;
        }

        match entry.node_type {
            NodeType::Pv => {
                self.hit_count += 1;
                
                if entry.score <= eval::CHECKMATE {
                    // adjust the stored checkmate score for the current mate distance
                    Some(entry.score - (entry.score - mate_dist*eval::CHECKMATE))
                } else if entry.score >= -eval::CHECKMATE {
                    Some(entry.score - (entry.score + mate_dist*eval::CHECKMATE))
                } else {
                    Some(entry.score)
                }
            }
            NodeType::Alpha => {
                if entry.score <= alpha {
                    self.hit_count += 1;
                    Some(alpha)
                } else {
                    None
                }
            }
            NodeType::Beta => {
                if entry.score >= beta{
                    self.hit_count += 1;
                    Some(beta)
                } else {
                    None
                }
            }

            _ => None
        }
    }

    pub fn get_bestmove(&self, hash: u64) -> Option<Move> {
        let entry = self.ttable[(hash & TTABLE_INDEX_MASK) as usize]; 
        
        if entry.hash != hash {
            None
        } else {
            entry.best_move
        }
    }

    pub fn insert(&mut self, entry: TEntry) {
        // always replace
        self.ttable[(entry.hash & TTABLE_INDEX_MASK) as usize] = entry;
    }

    pub fn get_hh(&self, piece: usize, to: usize) -> i32 {
        self.hheuristic[piece][to] 
    }

    pub fn inc_hh(&mut self, piece: usize, to: usize, depth: i32) {
        self.hheuristic[piece][to] += depth*depth
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum NodeType {
    Pv,
    Alpha,
    Beta,
}

#[derive(Copy, Clone)]
pub struct TEntry {
    hash: u64,
    best_move: Option<Move>,
    depth: u8,
    score: i32,
    node_type: NodeType,
}

impl TEntry {
    pub fn new(hash: u64, best_move: Option<Move>, depth: u8, score: i32, node_type: NodeType) -> TEntry {
        TEntry { hash, best_move, depth, score, node_type }
    }
}


// #[test]
// fn ttroubleshooting() {
//     let mut tt = TTable::new();

//     let mut b = crate::chess::Board::new_with_hash(&tt);
//     let b_score = 100;

//     tt.insert(TEntry::new(b.hash, None, 10, b_score, NodeType::Alpha));
//     let t_score = tt.get(b.hash, 1, 1, i32::MIN, i32::MAX).unwrap();
//     println!("{b_score}, {t_score}");
//     assert_ne!(b_score, t_score);
// }