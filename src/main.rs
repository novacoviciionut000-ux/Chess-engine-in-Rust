pub mod board;
pub mod moves;
pub mod ui;
pub mod dispatch_to_python;
pub mod evaluate;

use board::Board;
use evaluate::Evaluator;
use moves::{generate_legal_moves, Move};
use std::fs;
use std::thread;
use std::time::Duration;

fn main() {
    let mut board: Board = Board::new();
    let mut is_white_turn = true; // True = Human (White), False = Engine (Black)

    // Write the initial starting position so the Python board renders immediately
    if let Err(e) = crate::dispatch_to_python::write_board_state(&board, is_white_turn, "board_state.fen") {
        eprintln!("Error writing initial board state: {}", e);
    }

    // Ensure we start with a clean move file
    let _ = fs::write("player_move.txt", "");

    loop {
        board.print_board();

        if is_white_turn {
            println!("Waiting for human move (White)...");
            
            // Poll the file for a move from the Python UI
            loop {
                if let Ok(contents) = fs::read_to_string("player_move.txt") {
                    let move_str = contents.trim();
                    
                    if move_str.len() >= 4 {
                        if let Some(mv) = parse_gui_move(move_str, &board, is_white_turn) {
                            println!("Human plays: {}", move_str);
                            board.make_move(&mv);
                            
                            // Clear the file so we don't read it again next turn
                            let _ = fs::write("player_move.txt", "");
                            is_white_turn = false;
                            break; 
                        } else {
                            // The UI sent an illegal move. Clear it and wait for a valid one.
                            println!("Illegal move attempted: {}", move_str);
                            let _ = fs::write("player_move.txt", "");
                        }
                    }
                }
                // Sleep briefly to prevent the loop from maxing out 100% of a CPU core
                thread::sleep(Duration::from_millis(50));
            }
            
        } else {
            println!("Engine is thinking (Depth 6)...");
            let evaluator = Evaluator::new(6);
            
            if let Some(best_move) = evaluator.best_move(&mut board, is_white_turn) {
                let best_move_str = format!(
                    "{}{}{}{}",
                    (b'a' + best_move.piece.from_file as u8) as char,
                    best_move.piece.from_rank + 1,
                    (b'a' + best_move.to.1 as u8) as char,
                    best_move.to.0 + 1
                );
                
                println!("Engine plays: {}", best_move_str);
                board.make_move(&best_move);
                is_white_turn = true;
            } else {
                println!("Engine has no legal moves. Game over!");
                break;
            }
        }

        // Write the new board state to the FEN file so the Python UI updates
        if let Err(e) = crate::dispatch_to_python::write_board_state(&board, is_white_turn, "board_state.fen") {
            eprintln!("Error writing board state: {}", e);
        }
    }
}

/// Helper function to convert a string like "e2e4" into a valid Move struct.
/// It works by generating all legal moves and finding the one that matches the start/end squares.
fn parse_gui_move(move_str: &str, board: &Board, is_white: bool) -> Option<Move> {
    let chars: Vec<char> = move_str.chars().collect();
    if chars.len() < 4 {
        return None;
    }

    let from_file = (chars[0] as u8).saturating_sub(b'a') as usize;
    let from_rank = (chars[1] as u8).saturating_sub(b'1') as usize;
    let to_file = (chars[2] as u8).saturating_sub(b'a') as usize;
    let to_rank = (chars[3] as u8).saturating_sub(b'1') as usize;

    // Optional: detect if the string includes a promotion (e.g. "e7e8q")
    let promotion_char = chars.get(4).copied();

    let legal_moves = generate_legal_moves(board, is_white);
    
    for mv in legal_moves {
        if mv.piece.from_file == from_file 
            && mv.piece.from_rank == from_rank 
            && mv.to.1 == to_file 
            && mv.to.0 == to_rank 
        {
            // If the UI sends a promotion character, ensure it matches
            if let Some(p_char) = promotion_char {
                // You may need to adapt this part depending on how your Move struct handles promotions
                // Right now it just returns the first matching move coordinate-wise.
                return Some(mv); 
            }
            return Some(mv);
        }
    }
    
    None
}