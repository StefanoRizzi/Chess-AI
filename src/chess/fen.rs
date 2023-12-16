use super::*;

impl Chess {
    pub fn build(fen: &str) -> Chess {
        legal_moves::precompute();
        
        let mut iter = fen.trim().split(" ");
        let mut chess = Chess {
            board: [NONE;64],
            en_passant: u8::MAX,
            castling: CASTLE_NONE,
            half_move: 0,
            full_turn: 1,
            colour_turn: NONE_COLOUR,
            white_side: SideState { king_square: 0, pieces: Vec::new(), control: [0;64], slide_control: [0;64] },
            black_side: SideState { king_square: 0, pieces: Vec::new(), control: [0;64], slide_control: [0;64] },
            irreversable_state: Vec::new(),
        };
        chess.irreversable_state.reserve_exact(MAX_DEPTH);
        // Piece Placement
        let fen_board = iter.next().expect("FEN board");
        let (mut rank, mut file) = (7, 0);
        for symbol in fen_board.chars() {
            if symbol == '/' {
                rank -= 1;
                file = 0;
            } else if symbol.is_ascii_digit() {
                file += utils::number_from_0_9(symbol);
            } else {
                let square = rank * 8 + file;
                let piece = Piece::from_symbol(symbol);
                if piece.is_colour(WHITE) {
                    chess.white_side.add_piece_list(piece.get_type(), square as u8);
                    chess.update_control(WHITE, piece.get_type(), square as u8, 1);
                }
                else {
                    chess.black_side.add_piece_list(piece.get_type(), square as u8);
                    chess.update_control(BLACK, piece.get_type(), square as u8, 1);
                }
                chess.board[square] = piece;
                file += 1
            }
        }
        // Side to move
        chess.colour_turn = if iter.next() == Some("b") {BLACK} else {WHITE};
        // Castling ability
        let string = iter.next();
        if string != Some("-") {
            for ch in string.expect("castling ability").chars() {
                match ch {
                    'K' => chess.castling |= CASTLE_WHITE_KING,
                    'Q' => chess.castling |= CASTLE_WHITE_QUEEN,
                    'k' => chess.castling |= CASTLE_BLACK_KING,
                    'q' => chess.castling |= CASTLE_BLACK_QUEEN,
                    _ => panic!("castling ability")
                }
            }
        }
        // En passant target square
        let fen_en = iter.next().expect("FEN en passant");
        if fen_en != "-" {
            let mut chars = fen_en.chars();
            chess.en_passant = utils::square_from_text(chars.next().unwrap(), chars.next().unwrap());
        }
        // Halfmove clock
        if let Some(fen_half) = iter.next() {
            chess.half_move = fen_half.parse().unwrap();
        }
        // Fullmove counter
        if let Some(fen_full) = iter.next() {
            chess.full_turn = fen_full.parse().unwrap();
        }
        
        return chess
    }
}