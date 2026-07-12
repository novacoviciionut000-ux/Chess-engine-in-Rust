pub mod board;
pub mod moves;
use board::Board;
fn main() {
    let board: Board = Board::new();
    board.print_board();
}