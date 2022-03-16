use std::fmt;

use crate::Board;
use crate::board_info::SQ_NAMES;
use crate::movegen;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MoveType {
    Quiet,
    DoublePush,
    Capture,
    EpCapture,
    WKingSide,
    WQueenSide,
    BKingSide,
    BQueenSide,
    Promo,
    PromoCapture,
}

impl fmt::Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", 
            match self {
                MoveType::Quiet => "Quiet",
                MoveType::DoublePush => "DoublePush",
                MoveType::Capture => "Capture",
                MoveType::EpCapture => "EpCapture",
                MoveType::WKingSide => "WKingSide",
                MoveType::WQueenSide => "WQueenSide",
                MoveType::BKingSide => "BKingSide",
                MoveType::BQueenSide => "BQueenSide",
                MoveType::Promo => "Promo",
                MoveType::PromoCapture => "PromoCapture",
            }
        )
    }
}

#[derive(Debug, Copy ,Clone, PartialEq)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub piece: usize,

    pub move_type: MoveType,

    pub ep: u8,    
    pub xpiece: usize,
    pub castle_rights: u8,
    pub promo_piece: usize,

    pub last_halfmove: usize,

}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, 
"from: {}    to: {}
piece: {}    move type: {}
ep: {}    xpiece: {}    castle rights: {}    promo piece: {}", 
        self.from, self.to, self.piece,
        self.move_type, self.ep, self.xpiece, 
        self.castle_rights, self.promo_piece)
    }
}

impl Move {
    pub fn new_quiet(from: usize, to: usize, 
            piece: usize, ep: u8, castle_rights: u8, last_halfmove: usize) -> Move {
        
        Move { 
            from, 
            to, 
            piece,             
            move_type: MoveType::Quiet,

            ep, 
            xpiece: 12, 
            castle_rights,
            promo_piece: 12,
            last_halfmove,
        }
    }

    pub fn new_capture(from: usize, to: usize, 
            piece: usize, xpiece: usize, ep: u8, castle_rights: u8, last_halfmove: usize) -> Move {
        
        Move { 
            from, 
            to, 
            piece, 
            
            move_type: MoveType::Capture,
            ep, 
            
            xpiece, 
            castle_rights,
            promo_piece: 12,
            last_halfmove,
        }    
    }

    pub fn new_double_push(from: usize, to: usize, 
            piece: usize, ep: u8, castle_rights: u8, last_halfmove: usize) -> Move {
        Move { 
            from, 
            to, 
            piece, 
            move_type: MoveType::DoublePush,

            ep, 
            xpiece: 12, 
            castle_rights,
            promo_piece: 12,
            last_halfmove,
        }                
    }

    pub fn new_ep_capture(from: usize, to: usize, 
            piece: usize, xpiece: usize, ep: u8, castle_rights: u8, last_halfmove: usize) -> Move {
        Move { 
            from, 
            to, 
            piece, 
            move_type: MoveType::EpCapture,
            
            ep, 
            xpiece, 
            castle_rights,
            promo_piece: 12,
            last_halfmove,
        }
    }
    pub fn new_promo(from: usize, to: usize, 
            piece: usize, ep: u8, castle_rights: u8, last_halfmove: usize, promo_piece: usize) -> Move {

        Move { 
            from, 
            to, 
            piece, 
            move_type: MoveType::Promo,

            ep, 
            xpiece: 12, 
            castle_rights,
            promo_piece,
            last_halfmove,
        }
    }
    
    pub fn new_promo_capture(from: usize, to: usize, 
            piece: usize, xpiece: usize, ep: u8, castle_rights: u8, last_halfmove: usize, promo_piece: usize) -> Move {

        Move { 
            from, 
            to, 
            piece, 
            move_type: MoveType::PromoCapture,

            ep, 
            xpiece, 
            castle_rights,
            promo_piece,
            last_halfmove,
        }
}

    
    pub fn new_castle(from: usize, to: usize, 
            piece: usize, ep: u8, castle_rights: u8, last_halfmove: usize, castle_move: MoveType) -> Move {
        Move {
            from, 
            to, 
            piece, 
            move_type: castle_move,

            ep,
            xpiece: 12,             
            castle_rights,
            promo_piece: 12,
            last_halfmove,
        }
    } 

    pub fn new_from_text(text: &str, b: &Board) -> Move {
        let from = sq_from_text(&text[0..2]);
        let to = sq_from_text(&text[2..4]);
        
        let promo = if text.len() == 5 {
            Some(promo_piece_from_text(&text[4..]) + b.colour)
        } else {
            None
        };

        let promo_piece = match promo {
            Some(x) => x,
            None => 12
        };


        let piece = movegen::get_piece(from, &b);
        let xpiece = movegen::get_xpiece(to, &b);

        let mut move_type = MoveType::Quiet;

        if piece < 2 && (from as i32 - to as i32).abs() == 16 {
            move_type = MoveType::DoublePush;
        } else if piece == 10 {
            let diff = from as i32 - to as i32;
            if diff == -2 {
                move_type = MoveType::WKingSide;
            } else if diff == 2{
                move_type = MoveType::WQueenSide;
            }
        } else if piece == 11 {
            let diff = from as i32 - to as i32;
            if diff == -2 {
                move_type = MoveType::BKingSide;
            } else if diff == 2{
                move_type = MoveType::BQueenSide;
            }
        }

        if xpiece < 12 && promo_piece < 12 {
            move_type = MoveType::PromoCapture;
        } else if promo_piece < 12 {
            move_type = MoveType::Promo;
        } else if xpiece < 2 && to as u8 == b.ep {
            move_type = MoveType::EpCapture;
        } else if xpiece < 12 {
            move_type = MoveType::Capture;
        }

        Move { 
            from, 
            to, 
            piece, 
            move_type, 
            ep: b.ep, 
            xpiece, 
            castle_rights: 
            b.castle_state, 
            promo_piece, 
            last_halfmove: b.halfmove 
        }
    }

    pub fn as_uci_string(&self) -> String {
        let mut m = String::new();

        m.push_str(SQ_NAMES[self.from]);
        m.push_str(SQ_NAMES[self.to]);
        m.push_str(&text_from_promo_piece(self.promo_piece));
        m
    }
}

fn sq_from_text(sq: &str) -> usize {
    let sq = sq.as_bytes();

    ((sq[0] - "a".as_bytes()[0]) + (8 * (sq[1] - "1".as_bytes()[0]))) as usize
}

fn promo_piece_from_text(p: &str) -> usize {
    match p {
        "n" => 2,
        "r" => 4,
        "b" => 6,
        "q" => 8,
        _ => 12
    }
}

fn text_from_promo_piece(promo_piece: usize) -> String {
    match promo_piece {
        2 | 3 => String::from("n"),
        4 | 5 => String::from("r"),
        6 | 7 => String::from("b"),
        8 | 9 => String::from("q"),
        _ => String::from("")
    }
}