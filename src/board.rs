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
#[derive(Copy, Clone)]
enum Piece {
    WhitePawn = 0, WhiteKnight = 1, WhiteBishop = 2, WhiteRook = 3, WhiteQueen = 4, WhiteKing = 5,
    BlackPawn = 6, BlackKnight = 7, BlackBishop = 8, BlackRook = 9, BlackQueen = 10, BlackKing = 11,
}

impl Piece {
    pub fn is_white(&self) -> bool {
        (*self as usize) < 6
    }

    pub fn enemy_indices(&self) -> std::ops::Range<usize> {
        if self.is_white() { 6..12 } else { 0..6 }
    }
}

struct Board{
    bitboards: [u64; 12],
}
impl Index<Piece> for Board {
    type Output = u64;

    fn index(&self, piece: Piece) -> &Self::Output {
        &self.bitboards[piece as usize] 
    }
}
impl IndexMut<Piece> for Board {
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        &mut self.bitboards[piece as usize]
    }
}
impl Board{
pub fn new() -> Self {
        let mut board = Board { bitboards: [0; 12] };
        
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
    pub fn get_attacked_squares_white(&self) -> u64{
        let mut attacked_squares: u64 = 0;
        
    }
}
