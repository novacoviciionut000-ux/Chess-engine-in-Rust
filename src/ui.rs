use board::Piece;

use std::io::{self, Write};
use crate::board::Board;
use crate::moves::Move;
use crate::moves::ActivePiece;

pub struct MoveListener {
    pub original_square: (usize, usize),
    pub new_square: (usize, usize),
    pub current_player: bool, // true for white, false for black
}

// 1. A robust helper to translate strings like "e2" into (Rank, File) tuples
fn parse_square(square: &str) -> Option<(usize, usize)> {
    let chars: Vec<char> = square.chars().collect();
    if chars.len() != 2 { return None; }

    let file_char = chars[0].to_ascii_lowercase();
    let rank_char = chars[1];

    if !('a'..='h').contains(&file_char) || !('1'..='8').contains(&rank_char) {
        return None;
    }

    let file = (file_char as u8 - b'a') as usize;
    let rank = (rank_char as u8 - b'1') as usize;

    Some((rank, file))
}

// 2. A safe input reader that loops until you provide a valid square
fn read_square(prompt: &str) -> (usize, usize) {
    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if let Some(coords) = parse_square(input) {
            return coords;
        } else {
            println!("Invalid square! Please use algebraic notation (e.g., e2).");
        }
    }
}

// 3. Prompt for a promotion piece when a pawn reaches the back rank.
fn read_promotion_choice(is_white: bool) -> Piece {
    loop {
        print!("Promote to (q/r/b/n): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let choice = input.trim().to_ascii_lowercase();

        let piece = match choice.as_str() {
            "q" => Some(if is_white { Piece::WhiteQueen } else { Piece::BlackQueen }),
            "r" => Some(if is_white { Piece::WhiteRook } else { Piece::BlackRook }),
            "b" => Some(if is_white { Piece::WhiteBishop } else { Piece::BlackBishop }),
            "n" => Some(if is_white { Piece::WhiteKnight } else { Piece::BlackKnight }),
            _ => None,
        };

        if let Some(p) = piece {
            return p;
        }
        println!("Invalid choice! Enter q, r, b, or n.");
    }
}

impl MoveListener {
    pub fn new() -> Self {
        MoveListener {
            original_square: (0, 0),
            new_square: (0, 0),
            current_player: true,
        }
    }

    pub fn get_current_move(&mut self) {
        self.original_square = read_square("Enter original square (e.g., e2): ");
        self.new_square = read_square("Enter new square (e.g., e4): ");
    }

    pub fn convert_to_move(&self, board: &Board) -> Option<Move> {
        let piece_type = match board.get_piece_at((self.original_square.0, self.original_square.1), self.current_player) {
            Some(piece) => Piece::from_usize(piece).unwrap(),
            None => return None,
        };

        let active_piece = ActivePiece {
            piece_type,
            from_rank: self.original_square.0,
            from_file: self.original_square.1,
        };

        // A promotion piece is required exactly when a pawn reaches the back rank.
        let is_promotion = match piece_type {
            Piece::WhitePawn => self.new_square.0 == 7,
            Piece::BlackPawn => self.new_square.0 == 0,
            _ => false,
        };

        let promotion = if is_promotion {
            Some(read_promotion_choice(self.current_player))
        } else {
            None
        };

        Some(Move {
            piece: active_piece,
            to: self.new_square,
            promotion,
        })
    }

    pub fn get_move_convert_and_execute(&mut self, board: &mut Board) {
        self.get_current_move();

        let current_move = self.convert_to_move(board);

        match current_move {
            Some(mv) => {
                if mv.piece.piece_type.is_white() != self.current_player {
                    println!("It is not your turn!\n");
                    return;
                }


                if mv.is_legal(board) {
                    mv.execute_move(board);
                    println!("Move executed.\n");
                    self.current_player = !self.current_player;
                } else {
                    println!("Illegal move!\n");
                }
            },
            None => {
                println!("No piece at the original square.\n");
            }
        }
        if crate::moves::is_checkmate(board, self.current_player) {
            println!("Checkmate! {} wins!", if self.current_player { "Black" } else { "White" });
            std::process::exit(0);
        } else if crate::moves::is_stalemate(board, self.current_player) {
            println!("Stalemate! It's a draw.");
            std::process::exit(0);
        }
    }
}