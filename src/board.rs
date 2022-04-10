use std::fmt;

use crate::{ Move, TTable };
use crate::moves::MoveType;
use crate::board_info::*;

// 2 ^ 14 sized prev move array
const PREV_MOVE_SIZE: usize = 16384;
const PREV_MOVE_MASK: u64 = 0x3FFF;

#[derive(Debug, Clone)]
pub struct Board {
    pub pieces: [u64; 12],
    pub util: [u64; 3],

    pub colour: usize,

    pub ep: u8,
    pub castle_state: u8,
    pub whas_castled: bool,
    pub bhas_castled: bool,

    pub halfmove: u8,
    pub fullmove: u8,

    pub hash: u64,
    pub prev_moves: [u8; PREV_MOVE_SIZE],
}

impl Board {
    pub fn new() -> Board {
        let mut b = Board {
            pieces: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            util: [0, 0, 0],
            colour: 0,
            ep: 64,
            castle_state: 0b1111,
            whas_castled: false,
            bhas_castled: false,
            halfmove: 0,
            fullmove: 1,
            hash: 0,
            prev_moves: [0; PREV_MOVE_SIZE]
        };

        b.pieces[0] =  0b0000000000000000000000000000000000000000000000001111111100000000; //wp 0
        b.pieces[2] =  0b0000000000000000000000000000000000000000000000000000000001000010; //wn 2
        b.pieces[4] =  0b0000000000000000000000000000000000000000000000000000000010000001; //wr 4
        b.pieces[6] =  0b0000000000000000000000000000000000000000000000000000000000100100; //wb 6
        b.pieces[8] =  0b0000000000000000000000000000000000000000000000000000000000001000; //wq 8
        b.pieces[10] = 0b0000000000000000000000000000000000000000000000000000000000010000; //wk 10

        b.pieces[1] =  0b0000000011111111000000000000000000000000000000000000000000000000; //bp 1
        b.pieces[3] =  0b0100001000000000000000000000000000000000000000000000000000000000; //bn 3
        b.pieces[5] =  0b1000000100000000000000000000000000000000000000000000000000000000; //br 5
        b.pieces[7] =  0b0010010000000000000000000000000000000000000000000000000000000000; //bb 7
        b.pieces[9] =  0b0000100000000000000000000000000000000000000000000000000000000000; //bq 9
        b.pieces[11] = 0b0001000000000000000000000000000000000000000000000000000000000000; //bk 11

        // util[0] all white, util[1] all black, util[2] all pieces
        b.util[0] = b.pieces[0] | b.pieces[2] | b.pieces[4] | b.pieces[6] | b.pieces[8] | b.pieces[10];
        b.util[1] = b.pieces[1] | b.pieces[3] | b.pieces[5] | b.pieces[7] | b.pieces[9] | b.pieces[11];
        b.util[2] = b.util[0] | b.util[1];
        b
    }

    pub fn new_with_hash(tt: &TTable) -> Board {
        let mut board = Board::new();
        board.hash = board.get_hash(tt);
        
        board.add_prev_move();

        board
    }

    pub fn new_from_fen(fen: &str) -> Board {
        let mut b = Board {
            pieces: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            util: [0, 0, 0],
            colour: 0,
            ep: 64,
            castle_state: 0b1111,
            whas_castled: false,
            bhas_castled: false,
            halfmove: 0,
            fullmove: 0,
            hash: 0,
            prev_moves: [0; PREV_MOVE_SIZE],
        };
        let fen: Vec<&str> = fen.split(' ').collect();

        let mut j = 56;
        for f in fen[0].chars() {
            match f {
                'P' => {
                    b.pieces[0] ^= SQUARES[j];
                    j += 1;
                }
                'p' => {
                    b.pieces[1] ^= SQUARES[j];
                    j += 1;
                }
                'N' => {
                    b.pieces[2] ^= SQUARES[j];
                    j += 1;
                }
                'n' => {
                    b.pieces[3] ^= SQUARES[j];
                    j += 1;
                }
                'R' => {
                    b.pieces[4] ^= SQUARES[j];
                    j += 1;
                }
                'r' => {
                    b.pieces[5] ^= SQUARES[j];
                    j += 1;
                }
                'B' => {
                    b.pieces[6] ^= SQUARES[j];
                    j += 1;
                }
                'b' => {
                    b.pieces[7] ^= SQUARES[j];
                    j += 1;
                }
                'Q' => {
                    b.pieces[8] ^= SQUARES[j];
                    j += 1;
                }
                'q' => {
                    b.pieces[9] ^= SQUARES[j];
                    j += 1;
                }
                'K' => {
                    b.pieces[10] ^= SQUARES[j];
                    j += 1;
                }
                'k' => {
                    b.pieces[11] ^= SQUARES[j];
                    j += 1;
                }
                '1' => j += '1' as usize - '0' as usize,
                '2' => j += '2' as usize - '0' as usize,
                '3' => j += '3' as usize - '0' as usize,
                '4' => j += '4' as usize - '0' as usize,
                '5' => j += '5' as usize - '0' as usize,
                '6' => j += '6' as usize - '0' as usize,
                '7' => j += '7' as usize - '0' as usize,
                '8' => j += '8' as usize - '0' as usize,
                '/' => j -= 16,
                _ => {}
            }
        }

        b.util[0] =
            b.pieces[0] | b.pieces[2] | b.pieces[4] | b.pieces[6] | b.pieces[8] | b.pieces[10];
        b.util[1] =
            b.pieces[1] | b.pieces[3] | b.pieces[5] | b.pieces[7] | b.pieces[9] | b.pieces[11];
        b.util[2] = b.util[0] | b.util[1];
        b.colour = if fen[1].contains('w') { 0 } else { 1 };

        match fen[2] {
            "KQkq"  => b.castle_state = 0b1111,
            "KQk"   => b.castle_state = 0b1110,
            "KQq"   => b.castle_state = 0b1101,
            "KQ"    => b.castle_state = 0b1100,
            "Kkq"   => b.castle_state = 0b1011,
            "Kk"    => b.castle_state = 0b1010,
            "Kq"    => b.castle_state = 0b1001,
            "K"     => b.castle_state = 0b1000,
            "Qkq"   => b.castle_state = 0b0111,
            "Qk"    => b.castle_state = 0b0110,
            "Qq"    => b.castle_state = 0b0101,
            "Q"     => b.castle_state = 0b0100,
            "kq"    => b.castle_state = 0b0011,
            "k"     => b.castle_state = 0b0010,
            "q"     => b.castle_state = 0b0001,
            "-"     => b.castle_state = 0b0000,
            _       => b.castle_state = 16,
        }

        if fen[3].contains('-') {
            b.ep = 64;
        } else {
            for (i, sq_name) in SQ_NAMES.iter().enumerate() {
                if sq_name.contains(fen[3]) {
                    b.ep = i as u8;
                    break;
                }
            }
        }

        b.halfmove = fen[4].parse().unwrap();
        b.fullmove = fen[5].parse().unwrap();

        b
    }

    pub fn make(&mut self, m: &Move, tt: &TTable) {
        let from_to = SQUARES[m.from as usize] | SQUARES[m.to as usize];

        self.pieces[m.piece as usize] ^= from_to;
        self.util[self.colour] ^= from_to;
        self.util[2] ^= from_to;
        
        // clear ep file if there is one
        if self.ep < 64 {
            self.hash ^= tt.zorbist_array[773+ (self.ep % 8) as usize]; 
        }
        self.ep = 64;


        self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.from as usize];
        self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.to as usize];

        self.halfmove += 1;

        match &m.move_type {
            MoveType::Quiet => {
                if m.piece < 2 {
                    self.halfmove = 0;
                }
            }

            MoveType::Capture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];

                self.hash ^= tt.zorbist_array[m.xpiece as usize * 64 + m.to as usize];

                self.halfmove = 0;
            }

            MoveType::DoublePush => {
                self.ep = (m.to as usize - 8 + (self.colour * 16)) as u8;
                self.hash ^= tt.zorbist_array[773+(self.ep % 8) as usize];

                self.halfmove = 0;
            }

            MoveType::EpCapture => {
                self.pieces[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[2] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];

                self.hash ^= tt.zorbist_array[m.xpiece as usize * 64 + (m.to as usize - 8 + (self.colour * 16))];

                self.halfmove = 0;
            }
            
            MoveType::Promo => {
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];

                self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.to as usize];
                self.hash ^= tt.zorbist_array[m.promo_piece as usize * 64 + m.to as usize];

                self.halfmove = 0;
            }
            
            MoveType::PromoCapture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];

                self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.to as usize];
                self.hash ^= tt.zorbist_array[m.promo_piece as usize * 64 + m.to as usize];
                self.hash ^= tt.zorbist_array[m.xpiece as usize * 64 + m.to as usize];

                self.halfmove = 0;
            }
            
            MoveType::WKingSide => {
                self.whas_castled = true;
                self.pieces[4] ^= SQUARES[7] | SQUARES[5];
                self.util[0] ^= SQUARES[7] | SQUARES[5];
                self.util[2] ^= SQUARES[7] | SQUARES[5];

                self.hash ^= tt.zorbist_array[263]; // 4 * 64 + 7
                self.hash ^= tt.zorbist_array[261]; // 4 * 64 + 5
            }
            
            MoveType::WQueenSide => {
                self.whas_castled = true;
                self.pieces[4] ^= SQUARES[0] | SQUARES[3];
                self.util[0] ^= SQUARES[0] | SQUARES[3];
                self.util[2] ^= SQUARES[0] | SQUARES[3];

                self.hash ^= tt.zorbist_array[256]; // 4 * 64 + 0
                self.hash ^= tt.zorbist_array[259]; // 4 * 64 + 3
            }
            
            MoveType::BKingSide => {
                self.bhas_castled = true;
                self.pieces[5] ^= SQUARES[63] | SQUARES[61];
                self.util[1] ^= SQUARES[63] | SQUARES[61];
                self.util[2] ^= SQUARES[63] | SQUARES[61];

                self.hash ^= tt.zorbist_array[383]; // 5 * 64 + 63
                self.hash ^= tt.zorbist_array[381]; // 5 * 64 + 61
            }
            
            MoveType::BQueenSide => {
                self.bhas_castled = true;
                self.pieces[5] ^= SQUARES[56] | SQUARES[59];
                self.util[1] ^= SQUARES[56] | SQUARES[59];
                self.util[2] ^= SQUARES[56] | SQUARES[59];

                self.hash ^= tt.zorbist_array[376]; // 5 * 64 + 56
                self.hash ^= tt.zorbist_array[379]; // 5 * 64 + 59
            }
        }

        if m.piece as usize == 10 {
            if self.castle_state & 0b1000 > 0 {
                // println!("do K");
                self.hash ^= tt.zorbist_array[769];
            }

            if self.castle_state & 0b100 > 0 {
                // println!("do Q");
                self.hash ^= tt.zorbist_array[770];
            }
            self.castle_state &= 0b11;
        }
        else if m.piece as usize == 11 {
            if self.castle_state & 0b10 > 0{
                self.hash ^= tt.zorbist_array[771];
            }
            
            if self.castle_state & 0b1 > 0 {
                self.hash ^= tt.zorbist_array[772];
            }
            
            self.castle_state &= 0b1100;
        }
        

        if (m.from as usize == 7 || m.to as usize == 7) && self.castle_state & 0b1000 > 0 {
            self.castle_state &= 0b0111;
            self.hash ^= tt.zorbist_array[769];
        }
        if (m.from as usize == 0 || m.to as usize == 0) && self.castle_state & 0b100 > 0 {
            self.castle_state &= 0b1011;
            self.hash ^= tt.zorbist_array[770];
        }
        if (m.from as usize == 63 || m.to as usize == 63) && self.castle_state & 0b10 > 0 {
            self.castle_state &= 0b1101;
            self.hash ^= tt.zorbist_array[771];
        }
        if (m.from as usize == 56 || m.to as usize == 56) && self.castle_state & 0b1 > 0 {
            self.castle_state &= 0b1110;
            self.hash ^= tt.zorbist_array[772];
        }


        self.colour ^= 1;
        self.hash ^= tt.zorbist_array[768];

        //self.fullmove += self.colour as u8;

        self.add_prev_move();
    }

    pub fn unmake(&mut self, m: &Move, tt: &TTable) {
        self.rm_prev_move();

        //self.fullmove -= self.colour as u8;
        self.halfmove = m.last_halfmove;
        
        let from_to = SQUARES[m.from as usize] | SQUARES[m.to as usize];

        // if the castle rights of last move are different, xor the hash
        // as castle rights can only be lost, there is no need to check if a bit is present in self rather than m
        // dbg!(self.castle_state);
        if (self.castle_state ^ m.castle_rights) & 0b1000 == 8 {
            // println!("undo K");
            self.hash ^= tt.zorbist_array[769];
        }
        if (self.castle_state ^ m.castle_rights) & 0b100 == 4 {
            // println!("undo Q");
            self.hash ^= tt.zorbist_array[770];
        }
        if (self.castle_state ^ m.castle_rights) & 0b10 == 2 {
            // println!("k");
            self.hash ^= tt.zorbist_array[771];
        }
        if (self.castle_state ^ m.castle_rights) & 0b1 == 1 {
            // println!("q");
            self.hash ^= tt.zorbist_array[772];
        }

        self.castle_state = m.castle_rights;
        
        // toggle colour
        self.colour ^= 1;
        self.hash ^= tt.zorbist_array[768];

        // clear ep file in hash and set if needed
        if self.ep < 64 {
            self.hash ^= tt.zorbist_array[773+ (self.ep % 8) as usize];
        }
        if m.ep < 64 {
            self.hash ^= tt.zorbist_array[773+ (m.ep % 8) as usize];
        }
        
        self.ep = m.ep;
        
        self.pieces[m.piece as usize] ^= from_to;
        self.util[self.colour] ^= from_to;
        self.util[2] ^= from_to;

        self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.from as usize];
        self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.to as usize];

        match &m.move_type {
            MoveType::Quiet => {}

            MoveType::Capture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];

                self.hash ^= tt.zorbist_array[m.xpiece as usize * 64 + m.to as usize];
            }
            MoveType::DoublePush => {}
            MoveType::EpCapture => {
                self.pieces[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[2] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];

                self.hash ^= tt.zorbist_array[m.xpiece as usize * 64 + (m.to as usize - 8 + (self.colour * 16))];
            }
            MoveType::Promo => {
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];

                self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.to as usize];
                self.hash ^= tt.zorbist_array[m.promo_piece as usize * 64 + m.to as usize];
            }
            MoveType::PromoCapture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];

                self.hash ^= tt.zorbist_array[m.piece as usize * 64 + m.to as usize];
                self.hash ^= tt.zorbist_array[m.promo_piece as usize * 64 + m.to as usize];
                self.hash ^= tt.zorbist_array[m.xpiece as usize * 64 + m.to as usize];
            }
            MoveType::WKingSide => {
                self.whas_castled = false;
                self.pieces[4] ^= SQUARES[7] | SQUARES[5];
                self.util[0] ^= SQUARES[7] | SQUARES[5];
                self.util[2] ^= SQUARES[7] | SQUARES[5];

                self.hash ^= tt.zorbist_array[263]; // 4 * 64 + 7
                self.hash ^= tt.zorbist_array[261]; // 4 * 64 + 5
            }
            MoveType::WQueenSide => {
                self.whas_castled = false;
                self.pieces[4] ^= SQUARES[0] | SQUARES[3];
                self.util[0] ^= SQUARES[0] | SQUARES[3];
                self.util[2] ^= SQUARES[0] | SQUARES[3];

                self.hash ^= tt.zorbist_array[256]; // 4 * 64 + 0
                self.hash ^= tt.zorbist_array[259]; // 4 * 64 + 3
            }
            MoveType::BKingSide => {
                self.bhas_castled = false;
                self.pieces[5] ^= SQUARES[63] | SQUARES[61];
                self.util[1] ^= SQUARES[63] | SQUARES[61];
                self.util[2] ^= SQUARES[63] | SQUARES[61];

                self.hash ^= tt.zorbist_array[383]; // 5 * 64 + 63
                self.hash ^= tt.zorbist_array[381]; // 5 * 64 + 61
            }
            MoveType::BQueenSide => {
                self.bhas_castled = false;
                self.pieces[5] ^= SQUARES[56] | SQUARES[59];
                self.util[1] ^= SQUARES[56] | SQUARES[59];
                self.util[2] ^= SQUARES[56] | SQUARES[59];

                self.hash ^= tt.zorbist_array[376]; // 5 * 64 + 56
                self.hash ^= tt.zorbist_array[379]; // 5 * 64 + 59
            }
        }
    }


    pub fn make_no_hashing(&mut self, m: &Move) {
        let from_to = SQUARES[m.from as usize] | SQUARES[m.to as usize];

        self.pieces[m.piece as usize] ^= from_to;
        self.util[self.colour] ^= from_to;
        self.util[2] ^= from_to;
        self.ep = 64;

        match &m.move_type {
            MoveType::Quiet => {}

            MoveType::Capture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];
            }

            MoveType::DoublePush => {
                self.ep = (m.to as usize - 8 + (self.colour * 16)) as u8;
            }

            MoveType::EpCapture => {
                self.pieces[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[2] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
            }
            
            MoveType::Promo => {
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];
            }
            
            MoveType::PromoCapture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];
            }
            
            MoveType::WKingSide => {
                self.pieces[4] ^= SQUARES[7] | SQUARES[5];
                self.util[0] ^= SQUARES[7] | SQUARES[5];
                self.util[2] ^= SQUARES[7] | SQUARES[5];
            }
            
            MoveType::WQueenSide => {
                self.pieces[4] ^= SQUARES[0] | SQUARES[3];
                self.util[0] ^= SQUARES[0] | SQUARES[3];
                self.util[2] ^= SQUARES[0] | SQUARES[3];
            }
            
            MoveType::BKingSide => {
                self.pieces[5] ^= SQUARES[63] | SQUARES[61];
                self.util[1] ^= SQUARES[63] | SQUARES[61];
                self.util[2] ^= SQUARES[63] | SQUARES[61];
            }
            
            MoveType::BQueenSide => {
                self.pieces[5] ^= SQUARES[56] | SQUARES[59];
                self.util[1] ^= SQUARES[56] | SQUARES[59];
                self.util[2] ^= SQUARES[56] | SQUARES[59];
            }
        }

        // toggling castle rights
        if m.from as usize == 7 || m.to as usize == 7 {
            self.castle_state &= 0b0111;
        }
        if m.from as usize == 0 || m.to as usize == 0 {
            self.castle_state &= 0b1011;
        }
        if m.from as usize == 63 || m.to as usize == 63 {
            self.castle_state &= 0b1101;
        }
        if m.from as usize == 56 || m.to as usize == 56 {
            self.castle_state &= 0b1110;
        }
        if m.piece as usize == 10 {
            self.castle_state &= 0b11;
        }
        if m.piece as usize == 11 {
            self.castle_state &= 0b1100;
        }

        self.colour ^= 1;
        //self.fullmove += self.colour as u8;
    }

    pub fn unmake_no_hashing(&mut self, m: &Move) {
        let from_to = SQUARES[m.from as usize] | SQUARES[m.to as usize];

        self.castle_state = m.castle_rights;
        self.colour ^= 1;
        self.ep = m.ep;
        
        self.pieces[m.piece as usize] ^= from_to;
        self.util[self.colour] ^= from_to;
        self.util[2] ^= from_to;

        match &m.move_type {
            MoveType::Quiet => {}

            MoveType::Capture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];
            }
            MoveType::DoublePush => {}
            MoveType::EpCapture => {
                self.pieces[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
                self.util[2] ^= SQUARES[m.to as usize - 8 + (self.colour * 16)];
            }
            MoveType::Promo => {
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];
            }
            MoveType::PromoCapture => {
                self.pieces[m.xpiece as usize] ^= SQUARES[m.to as usize];
                self.util[1 - self.colour] ^= SQUARES[m.to as usize];
                self.util[2] ^= SQUARES[m.to as usize];
                self.pieces[self.colour] ^= SQUARES[m.to as usize];
                self.pieces[m.promo_piece as usize] ^= SQUARES[m.to as usize];
            }
            MoveType::WKingSide => {
                self.pieces[4] ^= SQUARES[7] | SQUARES[5];
                self.util[0] ^= SQUARES[7] | SQUARES[5];
                self.util[2] ^= SQUARES[7] | SQUARES[5];
            }
            MoveType::WQueenSide => {
                self.pieces[4] ^= SQUARES[0] | SQUARES[3];
                self.util[0] ^= SQUARES[0] | SQUARES[3];
                self.util[2] ^= SQUARES[0] | SQUARES[3];
            }
            MoveType::BKingSide => {
                self.pieces[5] ^= SQUARES[63] | SQUARES[61];
                self.util[1] ^= SQUARES[63] | SQUARES[61];
                self.util[2] ^= SQUARES[63] | SQUARES[61];
            }
            MoveType::BQueenSide => {
                self.pieces[5] ^= SQUARES[56] | SQUARES[59];
                self.util[1] ^= SQUARES[56] | SQUARES[59];
                self.util[2] ^= SQUARES[56] | SQUARES[59];
            }
        }
    }


    // designed to get original hash or only hash in certain circumstances 
    // (ie not very quick compared to incremental update of the board) 
    pub fn get_hash(&self, tt: &TTable) -> u64 {
        let mut hash: u64 = 0;

        for piece in 0..12 {
            for rank in 0..8{
                for file in 0..8 {
                    if (self.pieces[piece] & SQUARES[rank*8+file]) > 0 {
                        hash ^= tt.zorbist_array[piece*64 + rank*8 + file];
                    }
                }
            }
        } 

        hash ^= if self.colour == 1 { tt.zorbist_array[768] } else { 0 };
        
        if self.castle_state & 0b1000 == 8{
            hash ^= tt.zorbist_array[769];
        }
        if self.castle_state & 0b100 == 4{
            hash ^= tt.zorbist_array[770];
        }
        if self.castle_state & 0b10 == 2{
            hash ^= tt.zorbist_array[771];
        }
        if self.castle_state & 0b1 == 1{
            hash ^= tt.zorbist_array[772];
        }

        if self.ep < 64 {
            hash ^= tt.zorbist_array[773+(self.ep - (self.ep / 8)) as usize];
        }

        hash
    }

    pub fn add_prev_move(&mut self){
        self.prev_moves[(self.hash & PREV_MOVE_MASK) as usize] += 1;
    }

    pub fn rm_prev_move(&mut self){
        self.prev_moves[(self.hash & PREV_MOVE_MASK) as usize] -= 1;
    }

    // returns true if 3 move rep or 50 move rule occurs in this pos 
    pub fn is_bad_pos(&self) -> bool {
        self.prev_moves[(self.hash & PREV_MOVE_MASK) as usize] == 3 || self.halfmove >= 100
    }
}


impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        for i in (1..9).rev() {
            let s = i.to_string();
            out.push_str(&s);
            out.push_str("    ");
            for j in i * 8 - 8..i * 8 {
                if (SQUARES[j] & self.pieces[0]) > 0 {
                    out.push_str("P ");
                }
                if (SQUARES[j] & self.pieces[1]) > 0 {
                    out.push_str("p ");
                }
                if (SQUARES[j] & self.pieces[2]) > 0 {
                    out.push_str("N ");
                }
                if (SQUARES[j] & self.pieces[3]) > 0 {
                    out.push_str("n ");
                }
                if (SQUARES[j] & self.pieces[4]) > 0 {
                    out.push_str("R ");
                }
                if (SQUARES[j] & self.pieces[5]) > 0 {
                    out.push_str("r ");
                }
                if (SQUARES[j] & self.pieces[6]) > 0 {
                    out.push_str("B ");
                }
                if (SQUARES[j] & self.pieces[7]) > 0 {
                    out.push_str("b ");
                }
                if (SQUARES[j] & self.pieces[8]) > 0 {
                    out.push_str("Q ");
                }
                if (SQUARES[j] & self.pieces[9]) > 0 {
                    out.push_str("q ");
                }
                if (SQUARES[j] & self.pieces[10]) > 0 {
                    out.push_str("K ");
                }
                if (SQUARES[j] & self.pieces[11]) > 0 {
                    out.push_str("k ");
                }
                if (SQUARES[j] & self.util[2]) == 0 {
                    out.push_str("- ");
                }
            }
            out.push('\n');
        }
        out.push_str("\n     A B C D E F G H\n");
        write!(f, "{}", out)
    }
}
