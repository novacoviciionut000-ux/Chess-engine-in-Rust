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
    /// Only used for pawn moves landing on the back rank. Must be Some(Q/R/B/N)
    /// if and only if the move is actually a promotion; see `promotion_is_valid`.
    pub promotion: Option<Piece>,
}

impl Move {
    pub fn new(piece: ActivePiece, to: (usize, usize)) -> Self {
        Move { piece, to, promotion: None }
    }

    /// Construct a pawn promotion move. `promote_to` should be a queen, rook,
    /// bishop, or knight of the correct color.
    pub fn new_promotion(piece: ActivePiece, to: (usize, usize), promote_to: Piece) -> Self {
        Move { piece, to, promotion: Some(promote_to) }
    }

    /// Castling is just "move the king two squares toward the rook" --
    /// there's no separate constructor, `check_legality` recognizes it
    /// from the king's start/end squares.

    pub fn check_legality(&self, board: &Board) -> bool {
        if self.to.0 > 7 || self.to.1 > 7 {
            return false;
        }

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
            Piece::WhitePawn => self.white_pawn_legal(board) && self.promotion_is_valid(7),
            Piece::BlackPawn => self.black_pawn_legal(board) && self.promotion_is_valid(0),
            Piece::WhiteKnight | Piece::BlackKnight => {
                (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
            }
            Piece::WhiteBishop | Piece::BlackBishop => {
                rank_diff == file_diff && rank_diff != 0 && self.path_is_clear(board)
            }
            Piece::WhiteRook | Piece::BlackRook => {
                (self.piece.from_rank == self.to.0) != (self.piece.from_file == self.to.1)
                    && self.path_is_clear(board)
            }
            Piece::WhiteQueen | Piece::BlackQueen => {
                let straight = self.piece.from_rank == self.to.0 || self.piece.from_file == self.to.1;
                let diagonal = rank_diff == file_diff;
                (straight || diagonal) && (rank_diff != 0 || file_diff != 0) && self.path_is_clear(board)
            }
            Piece::WhiteKing | Piece::BlackKing => {
                if file_diff == 2 && rank_diff == 0 {
                    self.castle_is_legal(board)
                } else {
                    rank_diff <= 1 && file_diff <= 1 && (rank_diff != 0 || file_diff != 0)
                }
            }
        }
    }

    /// A promotion piece must be given exactly when the pawn reaches `back_rank`,
    /// and it must be a same-colored queen/rook/bishop/knight.
    fn promotion_is_valid(&self, back_rank: usize) -> bool {
        let reaches_back_rank = self.to.0 == back_rank;
        match self.promotion {
            Some(p) => {
                reaches_back_rank
                    && p.is_white() == self.piece.piece_type.is_white()
                    && matches!(
                        p,
                        Piece::WhiteQueen
                            | Piece::WhiteRook
                            | Piece::WhiteBishop
                            | Piece::WhiteKnight
                            | Piece::BlackQueen
                            | Piece::BlackRook
                            | Piece::BlackBishop
                            | Piece::BlackKnight
                    )
            }
            None => !reaches_back_rank,
        }
    }

    fn path_is_clear(&self, board: &Board) -> bool {
        let dr = (self.to.0 as isize - self.piece.from_rank as isize).signum();
        let df = (self.to.1 as isize - self.piece.from_file as isize).signum();

        let occupied = board.get_white_occupancy_mask() | board.get_black_occupancy_mask();

        let mut r = self.piece.from_rank as isize + dr;
        let mut f = self.piece.from_file as isize + df;

        while (r, f) != (self.to.0 as isize, self.to.1 as isize) {
            let sq_mask: u64 = 1 << (r * 8 + f);
            if occupied & sq_mask != 0 {
                return false;
            }
            r += dr;
            f += df;
        }
        true
    }

    fn white_pawn_legal(&self, board: &Board) -> bool {
        let from_rank = self.piece.from_rank as isize;
        let from_file = self.piece.from_file as isize;

        let mask: u64 = 1 << (self.piece.from_rank * 8 + self.piece.from_file);
        let is_first_move = (mask & board[Piece::WhitePawn]) != 0 && self.piece.from_rank == 1;

        let rank_difference = self.to.0 as isize - from_rank;
        let file_difference = self.to.1 as isize - from_file;

        let occupied = board.get_white_occupancy_mask() | board.get_black_occupancy_mask();
        let dest_mask: u64 = 1 << (self.to.0 * 8 + self.to.1);
        let dest_empty = occupied & dest_mask == 0;

        if file_difference == 0 {
            if rank_difference == 1 {
                return dest_empty;
            }
            if rank_difference == 2 && is_first_move {
                let mid_mask: u64 = 1 << ((self.piece.from_rank + 1) * 8 + self.piece.from_file);
                return dest_empty && occupied & mid_mask == 0;
            }
            return false;
        }

        if rank_difference == 1 && file_difference.abs() == 1 {
            let black_occ = board.get_black_occupancy_mask();
            if black_occ & dest_mask != 0 {
                return true; // ordinary diagonal capture
            }
            // en passant: destination is empty, but it matches the target
            // square set by the opponent's last (two-square) pawn move
            return board.en_passant_target == Some(self.to);
        }

        false
    }

    fn black_pawn_legal(&self, board: &Board) -> bool {
        let from_rank = self.piece.from_rank as isize;
        let from_file = self.piece.from_file as isize;

        let mask: u64 = 1 << (self.piece.from_rank * 8 + self.piece.from_file);
        let is_first_move = (mask & board[Piece::BlackPawn]) != 0 && self.piece.from_rank == 6;

        let rank_difference = self.to.0 as isize - from_rank;
        let file_difference = self.to.1 as isize - from_file;

        let occupied = board.get_white_occupancy_mask() | board.get_black_occupancy_mask();
        let dest_mask: u64 = 1 << (self.to.0 * 8 + self.to.1);
        let dest_empty = occupied & dest_mask == 0;

        if file_difference == 0 {
            if rank_difference == -1 {
                return dest_empty;
            }
            if rank_difference == -2 && is_first_move {
                let mid_mask: u64 = 1 << ((self.piece.from_rank - 1) * 8 + self.piece.from_file);
                return dest_empty && occupied & mid_mask == 0;
            }
            return false;
        }

        if rank_difference == -1 && file_difference.abs() == 1 {
            let white_occ = board.get_white_occupancy_mask();
            if white_occ & dest_mask != 0 {
                return true;
            }
            return board.en_passant_target == Some(self.to);
        }

        false
    }

    /// Castling legality: correct starting squares, rights not yet revoked,
    /// rook present, squares between empty, and the king is not in check,
    /// does not pass through check, and does not land in check.
    fn castle_is_legal(&self, board: &Board) -> bool {
        let is_white = self.piece.piece_type.is_white();
        let rank = if is_white { 0 } else { 7 };

        if self.piece.from_rank != rank || self.piece.from_file != 4 {
            return false;
        }

        let kingside = self.to.1 == 6;
        let queenside = self.to.1 == 2;
        if !kingside && !queenside {
            return false;
        }

        let rights = &board.castling_rights;
        let allowed = match (is_white, kingside) {
            (true, true) => rights.white_kingside,
            (true, false) => rights.white_queenside,
            (false, true) => rights.black_kingside,
            (false, false) => rights.black_queenside,
        };
        if !allowed {
            return false;
        }

        let rook_file = if kingside { 7 } else { 0 };
        let rook_piece = if is_white { Piece::WhiteRook } else { Piece::BlackRook };
        if board[rook_piece] & (1u64 << (rank * 8 + rook_file)) == 0 {
            return false; // rook has moved or been captured
        }

        let empty_files: &[usize] = if kingside { &[5, 6] } else { &[1, 2, 3] };
        let occupied = board.get_white_occupancy_mask() | board.get_black_occupancy_mask();
        for &f in empty_files {
            if occupied & (1u64 << (rank * 8 + f)) != 0 {
                return false;
            }
        }

        // King can't start in, pass through, or land on an attacked square.
        let step: isize = if kingside { 1 } else { -1 };
        let start_file = self.piece.from_file as isize;
        for i in 0..=2 {
            let f = start_file + step * i;
            if !(0..=7).contains(&f) {
                return false;
            }
            if Self::square_attacked(board, (rank, f as usize), !is_white) {
                return false;
            }
        }

        true
    }

    pub fn execute_move(&self, board: &mut Board) {
        let curr_position_mask = 1u64 << (self.piece.from_rank * 8 + self.piece.from_file);
        let new_position_mask = 1u64 << (self.to.0 * 8 + self.to.1);
        let is_white = self.piece.piece_type.is_white();

        board[self.piece.piece_type] &= !curr_position_mask;

        // En passant: the captured pawn sits behind the destination square,
        // not on it -- detect this as "diagonal pawn move onto an empty square".
        let is_en_passant = matches!(self.piece.piece_type, Piece::WhitePawn | Piece::BlackPawn)
            && self.piece.from_file != self.to.1
            && (board.get_white_occupancy_mask() | board.get_black_occupancy_mask()) & new_position_mask == 0;

        if is_en_passant {
            let captured_rank = if is_white { self.to.0 - 1 } else { self.to.0 + 1 };
            let captured_mask = 1u64 << (captured_rank * 8 + self.to.1);
            let captured_piece = if is_white { Piece::BlackPawn } else { Piece::WhitePawn };
            board[captured_piece] &= !captured_mask;
        } else {
            for i in self.piece.piece_type.enemy_indices() {
                board.bitboards[i] &= !new_position_mask;
            }
        }

        // Promotion: the piece that lands is the chosen piece, not the pawn.
        let landing_piece = self.promotion.unwrap_or(self.piece.piece_type);
        board[landing_piece] |= new_position_mask;

        // Castling: also relocate the rook to the other side of the king.
        if matches!(self.piece.piece_type, Piece::WhiteKing | Piece::BlackKing)
            && (self.piece.from_file as isize - self.to.1 as isize).abs() == 2
        {
            let rank = self.piece.from_rank;
            let kingside = self.to.1 == 6;
            let rook_piece = if is_white { Piece::WhiteRook } else { Piece::BlackRook };
            let (rook_from, rook_to) = if kingside { (7, 5) } else { (0, 3) };
            board[rook_piece] &= !(1u64 << (rank * 8 + rook_from));
            board[rook_piece] |= 1u64 << (rank * 8 + rook_to);
        }

        // Set (or clear) the en passant target for the *next* move.
        board.en_passant_target = if matches!(self.piece.piece_type, Piece::WhitePawn | Piece::BlackPawn)
            && (self.to.0 as isize - self.piece.from_rank as isize).abs() == 2
        {
            Some(((self.to.0 + self.piece.from_rank) / 2, self.to.1))
        } else {
            None
        };

        Self::update_castling_rights(board, self);
    }

    /// Revoke castling rights when a king moves, or when a rook moves away
    /// from or is captured on its home corner.
    fn update_castling_rights(board: &mut Board, mv: &Move) {
        if matches!(mv.piece.piece_type, Piece::WhiteKing | Piece::BlackKing) {
            if mv.piece.piece_type.is_white() {
                board.castling_rights.white_kingside = false;
                board.castling_rights.white_queenside = false;
            } else {
                board.castling_rights.black_kingside = false;
                board.castling_rights.black_queenside = false;
            }
        }

        for square in [(mv.piece.from_rank, mv.piece.from_file), mv.to] {
            match square {
                (0, 7) => board.castling_rights.white_kingside = false,
                (0, 0) => board.castling_rights.white_queenside = false,
                (7, 7) => board.castling_rights.black_kingside = false,
                (7, 0) => board.castling_rights.black_queenside = false,
                _ => {}
            }
        }
    }

    /// Full legality: geometry/occupancy/path-blocking/castling-rules, *and*
    /// the move must not leave the mover's own king in check.
    pub fn is_legal(&self, board: &Board) -> bool {
        if !self.check_legality(board) {
            return false;
        }

        let mut hypothetical = board.clone();
        self.execute_move(&mut hypothetical);

        !Self::king_in_check(&hypothetical, self.piece.piece_type.is_white())
    }

    /// True if `square` is attacked by any piece of the given color.
    pub fn square_attacked(board: &Board, square: (usize, usize), by_white: bool) -> bool {
        for piece in Piece::all_of_color(by_white) {
            let mut bb: u64 = board[piece];
            while bb != 0 {
                let sq = bb.trailing_zeros() as usize;
                bb &= bb - 1;

                let attacker = ActivePiece {
                    piece_type: piece,
                    from_rank: sq / 8,
                    from_file: sq % 8,
                };
                let attacking_move = Move::new(attacker, square);

                if attacking_move.check_legality(board) {
                    return true;
                }
            }
        }
        false
    }

    pub fn king_in_check(board: &Board, is_white: bool) -> bool {
        let king_piece = if is_white { Piece::WhiteKing } else { Piece::BlackKing };
        let king_bb = board[king_piece];
        if king_bb == 0 {
            return false;
        }
        let king_square = king_bb.trailing_zeros() as usize;
        Self::square_attacked(board, (king_square / 8, king_square % 8), !is_white)
    }

    pub fn check_and_execute(&self, board: &mut Board) -> bool {
        if self.is_legal(board) {
            self.execute_move(board);
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Move generation
//
// The functions below produce *pseudo-legal* candidate destinations directly
// from each piece's movement pattern (knight offsets, sliding rays stopped at
// the first blocker, pawn's small fixed set of destinations) rather than
// trying all 64 squares and throwing most of them away. Each candidate is
// still run through `is_legal`, which is the only step that has to clone the
// board (to check whether the move leaves the mover's own king in check) --
// that part doesn't get cheaper without incremental check tracking, which is
// the natural next step once this becomes an engine.
// ---------------------------------------------------------------------------

const KNIGHT_OFFSETS: [(isize, isize); 8] = [
    (1, 2), (1, -2), (-1, 2), (-1, -2),
    (2, 1), (2, -1), (-2, 1), (-2, -1),
];

const KING_OFFSETS: [(isize, isize); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), (0, 1),
    (1, -1), (1, 0), (1, 1),
];

const BISHOP_DIRECTIONS: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_DIRECTIONS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// All pseudo-legal destination squares for one piece, ignoring whether the
/// resulting position leaves the mover's own king in check (that's `is_legal`'s
/// job, applied afterward).
fn pseudo_legal_destinations(
    board: &Board,
    piece_type: Piece,
    from_rank: usize,
    from_file: usize,
) -> Vec<(usize, usize)> {
    let is_white = piece_type.is_white();
    let friendly = if is_white {
        board.get_white_occupancy_mask()
    } else {
        board.get_black_occupancy_mask()
    };
    let enemy = if is_white {
        board.get_black_occupancy_mask()
    } else {
        board.get_white_occupancy_mask()
    };
    let occupied = friendly | enemy;

    match piece_type {
        Piece::WhiteKnight | Piece::BlackKnight => {
            offset_destinations(&KNIGHT_OFFSETS, from_rank, from_file, friendly)
        }
        Piece::WhiteKing | Piece::BlackKing => {
            let mut dests = offset_destinations(&KING_OFFSETS, from_rank, from_file, friendly);
            // Candidate castling squares -- castle_is_legal (called via
            // is_legal) does the real validation; these are cheap to include
            // and get filtered out immediately if invalid.
            if from_rank == self_home_rank(is_white) && from_file == 4 {
                dests.push((from_rank, 6));
                dests.push((from_rank, 2));
            }
            dests
        }
        Piece::WhiteBishop | Piece::BlackBishop => {
            ray_destinations(&BISHOP_DIRECTIONS, from_rank, from_file, friendly, occupied)
        }
        Piece::WhiteRook | Piece::BlackRook => {
            ray_destinations(&ROOK_DIRECTIONS, from_rank, from_file, friendly, occupied)
        }
        Piece::WhiteQueen | Piece::BlackQueen => {
            let mut dests = ray_destinations(&BISHOP_DIRECTIONS, from_rank, from_file, friendly, occupied);
            dests.extend(ray_destinations(&ROOK_DIRECTIONS, from_rank, from_file, friendly, occupied));
            dests
        }
        Piece::WhitePawn => pawn_destinations(board, from_rank, from_file, true, occupied),
        Piece::BlackPawn => pawn_destinations(board, from_rank, from_file, false, occupied),
    }
}

fn self_home_rank(is_white: bool) -> usize {
    if is_white { 0 } else { 7 }
}

/// Fixed-offset pieces (knight, king): try each offset once, keep it if it's
/// on the board and not occupied by a friendly piece.
fn offset_destinations(
    offsets: &[(isize, isize)],
    from_rank: usize,
    from_file: usize,
    friendly: u64,
) -> Vec<(usize, usize)> {
    let mut dests = Vec::with_capacity(offsets.len());
    for &(dr, df) in offsets {
        let r = from_rank as isize + dr;
        let f = from_file as isize + df;
        if !(0..=7).contains(&r) || !(0..=7).contains(&f) {
            continue;
        }
        let (r, f) = (r as usize, f as usize);
        let mask = 1u64 << (r * 8 + f);
        if friendly & mask == 0 {
            dests.push((r, f));
        }
    }
    dests
}

/// Sliding pieces (bishop, rook, queen): walk each direction one square at a
/// time, stopping at the edge of the board, a friendly piece (excluded), or
/// an enemy piece (included, since that's a capture, then stop).
fn ray_destinations(
    directions: &[(isize, isize)],
    from_rank: usize,
    from_file: usize,
    friendly: u64,
    occupied: u64,
) -> Vec<(usize, usize)> {
    let mut dests = Vec::with_capacity(directions.len() * 3);
    for &(dr, df) in directions {
        let mut r = from_rank as isize + dr;
        let mut f = from_file as isize + df;
        while (0..=7).contains(&r) && (0..=7).contains(&f) {
            let (ru, fu) = (r as usize, f as usize);
            let mask = 1u64 << (ru * 8 + fu);

            if friendly & mask != 0 {
                break; // blocked by our own piece, and can't land on it
            }
            dests.push((ru, fu));
            if occupied & mask != 0 {
                break; // captured an enemy piece here, ray stops
            }
            r += dr;
            f += df;
        }
    }
    dests
}

/// Pawns have a small fixed set of possible destinations: one step forward,
/// two steps forward from the start rank, and two diagonal captures
/// (including en passant).
fn pawn_destinations(
    board: &Board,
    from_rank: usize,
    from_file: usize,
    is_white: bool,
    occupied: u64,
) -> Vec<(usize, usize)> {
    let mut dests = Vec::with_capacity(4);
    let dir: isize = if is_white { 1 } else { -1 };
    let start_rank = if is_white { 1 } else { 6 };

    let one_step = from_rank as isize + dir;
    if (0..=7).contains(&one_step) {
        let one_step = one_step as usize;
        if occupied & (1u64 << (one_step * 8 + from_file)) == 0 {
            dests.push((one_step, from_file));

            if from_rank == start_rank {
                let two_step = from_rank as isize + 2 * dir;
                if (0..=7).contains(&two_step) {
                    let two_step = two_step as usize;
                    if occupied & (1u64 << (two_step * 8 + from_file)) == 0 {
                        dests.push((two_step, from_file));
                    }
                }
            }
        }
    }

    let enemy = if is_white {
        board.get_black_occupancy_mask()
    } else {
        board.get_white_occupancy_mask()
    };

    for df in [-1isize, 1isize] {
        let r = from_rank as isize + dir;
        let f = from_file as isize + df;
        if !(0..=7).contains(&r) || !(0..=7).contains(&f) {
            continue;
        }
        let (r, f) = (r as usize, f as usize);
        let mask = 1u64 << (r * 8 + f);
        if enemy & mask != 0 || board.en_passant_target == Some((r, f)) {
            dests.push((r, f));
        }
    }

    dests
}

fn promotion_choices(is_white: bool) -> [Piece; 4] {
    if is_white {
        [Piece::WhiteQueen, Piece::WhiteRook, Piece::WhiteBishop, Piece::WhiteKnight]
    } else {
        [Piece::BlackQueen, Piece::BlackRook, Piece::BlackBishop, Piece::BlackKnight]
    }
}

/// All fully-legal moves for one piece (pins, check, castling rules, and
/// promotion all correctly enforced via `is_legal`).
pub fn legal_moves_for_piece(
    board: &Board,
    piece_type: Piece,
    from_rank: usize,
    from_file: usize,
) -> Vec<Move> {
    let mut moves = Vec::new();

    for to in pseudo_legal_destinations(board, piece_type, from_rank, from_file) {
        let reaches_back_rank = matches!(
            (piece_type, to.0),
            (Piece::WhitePawn, 7) | (Piece::BlackPawn, 0)
        );

        if reaches_back_rank {
            for promo in promotion_choices(piece_type.is_white()) {
                let mv = Move::new_promotion(
                    ActivePiece { piece_type, from_rank, from_file },
                    to,
                    promo,
                );
                if mv.is_legal(board) {
                    moves.push(mv);
                }
            }
        } else {
            let mv = Move::new(ActivePiece { piece_type, from_rank, from_file }, to);
            if mv.is_legal(board) {
                moves.push(mv);
            }
        }
    }

    moves
}

/// Every fully-legal move for every piece belonging to `is_white`.
pub fn generate_legal_moves(board: &Board, is_white: bool) -> Vec<Move> {
    let mut moves = Vec::new();

    for piece_type in Piece::all_of_color(is_white) {
        let mut bb: u64 = board[piece_type];
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            bb &= bb - 1; // pop lowest set bit, advance to the next piece of this type

            moves.extend(legal_moves_for_piece(board, piece_type, sq / 8, sq % 8));
        }
    }

    moves
}

/// True if `is_white` has no legal moves and is currently in check (loss).
pub fn is_checkmate(board: &Board, is_white: bool) -> bool {
    Move::king_in_check(board, is_white) && generate_legal_moves(board, is_white).is_empty()
}

/// True if `is_white` has no legal moves but is not in check (draw).
pub fn is_stalemate(board: &Board, is_white: bool) -> bool {
    !Move::king_in_check(board, is_white) && generate_legal_moves(board, is_white).is_empty()
}