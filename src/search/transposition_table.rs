use rand::prelude::*;

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

    pub fn get(&self, hash: u64, curr_depth: u8) -> Option<i32> {
        let entry = self.ttable[(hash & TTABLE_INDEX_MASK) as usize]; 
        if entry.hash == hash && entry.depth >= curr_depth {
            Some(entry.score)
        } else {
            None
        }
    }

    pub fn insert(&mut self, entry: TEntry) {
        //if new entry's depth is of a higher depth or is newer
        //if entry.depth >= self.ttable[(entry.hash & TTABLE_INDEX_MASK) as usize].depth {
            //}
        
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
    depth: u8,
    score: i32,
    node_type: NodeType,
}

impl TEntry {
    pub fn new(hash: u64, depth: u8, score: i32, node_type: NodeType) -> TEntry {
        TEntry { hash, depth, score, node_type }
    }
}
