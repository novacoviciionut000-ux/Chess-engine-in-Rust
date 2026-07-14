use crate::board::Board;
use crate::board::Piece;
use crate::moves::{generate_legal_moves, is_checkmate, is_stalemate, Move};
use crate::board::UndoInfo;
const WHITE_PAWN_VALUE: i64 = 1;
const WHITE_KNIGHT_VALUE: i64 = 3;
const WHITE_BISHOP_VALUE: i64 = 3;
const WHITE_ROOK_VALUE: i64 = 5;
const WHITE_QUEEN_VALUE: i64 = 9;
const WHITE_KING_VALUE: i64 = 20000;
const BLACK_PAWN_VALUE: i64 = 1;
const BLACK_KNIGHT_VALUE: i64 = 3;
const BLACK_BISHOP_VALUE: i64 = 3;
const BLACK_ROOK_VALUE: i64 = 5;
const BLACK_QUEEN_VALUE: i64 = 9;
const BLACK_KING_VALUE: i64 = 20000;

const PIECE_VALUES: [i64; 12] = [
    WHITE_PAWN_VALUE, WHITE_KNIGHT_VALUE, WHITE_BISHOP_VALUE, WHITE_ROOK_VALUE,
    WHITE_QUEEN_VALUE, WHITE_KING_VALUE,
    BLACK_PAWN_VALUE, BLACK_KNIGHT_VALUE, BLACK_BISHOP_VALUE, BLACK_ROOK_VALUE,
    BLACK_QUEEN_VALUE, BLACK_KING_VALUE,
];

#[rustfmt::skip]
const PAWN_TABLE: [i64; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10,-15,-15, 10, 10,  5,
     5, -5,-10,  5,  5,-10, -5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5,  5, 10, 25, 25, 10,  5,  5,
    10, 15, 20, 30, 30, 20, 15, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT_TABLE: [i64; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 20, 25, 25, 20,  0,-30,
    -30,  5, 20, 25, 25, 20,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i64; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 15, 15, 10,  0,-10,
    -10,  5,  5, 15, 15,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_TABLE: [i64; 64] = [
      0,  0,  5, 10, 10,  5,  0,  0,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
     -5,  0,  0,  0,  0,  0,  0, -5,
      5, 10, 10, 10, 10, 10, 10,  5,
      0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_TABLE: [i64; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
      0,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const KING_TABLE: [i64; 64] = [
     20, 30, 10,  0,  0, 10, 30, 20,
     20, 20,  0,  0,  0,  0, 20, 20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
];

fn piece_square_table(piece_type: Piece) -> &'static [i64; 64] {
    match piece_type {
        Piece::WhitePawn | Piece::BlackPawn => &PAWN_TABLE,
        Piece::WhiteKnight | Piece::BlackKnight => &KNIGHT_TABLE,
        Piece::WhiteBishop | Piece::BlackBishop => &BISHOP_TABLE,
        Piece::WhiteRook | Piece::BlackRook => &ROOK_TABLE,
        Piece::WhiteQueen | Piece::BlackQueen => &QUEEN_TABLE,
        Piece::WhiteKing | Piece::BlackKing => &KING_TABLE,
    }
}

fn mirror_square(square: usize) -> usize {
    square ^ 56
}

fn piece_at(board: &Board, square: (usize, usize)) -> Option<Piece> {
    let mask = 1u64 << (square.0 * 8 + square.1);
    Piece::all_of_color(true)
        .iter()
        .chain(Piece::all_of_color(false).iter())
        .copied()
        .find(|&p| board[p] & mask != 0)
}

pub struct Evaluator {
    pub max_depth: u32,
}

impl Evaluator {
    pub fn new(max_depth: u32) -> Self {
        Evaluator { max_depth }
    }

    pub fn evaluate_position(&self, board: &mut Board, is_white: bool) -> i64 {
        let alpha = i64::MIN;
        let beta = i64::MAX;
        self.minimax(board, self.max_depth, is_white, alpha, beta)
    }

    pub fn best_move(&self, board: &mut Board, is_white: bool) -> Option<Move> {
        let mut moves = generate_legal_moves(board, is_white);
        moves.sort_by_key(|mv| std::cmp::Reverse(self.move_score(mv, board)));

        let mut best_score = if is_white { i64::MIN } else { i64::MAX };
        let mut best: Option<Move> = None;
        let mut alpha = i64::MIN;
        let mut beta = i64::MAX;

        for mv in moves {
            let undo = board.make_move(&mv);
            let score = self.minimax(board, self.max_depth.saturating_sub(1), !is_white, alpha, beta);
            board.unmake_move(&mv, undo);

            let better = if is_white { score > best_score } else { score < best_score };
            if better {
                best_score = score;
                best = Some(mv);
            }

            if is_white {
                alpha = alpha.max(best_score);
            } else {
                beta = beta.min(best_score);
            }
        }

        best
    }

    fn move_score(&self, mv: &Move, board: &Board) -> i64 {
        let mut score = 0;

        if let Some(promo) = mv.promotion {
            score += PIECE_VALUES[promo as usize] * 100;
        }

        if let Some(victim) = piece_at(board, mv.to) {
            let attacker_value = PIECE_VALUES[mv.piece.piece_type as usize];
            let victim_value = PIECE_VALUES[victim as usize];
            score += victim_value * 100 - attacker_value;
        } else if matches!(mv.piece.piece_type, Piece::WhitePawn | Piece::BlackPawn)
            && mv.piece.from_file != mv.to.1
        {
            let pawn_value = PIECE_VALUES[Piece::WhitePawn as usize];
            score += pawn_value * 100 - pawn_value;
        }

        score
    }

    fn minimax(&self, board: &mut Board, depth: u32, is_white: bool, mut alpha: i64, mut beta: i64) -> i64 {
        if depth == 0 {
            return self.static_evaluation(board);
        }

        let mut moves = generate_legal_moves(board, is_white);
        moves.sort_by_key(|mv| std::cmp::Reverse(self.move_score(mv, board)));

        if moves.is_empty() {
            if is_checkmate(board, is_white) {
                return if is_white {
                    i64::MIN + 1000 + depth as i64
                } else {
                    i64::MAX - 1000 - depth as i64
                };
            }
            if is_stalemate(board, is_white) {
                return 0;
            }
        }

        if is_white {
            let mut best = i64::MIN;
            for mv in moves {
                let undo_move: UndoInfo = board.make_move(&mv);
                let score = self.minimax(board, depth - 1, false, alpha, beta);
                board.unmake_move(&mv, undo_move);
                best = best.max(score);
                alpha = alpha.max(best);

                if beta <= alpha {
                    break;
                }
            }
            best
        } else {
            let mut best = i64::MAX;
            for mv in moves {
                let undo_move: UndoInfo = board.make_move(&mv);
                let score = self.minimax(board, depth-1, true, alpha, beta);
                board.unmake_move(&mv, undo_move);
                best = best.min(score);
                beta = beta.min(best);

                if beta <= alpha {
                    break;
                }
            }
            best
        }
    }

    pub fn static_evaluation(&self, board: &Board) -> i64 {
        let material = self.material_count(board, true) - self.material_count(board, false);
        let positional = self.positional_score(board, true) - self.positional_score(board, false);
        material * 100 + positional
    }

    pub fn material_count(&self, board: &Board, is_white: bool) -> i64 {
        let mut total = 0;
        for piece in Piece::all_of_color(is_white) {
            let piece_value = PIECE_VALUES[piece as usize];
            let piece_count = board[piece].count_ones() as i64;
            total += piece_value * piece_count;
        }
        total
    }

    pub fn positional_score(&self, board: &Board, is_white: bool) -> i64 {
        let mut total = 0;
        for piece in Piece::all_of_color(is_white) {
            let table = piece_square_table(piece);
            let mut bb: u64 = board[piece];
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                bb &= bb - 1;

                let index = if is_white { sq } else { mirror_square(sq) };
                total += table[index];
            }
        }
        total
    }
}