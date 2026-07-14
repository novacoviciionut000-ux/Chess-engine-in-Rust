use crate::board::Board;
use crate::board::Piece;
use std::fs;
use std::io;

fn piece_char(piece: Piece) -> char {
    match piece {
        Piece::WhitePawn => 'P',
        Piece::WhiteKnight => 'N',
        Piece::WhiteBishop => 'B',
        Piece::WhiteRook => 'R',
        Piece::WhiteQueen => 'Q',
        Piece::WhiteKing => 'K',
        Piece::BlackPawn => 'p',
        Piece::BlackKnight => 'n',
        Piece::BlackBishop => 'b',
        Piece::BlackRook => 'r',
        Piece::BlackQueen => 'q',
        Piece::BlackKing => 'k',
    }
}

fn square_to_algebraic(rank: usize, file: usize) -> String {
    let file_char = (b'a' + file as u8) as char;
    let rank_char = (b'1' + rank as u8) as char;
    format!("{}{}", file_char, rank_char)
}

/// Converts the board into a standard FEN string. Halfmove clock and
/// fullmove number aren't tracked yet, so they're placeholders (0 / 1) --
/// fine for a viewer, but fill them in properly if you ever want to export
/// real game files (PGN, etc).
pub fn board_to_fen(board: &Board, white_to_move: bool) -> String {
    let mut fields: Vec<String> = Vec::with_capacity(6);

    // 1. Piece placement: FEN goes rank 8 down to rank 1, so we walk our
    // rank index from 7 down to 0 (our rank 0 is white's back rank / FEN's rank 1).
    let white_pieces = Piece::all_of_color(true);
    let black_pieces = Piece::all_of_color(false);

    let mut placement = String::new();
    for rank in (0..8).rev() {
        let mut empty_run = 0;
        for file in 0..8 {
            let sq_mask = 1u64 << (rank * 8 + file);
            let mut occupant: Option<Piece> = None;
            // .iter() + `&piece` works whether all_of_color returns an owned
            // [Piece; 6] or a reference to one -- either way this gives us
            // back an owned, Copy'd Piece to index the board with.
            for &piece in white_pieces.iter().chain(black_pieces.iter()) {
                if board[piece] & sq_mask != 0 {
                    occupant = Some(piece);
                    break;
                }
            }

            match occupant {
                Some(p) => {
                    if empty_run > 0 {
                        placement.push_str(&empty_run.to_string());
                        empty_run = 0;
                    }
                    placement.push(piece_char(p));
                }
                None => empty_run += 1,
            }
        }
        if empty_run > 0 {
            placement.push_str(&empty_run.to_string());
        }
        if rank != 0 {
            placement.push('/');
        }
    }
    fields.push(placement);

    // 2. Active color
    fields.push(if white_to_move { "w".to_string() } else { "b".to_string() });

    // 3. Castling availability
    let mut castling = String::new();
    let rights = &board.castling_rights;
    if rights.white_kingside {
        castling.push('K');
    }
    if rights.white_queenside {
        castling.push('Q');
    }
    if rights.black_kingside {
        castling.push('k');
    }
    if rights.black_queenside {
        castling.push('q');
    }
    if castling.is_empty() {
        castling.push('-');
    }
    fields.push(castling);

    // 4. En passant target square
    fields.push(match board.en_passant_target {
        Some((rank, file)) => square_to_algebraic(rank, file),
        None => "-".to_string(),
    });

    // 5. Halfmove clock, 6. Fullmove number -- placeholders for now.
    fields.push("0".to_string());
    fields.push("1".to_string());

    fields.join(" ")
}

/// Writes the current position to `path` as a FEN string. Call this once
/// after every move executes; the Python viewer polls this file and
/// redraws whenever its contents change.
pub fn write_board_state(board: &Board, white_to_move: bool, path: &str) -> io::Result<()> {
    let fen = board_to_fen(board, white_to_move);
    fs::write(path, fen)
}