use std::fmt;
use crate::chess::moves::{ MoveType, Move };
use crate::chess::SQUARES;
use crate::chess::movegen;

#[derive(Copy, Clone)]
pub struct Board {
    pub pieces: [u64; 12],
    pub util: [u64; 3],

    pub is_white: bool,
    pub colour: usize,

    pub ep: u8,
    
    pub castle_state: u8,

    pub halfmove: u16,
    pub fullmove: u16,
}

impl Board {
    pub fn new() -> Board {
        let mut b = Board {
            pieces: [0,0,0,0,0,0,0,0,0,0,0,0,],
            util: [0,0,0],
            is_white: true,
            colour: 0,
            ep: 0,
            castle_state: 0b1111,
            halfmove: 0,
            fullmove: 0, 
        };
            b.pieces[0] = 0b0000000000000000000000000000000000000000000000001111111100000000; //wp 0
            b.pieces[2] = 0b0000000000000000000000000000000000000000000000000000000001000010; //wn 2
            b.pieces[4] = 0b0000000000000000000000000000000000000000000000000000000010000001; //wr 4
            b.pieces[6] = 0b0000000000000000000000000000000000000000000000000000000000100100;	//wb 6
            b.pieces[8] = 0b0000000000000000000000000000000000000000000000000000000000001000; //wq 8
            b.pieces[10] = 0b0000000000000000000000000000000000000000000000000000000000010000;	//wk 10
            
            b.pieces[1] = 0b0000000011111111000000000000000000000000000000000000000000000000;	//bp 1
            b.pieces[3] = 0b0100001000000000000000000000000000000000000000000000000000000000;	//bn 3
            b.pieces[5] = 0b1000000100000000000000000000000000000000000000000000000000000000;	//br 5
            b.pieces[7] = 0b0010010000000000000000000000000000000000000000000000000000000000;	//bb 7
            b.pieces[9] = 0b0000100000000000000000000000000000000000000000000000000000000000;	//bq 9
            b.pieces[11] = 0b0001000000000000000000000000000000000000000000000000000000000000;	//bk 11
            
            // util[0] all white, util[1] all black, util[2] all pieces
            b.util[0] = b.pieces[0] | b.pieces[2] | b.pieces[4] | b.pieces[6] | b.pieces[8] | b.pieces[10];
            b.util[1] = b.pieces[1] | b.pieces[3] | b.pieces[5] | b.pieces[7] | b.pieces[9] | b.pieces[11];
            b.util[2] = b.util[0] | b.util[1];
            
        b
    }

    // pub fn new_from_fen(fen: &str) -> Board {
    //     let mut b = Board {
    //         pieces: [0,0,0,0,0,0,0,0,0,0,0,0,],
    //         util: [0,0,0],
    //         is_white: true,
    //         colour: 0,
    //         ep: 0,
    //         castle_state: 0b1111,
    //         halfmove: 0,
    //         fullmove: 0, 
    //     };
    //     b
    // }
    
    // pub fn new_from_moves(moves: &str) -> Board {
    //     let mut b = Board {
    //         pieces: [0,0,0,0,0,0,0,0,0,0,0,0,],
    //         util: [0,0,0],
    //         is_white: true,
    //         colour: 0,
    //         ep: 0,
    //         castle_state: 0b1111,
    //         halfmove: 0,
    //         fullmove: 0, 
    //     };
    //     b
    // }

    // pub fn print_board(&self, overwrite: bool) {
        
    // }

    pub fn make(&mut self, m: &Move){
        let from_to = SQUARES[m.from] | SQUARES[m.to];

        self.pieces[m.piece] ^= from_to;
        self.util[self.colour] ^= from_to;
        self.util[2] ^= from_to;
        self.ep = 64;

        match &m.move_type {
            MoveType::Quiet => {},

            MoveType::Capture => {
                //println!("{}\n", m);
                self.pieces[m.xpiece] ^= SQUARES[m.to];
                self.util[1 - self.colour] ^= SQUARES[m.to];
                self.util[2] ^= SQUARES[m.to];
            },

            MoveType::DoublePush => {
                self.ep = (m.to - 8 + (self.colour * 16)) as u8;
            },
            
            MoveType::EpCapture => {
                self.pieces[1 - self.colour] ^= SQUARES[m.to - 8 + (self.colour * 16)];
                self.util[1 - self.colour] ^= SQUARES[m.to - 8 + (self.colour * 16)];
                self.util[2] ^= SQUARES[m.to - 8 + (self.colour * 16)];
            },
            
            MoveType::Promo => {
                self.pieces[self.colour] ^= SQUARES[m.to];
                self.pieces[m.promo_piece] ^= SQUARES[m.to];
            },
            
            MoveType::PromoCapture => {
                self.pieces[m.xpiece] ^= SQUARES[m.to];
                self.util[1 - self.colour] ^= SQUARES[m.to];
                self.util[2] ^= SQUARES[m.to];
                self.pieces[self.colour] ^= SQUARES[m.to];
                self.pieces[m.promo_piece] ^= SQUARES[m.to];
            },
            
            MoveType::WKingSide => {
                self.pieces[4] ^= SQUARES[7] | SQUARES[5];
			    self.util[0] ^= SQUARES[7] | SQUARES[5];
			    self.util[2] ^= SQUARES[7] | SQUARES[5];
            },
            
            MoveType::WQueenSide => {
                self.pieces[4] ^= SQUARES[0] | SQUARES[3];
                self.util[0] ^= SQUARES[0] | SQUARES[3];
                self.util[2] ^= SQUARES[0] | SQUARES[3];
            },
            
            MoveType::BKingSide => {
                self.pieces[5] ^= SQUARES[63] | SQUARES[61];
                self.util[1] ^= SQUARES[63] | SQUARES[61];
                self.util[2] ^= SQUARES[63] | SQUARES[61];
            },
            
            MoveType::BQueenSide => {
                self.pieces[5] ^= SQUARES[56] | SQUARES[59];
                self.util[1] ^= SQUARES[56] | SQUARES[59];
                self.util[2] ^= SQUARES[56] | SQUARES[59];
            },
        }

        if m.from == 7 || m.to == 7 {
            self.castle_state &= 0b0111;
        } 
        if m.from == 0 || m.to == 0 {
            self.castle_state &= 0b1011;
        } 
        if m.from == 63 || m.to == 63 {
            self.castle_state &= 0b1101;
        } 
        if m.from == 56 || m.to == 56 {
            self.castle_state &= 0b1110;
        } 
        if m.piece == 10 {
            self.castle_state &= 0b11;
        } 
        if m.piece == 11 {
            self.castle_state &= 0b1100;
        }

        //self.fullmove += self.colour as u16;
        self.colour = if self.colour == 0 {
            1
        } else {
            0
        };
    }

    pub fn unmake(&mut self, m: &Move) {
        let from_to = SQUARES[m.from] | SQUARES[m.to];

        self.castle_state = m.castle_rights;
        self.colour = if self.colour == 0 {
            1
        } else {
            0
        };
        self.ep = m.ep;
        
        self.pieces[m.piece] ^= from_to;
        self.util[self.colour] ^= from_to;
        // movegen::print_bb(self.util[2], self);
        self.util[2] ^= from_to;
        // movegen::print_bb(self.util[2], self);

        match &m.move_type {
            MoveType::Quiet => {},

            MoveType::Capture => {
                self.pieces[m.xpiece] ^= SQUARES[m.to];
                self.util[1 - self.colour] ^= SQUARES[m.to];
                self.util[2] ^= SQUARES[m.to];
            }
            MoveType::DoublePush => {},
            
            MoveType::EpCapture => {
                self.pieces[1 - self.colour] ^= SQUARES[m.to - 8 + (self.colour * 16)];
                self.util[1 - self.colour] ^= SQUARES[m.to - 8 + (self.colour * 16)];
                self.util[2] ^= SQUARES[m.to - 8 + (self.colour * 16)];
            },
            
            MoveType::Promo => {
                self.pieces[self.colour] ^= SQUARES[m.to];
                self.pieces[m.promo_piece] ^= SQUARES[m.to];
            },
            
            MoveType::PromoCapture => {
                self.pieces[m.xpiece] ^= SQUARES[m.to];
                self.util[1 - self.colour] ^= SQUARES[m.to];
                self.util[2] ^= SQUARES[m.to];
                self.pieces[self.colour] ^= SQUARES[m.to];
                self.pieces[m.promo_piece] ^= SQUARES[m.to];
            },
            
            MoveType::WKingSide => {
                self.pieces[4] ^= SQUARES[7] | SQUARES[5];
			    self.util[0] ^= SQUARES[7] | SQUARES[5];
			    self.util[2] ^= SQUARES[7] | SQUARES[5];
            },
            
            MoveType::WQueenSide => {
                self.pieces[4] ^= SQUARES[0] | SQUARES[3];
                self.util[0] ^= SQUARES[0] | SQUARES[3];
                self.util[2] ^= SQUARES[0] | SQUARES[3];
            },
            
            MoveType::BKingSide => {
                self.pieces[5] ^= SQUARES[63] | SQUARES[61];
                self.util[1] ^= SQUARES[63] | SQUARES[61];
                self.util[2] ^= SQUARES[63] | SQUARES[61];
            },
            
            MoveType::BQueenSide => {
                self.pieces[5] ^= SQUARES[56] | SQUARES[59];
                self.util[1] ^= SQUARES[56] | SQUARES[59];
                self.util[2] ^= SQUARES[56] | SQUARES[59];
            },
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        
        for i in (1..9).rev() {    //(int i = 8; i > 0; i--) {
            let s = i.to_string();
            out.push_str(&s);
            out.push_str("    ");
            for j in i*8-8..i*8 {            //(int j = (i * 8 - 8); j < (i * 8); j++) {
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
            out.push_str("\n");
        }
        out.push_str("\n     A B C D E F G H\n");
        
        write!(f, "{}", out)
    }
}
