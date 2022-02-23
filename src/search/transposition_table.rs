use rand::prelude::*;

use crate::search::eval;
use crate::chess::Move;

const TTABLE_SIZE: usize = 1048576; // 2^20
const TTABLE_INDEX_MASK: u64 = 0xFFFFF; 


pub struct TTable {
    pub zorbist_array: [u64; 781],
    pub ttable: Vec<TEntry>,
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
        

        let mut ttable = vec![tentry; TTABLE_SIZE];
        TTable { 
            zorbist_array: TTable::init_zorbist_array(), 
            ttable,
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

        for i in 0..781 {
            zorbist_array[i] = prng.gen::<u64>()
        }

        zorbist_array
    }

    pub fn get(&self, hash: u64, curr_depth: u8, mate_dist: i32, alpha: i32, beta: i32) -> Option<i32> {
        let entry = self.ttable[(hash & TTABLE_INDEX_MASK) as usize]; 
        
        if entry.hash != hash || entry.depth < curr_depth {
            return None
        }

        if entry.score <= eval::CHECKMATE {
            // adjust the stored checkmate score for the current mate distance
            return Some(entry.score - (entry.score - mate_dist*eval::CHECKMATE));
        } else if entry.score >= -eval::CHECKMATE {
            return Some(entry.score - (entry.score + mate_dist*eval::CHECKMATE));
        }
        
        match entry.node_type {
            NodeType::Pv => Some(entry.score),
            NodeType::Alpha => {
                if entry.score <= alpha {
                    Some(alpha)
                } else {
                    None
                }
            }
            NodeType::Beta => {
                if entry.score >= beta{
                    Some(beta)
                } else {
                    None
                }
            }
        }
    }

    pub fn insert(&mut self, entry: TEntry) {
        // always replace
        self.ttable[(entry.hash & TTABLE_INDEX_MASK) as usize] = entry;
    }
}

#[derive(Copy, Clone)]
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


#[test]
fn ttroubleshooting() {
    let mut tt = TTable::new();

    let mut b = crate::chess::Board::new_with_hash(&tt);
    let b_score = 100;

    tt.insert(TEntry::new(b.hash, None, 10, b_score, NodeType::Alpha));
    let t_score = tt.get(b.hash, 1, 1, i32::MIN, i32::MAX).unwrap();
    println!("{b_score}, {t_score}");
    assert_ne!(b_score, t_score);
}