use crate::board::Board;
use crate::board::Piece;

pub struct ActivePiece {
    pub piece_type: Piece,
    pub from_rank: usize,
    pub from_file: usize,
}

pub struct Move {
    pub piece: ActivePiece,
    pub to: (usize, usize),
}

impl Move {
    pub fn check_legality(&self, board: &Board) -> bool {
        let friendly_occupied = if self.piece.piece_type.is_white() {
            board.get_white_occupancy_mask()
        } else {
            board.get_black_occupancy_mask()
        };

        let mask_for_checking: u64 = 1 << (self.to.0 * 8 + self.to.1);
        
        if friendly_occupied & mask_for_checking != 0 {
            return false;
        }

        let rank_diff = (self.piece.from_rank as isize - self.to.0 as isize).abs();
        let file_diff = (self.piece.from_file as isize - self.to.1 as isize).abs();

        match self.piece.piece_type {
            Piece::WhitePawn => {
                let expected_rank = self.piece.from_rank + 1;
                expected_rank == self.to.0 && self.piece.from_file == self.to.1
            },
            Piece::BlackPawn => {
                let expected_rank = self.piece.from_rank - 1;
                expected_rank == self.to.0 && self.piece.from_file == self.to.1
            },
            Piece::WhiteKnight | Piece::BlackKnight => {
                (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
            },
            Piece::WhiteBishop | Piece::BlackBishop => {
                rank_diff == file_diff
            },
            Piece::WhiteRook | Piece::BlackRook => {
                self.piece.from_rank == self.to.0 || self.piece.from_file == self.to.1
            },
            Piece::WhiteQueen | Piece::BlackQueen => {
                self.piece.from_rank == self.to.0 || self.piece.from_file == self.to.1 || rank_diff == file_diff
            },
            Piece::WhiteKing | Piece::BlackKing => {
                rank_diff <= 1 && file_diff <= 1
            },
        }
    }

    pub fn execute_move(&self, board: &mut Board) {
        let curr_position_mask = 1u64 << (self.piece.from_rank * 8 + self.piece.from_file);
        let new_position_mask = 1u64 << (self.to.0 * 8 + self.to.1);
        
        board[self.piece.piece_type] &= !curr_position_mask;
        
        for i in self.piece.piece_type.enemy_indices() {
            board.bitboards[i] &= !new_position_mask; 
        }

        board[self.piece.piece_type] |= new_position_mask;
    }

    pub fn check_and_execute(&self, board: &mut Board) -> bool {
        if self.check_legality(board) {
            self.execute_move(board);
            true
        } else {
            false
        }
    }
}