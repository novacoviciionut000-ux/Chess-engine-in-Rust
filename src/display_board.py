"""
Chess board viewer. Reads a FEN string from board_state.fen and redraws.
Supports drag-and-drop: writes user moves (e.g., 'e2e4') to player_move.txt.

Run this in a separate terminal from the Rust game:
    pip install pygame
    python board_viewer.py
"""

import sys
import pygame

FEN_PATH = "board_state.fen"
MOVE_PATH = "player_move.txt"
SQUARE_SIZE = 80
BOARD_SIZE = SQUARE_SIZE * 8
LIGHT_SQUARE = (240, 217, 181)
DARK_SQUARE = (181, 136, 99)
WHITE_PIECE_COLOR = (250, 250, 250)
BLACK_PIECE_COLOR = (15, 15, 15)

UNICODE_PIECES = {
    "P": "\u2659", "N": "\u2658", "B": "\u2657", "R": "\u2656", "Q": "\u2655", "K": "\u2654",
    "p": "\u265F", "n": "\u265E", "b": "\u265D", "r": "\u265C", "q": "\u265B", "k": "\u265A",
}


def parse_fen_placement(fen):
    """Returns an 8x8 list, board[rank][file], with rank 0 = FEN's rank 8 (top row)."""
    placement = fen.split(" ")[0]
    rows = placement.split("/")
    board = [[None] * 8 for _ in range(8)]
    for rank_index, row in enumerate(rows):
        file_index = 0
        for ch in row:
            if ch.isdigit():
                file_index += int(ch)
            else:
                board[rank_index][file_index] = ch
                file_index += 1
    return board


def read_fen():
    try:
        with open(FEN_PATH, "r") as f:
            return f.read().strip()
    except FileNotFoundError:
        return None


def write_move(move_str):
    """Writes the move to a file for the Rust engine to read."""
    with open(MOVE_PATH, "w") as f:
        f.write(move_str)


def find_piece_font():
    """Unicode chess glyphs need a font that actually has them."""
    candidates = ["Segoe UI Symbol", "DejaVu Sans", "Arial Unicode MS", "Noto Sans Symbols"]
    available = pygame.font.get_fonts()
    for name in candidates:
        key = name.lower().replace(" ", "")
        if key in available:
            return pygame.font.SysFont(name, int(SQUARE_SIZE * 0.7))
    return pygame.font.SysFont(None, int(SQUARE_SIZE * 0.7))


def get_square_name(rank, file):
    """Converts grid indices to chess coordinates (e.g., rank=6, file=4 -> 'e2')."""
    file_char = chr(ord('a') + file)
    rank_char = str(8 - rank)
    return file_char + rank_char


def draw_board(screen, board, font, inscription_font):
    for rank in range(8):
        for file in range(8):
            is_light_square = (rank + file) % 2 == 0
            color = LIGHT_SQUARE if is_light_square else DARK_SQUARE
            rect = pygame.Rect(file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE)
            pygame.draw.rect(screen, color, rect)

            # Draw inscriptions (rank numbers on the left, file letters on the bottom)
            text_color = DARK_SQUARE if is_light_square else LIGHT_SQUARE
            if file == 0:
                rank_text = inscription_font.render(str(8 - rank), True, text_color)
                screen.blit(rank_text, (rect.left + 3, rect.top + 3))
            if rank == 7:
                file_text = inscription_font.render(chr(ord('a') + file), True, text_color)
                screen.blit(file_text, (rect.right - 12, rect.bottom - 18))

            # Draw pieces
            piece = board[rank][file]
            if piece is None:
                continue

            glyph = UNICODE_PIECES.get(piece, piece)
            piece_color = WHITE_PIECE_COLOR if piece.isupper() else BLACK_PIECE_COLOR
            text_surface = font.render(glyph, True, piece_color)
            text_rect = text_surface.get_rect(center=rect.center)
            screen.blit(text_surface, text_rect)


def main():
    pygame.init()
    screen = pygame.display.set_mode((BOARD_SIZE, BOARD_SIZE))
    pygame.display.set_caption("Chess Board Viewer")
    
    font = find_piece_font()
    inscription_font = pygame.font.SysFont(None, 22)

    last_fen = None
    board = [[None] * 8 for _ in range(8)]
    clock = pygame.time.Clock()

    # Drag and drop state
    dragging = False
    drag_piece = None
    drag_start_tuple = None

    running = True
    while running:
        mouse_pos = pygame.mouse.get_pos()
        
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            
            elif event.type == pygame.MOUSEBUTTONDOWN and event.button == 1:
                # Calculate which square was clicked
                f = mouse_pos[0] // SQUARE_SIZE
                r = mouse_pos[1] // SQUARE_SIZE
                
                if 0 <= f < 8 and 0 <= r < 8 and board[r][f] is not None:
                    dragging = True
                    drag_piece = board[r][f]
                    drag_start_tuple = (r, f)
                    board[r][f] = None  # Temporarily remove piece from grid while dragging

            elif event.type == pygame.MOUSEBUTTONUP and event.button == 1:
                if dragging:
                    f = mouse_pos[0] // SQUARE_SIZE
                    r = mouse_pos[1] // SQUARE_SIZE
                    
                    # Put the piece back down on its original square visually.
                    # It will stay there until the Rust engine validates the move 
                    # and pushes a new FEN.
                    board[drag_start_tuple[0]][drag_start_tuple[1]] = drag_piece
                    
                    # If dropped on a new valid square, write the move
                    if 0 <= f < 8 and 0 <= r < 8 and (r, f) != drag_start_tuple:
                        start_sq = get_square_name(drag_start_tuple[0], drag_start_tuple[1])
                        end_sq = get_square_name(r, f)
                        move_str = start_sq + end_sq
                        write_move(move_str)
                    
                    dragging = False
                    drag_piece = None

        # Check for FEN updates
        fen = read_fen()
        # Only update the board state from FEN if we aren't currently dragging a piece
        # This prevents the board from "snapping" mid-drag if the engine thinks.
        if fen and fen != last_fen and not dragging:
            last_fen = fen
            board = parse_fen_placement(fen)

        # Render
        draw_board(screen, board, font, inscription_font)
        
        # Draw the dragged piece floating under the mouse
        if dragging and drag_piece:
            glyph = UNICODE_PIECES.get(drag_piece, drag_piece)
            piece_color = WHITE_PIECE_COLOR if drag_piece.isupper() else BLACK_PIECE_COLOR
            text_surface = font.render(glyph, True, piece_color)
            text_rect = text_surface.get_rect(center=mouse_pos)
            screen.blit(text_surface, text_rect)

        pygame.display.flip()
        clock.tick(60)  # Bumped to 60fps for smooth piece dragging

    pygame.quit()
    sys.exit()


if __name__ == "__main__":
    main()