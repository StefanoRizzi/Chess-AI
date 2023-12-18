use super::*;

impl Chess {
    pub fn build(fen: &str) -> Chess {
        legal_moves::precompute();
        
        let mut iter = fen.trim().split(" ");
        let mut chess = Chess {
            board: [NONE; 64],
            en_passant: u8::MAX,
            castling: CASTLE_NONE,
            half_move: 0,
            full_turn: 1,
            is_white_to_move: true,
            side: [Default::default(), Default::default()],
            irreversable_state: Vec::new(),
            moves_history: Vec::new(),
        };
        chess.irreversable_state.reserve_exact(MAX_DEPTH);
        chess.moves_history.reserve_exact(MAX_MOVES);
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
                let colour = piece.get_colour();

                chess.side[colour.colour_index()].new_piece(piece.get_type(), square as u8);
                chess.put_attack_and_update(piece.get_type(), colour, square as u8);
                chess.board[square] = piece;
                file += 1
            }
        }
        // Side to move
        chess.is_white_to_move = iter.next() == Some("w");
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