#![allow(dead_code)]

use std::ops::{Index, IndexMut};
use moves::Move;
//-----WHITE PIECES (Ranks 1 & 2)-----
const INITIAL_WHITE_PAWNS_MASK: u64   = 0x0000_0000_0000_FF00;
const INITIAL_WHITE_ROOKS_MASK: u64   = 0x0000_0000_0000_0081;
const INITIAL_WHITE_KNIGHTS_MASK: u64 = 0x0000_0000_0000_0042;
const INITIAL_WHITE_BISHOPS_MASK: u64 = 0x0000_0000_0000_0024;
const INITIAL_WHITE_QUEENS_MASK: u64  = 0x0000_0000_0000_0008;
const INITIAL_WHITE_KING_MASK: u64    = 0x0000_0000_0000_0010;

//-----BLACK PIECES (Ranks 7 & 8)-----
const INITIAL_BLACK_PAWNS_MASK: u64   = 0x00FF_0000_0000_0000;
const INITIAL_BLACK_ROOKS_MASK: u64   = 0x8100_0000_0000_0000;
const INITIAL_BLACK_KNIGHTS_MASK: u64 = 0x4200_0000_0000_0000;
const INITIAL_BLACK_BISHOPS_MASK: u64 = 0x2400_0000_0000_0000;
const INITIAL_BLACK_QUEENS_MASK: u64  = 0x0800_0000_0000_0000;
const INITIAL_BLACK_KING_MASK: u64    = 0x1000_0000_0000_0000;

//HELPFUL CONSTANTS

//-----FILES (Vertical columns A through H)-----
const FILE_A_MASK: u64 = 0x0101_0101_0101_0101;
const FILE_B_MASK: u64 = 0x0202_0202_0202_0202;
const FILE_C_MASK: u64 = 0x0404_0404_0404_0404;
const FILE_D_MASK: u64 = 0x0808_0808_0808_0808;
const FILE_E_MASK: u64 = 0x1010_1010_1010_1010;
const FILE_F_MASK: u64 = 0x2020_2020_2020_2020;
const FILE_G_MASK: u64 = 0x4040_4040_4040_4040;
const FILE_H_MASK: u64 = 0x8080_8080_8080_8080;

//-----RANKS (Horizontal rows 1 through 8)-----
const RANK_1_MASK: u64 = 0x0000_0000_0000_00FF;
const RANK_2_MASK: u64 = 0x0000_0000_0000_FF00;
const RANK_3_MASK: u64 = 0x0000_0000_00FF_0000;
const RANK_4_MASK: u64 = 0x0000_0000_FF00_0000;
const RANK_5_MASK: u64 = 0x0000_00FF_0000_0000;
const RANK_6_MASK: u64 = 0x0000_FF00_0000_0000;
const RANK_7_MASK: u64 = 0x00FF_0000_0000_0000;
const RANK_8_MASK: u64 = 0xFF00_0000_0000_0000;

const FILE_MASKS: [u64; 8] = [
    FILE_A_MASK,
    FILE_B_MASK,
    FILE_C_MASK,
    FILE_D_MASK,
    FILE_E_MASK,
    FILE_F_MASK,
    FILE_G_MASK,
    FILE_H_MASK
];
const RANK_MASKS: [u64; 8] = [
    RANK_1_MASK,
    RANK_2_MASK,
    RANK_3_MASK,
    RANK_4_MASK,
    RANK_5_MASK,
    RANK_6_MASK,
    RANK_7_MASK,
    RANK_8_MASK
];
pub struct UndoInfo{
    captured_piece: Option<Piece>,
    previous_castling_rights: CastlingRights,
    previous_en_passant_target: Option<(usize, usize)>,
    captured_square: Option<(usize,usize)>,
}
impl UndoInfo{
    pub fn new(captured_piece: Option<Piece>, 
        previous_castling_rights: CastlingRights, 
        previous_en_passant_target: Option<(usize, usize)>,
        captured_square: Option<(usize,usize)>) -> Self{
        UndoInfo{
            captured_piece,
            previous_castling_rights,
            previous_en_passant_target,
            captured_square,
        }
    }
}
#[derive(Copy, Clone)]
pub enum Piece {
    WhitePawn = 0, WhiteKnight = 1, WhiteBishop = 2, WhiteRook = 3, WhiteQueen = 4, WhiteKing = 5,
    BlackPawn = 6, BlackKnight = 7, BlackBishop = 8, BlackRook = 9, BlackQueen = 10, BlackKing = 11,
}

impl Piece {
    pub fn all_of_color(white: bool) -> [Piece; 6] {
       if white {
          [Piece::WhitePawn, Piece::WhiteKnight, Piece::WhiteBishop,
            Piece::WhiteRook, Piece::WhiteQueen, Piece::WhiteKing]
       } else {
           [Piece::BlackPawn, Piece::BlackKnight, Piece::BlackBishop,
            Piece::BlackRook, Piece::BlackQueen, Piece::BlackKing]
       }
   }
    pub fn is_white(&self) -> bool {
        (*self as usize) < 6
    }

    pub fn enemy_indices(&self) -> std::ops::Range<usize> {
        if self.is_white() { 6..12 } else { 0..6 }
    }
    pub fn friendly_indices(&self) -> std::ops::Range<usize> {
        if self.is_white() { 0..6 } else { 6..12 }
    }
    pub fn from_usize(value: usize) -> Option<Piece> {
        match value {
            0 => Some(Piece::WhitePawn),
            1 => Some(Piece::WhiteKnight),
            2 => Some(Piece::WhiteBishop),
            3 => Some(Piece::WhiteRook),
            4 => Some(Piece::WhiteQueen),
            5 => Some(Piece::WhiteKing),
            6 => Some(Piece::BlackPawn),
            7 => Some(Piece::BlackKnight),
            8 => Some(Piece::BlackBishop),
            9 => Some(Piece::BlackRook),
            10 => Some(Piece::BlackQueen),
            11 => Some(Piece::BlackKing),
            _ => None, // Fails safely if the number is out of bounds!
        }
    }
}
#[derive(Copy, Clone)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}
#[derive(Copy, Clone)]
pub struct Board{
    pub bitboards: [u64; 12],
    pub en_passant_target: Option<(usize, usize)>,
    pub castling_rights: CastlingRights,
}

impl Index<Piece> for Board {
    type Output = u64;

    fn index(&self, piece: Piece) -> &Self::Output {
        &self.bitboards[piece as usize] 
    }
}
impl Index<usize> for Board {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bitboards[index]
    }
}
impl IndexMut<usize> for Board {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bitboards[index]
    }
}
impl IndexMut<Piece> for Board {
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        &mut self.bitboards[piece as usize]
    }
}
impl Board{
    pub fn get_black_indices() -> std::ops::Range<usize> {
        5..11
    }
    pub fn get_white_indices() -> std::ops::Range<usize> {
        0..5
    }
    pub fn new() -> Self {
        let mut board = Board { bitboards: [0; 12], en_passant_target: None, 
            castling_rights: CastlingRights { white_kingside: true, 
                                                white_queenside: true, black_kingside: true, 
                                                black_queenside: true } };
        
        board[Piece::WhitePawn]   = INITIAL_WHITE_PAWNS_MASK;
        board[Piece::WhiteKnight] = INITIAL_WHITE_KNIGHTS_MASK;
        board[Piece::WhiteBishop] = INITIAL_WHITE_BISHOPS_MASK;
        board[Piece::WhiteRook]   = INITIAL_WHITE_ROOKS_MASK;
        board[Piece::WhiteQueen]  = INITIAL_WHITE_QUEENS_MASK;
        board[Piece::WhiteKing]   = INITIAL_WHITE_KING_MASK;
        
        board[Piece::BlackPawn]   = INITIAL_BLACK_PAWNS_MASK;
        board[Piece::BlackKnight] = INITIAL_BLACK_KNIGHTS_MASK;
        board[Piece::BlackBishop] = INITIAL_BLACK_BISHOPS_MASK;
        board[Piece::BlackRook]   = INITIAL_BLACK_ROOKS_MASK;
        board[Piece::BlackQueen]  = INITIAL_BLACK_QUEENS_MASK;
        board[Piece::BlackKing]   = INITIAL_BLACK_KING_MASK;

        board
    }
    pub fn print_board(&self){
        for rank in (0..8).rev(){
            for file in 0..8{
                let square_index = rank * 8 + file;
                let square_mask = 1u64 << square_index;
                if self[Piece::WhitePawn] & square_mask != 0{
                    print!("P ");
                } else if self[Piece::WhiteKnight] & square_mask != 0{
                    print!("N ");
                } else if self[Piece::WhiteBishop] & square_mask != 0{
                    print!("B ");
                } else if self[Piece::WhiteRook] & square_mask != 0{
                    print!("R ");
                } else if self[Piece::WhiteQueen] & square_mask != 0{
                    print!("Q ");
                } else if self[Piece::WhiteKing] & square_mask != 0{
                    print!("K ");
                } else if self[Piece::BlackPawn] & square_mask != 0{
                    print!("p ");
                } else if self[Piece::BlackKnight] & square_mask != 0{
                    print!("n ");
                } else if self[Piece::BlackBishop] & square_mask != 0{
                    print!("b ");
                } else if self[Piece::BlackRook] & square_mask != 0{
                    print!("r ");
                } else if self[Piece::BlackQueen] & square_mask != 0{
                    print!("q ");
                } else if self[Piece::BlackKing] & square_mask != 0{
                    print!("k ");
                } else {
                    print!(". ");
                }
            }
            println!();
        }
    }
    pub fn get_occupancy_mask(&self) -> u64{
        self[Piece::WhitePawn] | self[Piece::WhiteKnight] | self[Piece::WhiteBishop] | self[Piece::WhiteRook] | self[Piece::WhiteQueen] | self[Piece::WhiteKing] |
        self[Piece::BlackPawn] | self[Piece::BlackKnight] | self[Piece::BlackBishop] | self[Piece::BlackRook] | self[Piece::BlackQueen] | self[Piece::BlackKing]
    }
    pub fn get_white_occupancy_mask(&self) -> u64{
        self[Piece::WhitePawn] | self[Piece::WhiteKnight] | self[Piece::WhiteBishop] | self[Piece::WhiteRook] | self[Piece::WhiteQueen] | self[Piece::WhiteKing]
    }
    pub fn get_black_occupancy_mask(&self) -> u64{
        self[Piece::BlackPawn] | self[Piece::BlackKnight] | self[Piece::BlackBishop] | self[Piece::BlackRook] | self[Piece::BlackQueen] | self[Piece::BlackKing]
    }

    pub fn get_piece_at(&self, square: (usize, usize), current_player: bool) -> Option<usize> {
        let square_index = square.0 * 8 + square.1;
        let square_mask = 1u64 << square_index;
        let piece_range = if current_player {
            0..6
        } else {
            6..12
        };
        for (piece, &board) in self.bitboards.iter().enumerate().skip(piece_range.start).take(piece_range.end - piece_range.start) {
            if board & square_mask != 0 {
                return Some(piece);
            }
        }
        None
    }
  pub fn make_move(&mut self, mv: &Move) -> UndoInfo {
        // Detect en passant the same way execute_move does: a pawn moving
        // diagonally onto a square that's empty *before* the move happens.
        let is_en_passant = matches!(mv.piece.piece_type, Piece::WhitePawn | Piece::BlackPawn)
            && mv.piece.from_file != mv.to.1
            && self.get_any_piece_at(mv.to).is_none();
 
        let (captured_piece, captured_square) = if is_en_passant {
            let is_white = mv.piece.piece_type.is_white();
            let captured_rank = if is_white { mv.to.0 - 1 } else { mv.to.0 + 1 };
            let square = (captured_rank, mv.to.1);
            (self.get_any_piece_at(square), Some(square))
        } else {
            (self.get_any_piece_at(mv.to), Some(mv.to))
        };
 
        let undo = UndoInfo {
            captured_piece,
            captured_square: if captured_piece.is_some() { captured_square } else { None },
            previous_castling_rights: self.castling_rights.clone(),
            previous_en_passant_target: self.en_passant_target,
        };
 
        mv.execute_move(self);
        undo
    }
 
    /// Reverses exactly the move `make_move` just applied, using the
    /// snapshot it handed back. Must be called with the *same* `mv` and
    /// the `UndoInfo` `make_move` returned for it, in that order.
    pub fn unmake_move(&mut self, mv: &Move, undo: UndoInfo) {
        // Whatever is actually sitting on `to` right now might be a promoted
        // piece, not the original mover -- clear the right one.
        let landing_piece = mv.promotion.unwrap_or(mv.piece.piece_type);
        let to_mask = 1u64 << (mv.to.0 * 8 + mv.to.1);
        let from_mask = 1u64 << (mv.piece.from_rank * 8 + mv.piece.from_file);
 
        self[landing_piece] &= !to_mask;          // remove whatever landed on `to`
        self[mv.piece.piece_type] |= from_mask;    // put the original piece back on `from`
 
        // Restore whatever was captured, on whichever square it actually
        // came from (matters for en passant, where that's not `to`).
        if let (Some(piece), Some(square)) = (undo.captured_piece, undo.captured_square) {
            let mask = 1u64 << (square.0 * 8 + square.1);
            self[piece] |= mask;
        }
 
        // Undo castling: move the rook back too.
        if matches!(mv.piece.piece_type, Piece::WhiteKing | Piece::BlackKing)
            && (mv.piece.from_file as isize - mv.to.1 as isize).abs() == 2
        {
            let is_white = mv.piece.piece_type.is_white();
            let rank = mv.piece.from_rank;
            let kingside = mv.to.1 == 6;
            let rook_piece = if is_white { Piece::WhiteRook } else { Piece::BlackRook };
            let (rook_from, rook_to) = if kingside { (7, 5) } else { (0, 3) };
            self[rook_piece] &= !(1u64 << (rank * 8 + rook_to));
            self[rook_piece] |= 1u64 << (rank * 8 + rook_from);
        }
 
        // Restore state that can't be derived from the move alone.
        self.castling_rights = undo.previous_castling_rights;
        self.en_passant_target = undo.previous_en_passant_target;
    }
    pub fn get_any_piece_at(&self, square: (usize, usize)) -> Option<Piece> {
        let square_index = square.0 * 8 + square.1;
        let square_mask = 1u64 << square_index;
        
        for piece_val in 0..12 {
            if self.bitboards[piece_val] & square_mask != 0 {
                return Piece::from_usize(piece_val);
            }
        }
        None
    }
}
