use super::*;

impl Chess {
    pub fn build(fen: &str) -> Chess {
        legal_moves::precomputed_move_data();
        
        let mut iter = fen.split(" ");
        let mut chess = Chess {
            board: [NONE;64],
            white_turn: true,
            castle: CASTLE_NONE,
            en_passant: u8::MAX,
            half_move: 0,
            turn: 1,
            irreversable_state: Vec::new()
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
                let piece_colour = if symbol.is_ascii_uppercase() {piece::WHITE} else {piece::BLACK};
                let piece_type = match symbol.to_ascii_lowercase() {
                    'p' => PAWN, 'n' => KNIGHT, 'b' => BISHOP,
                    'r' => ROCK, 'q' => QUEEN, 'k' => KING,
                    _ => NONE
                };
                chess.board[rank * 8 + file] = piece_type | piece_colour;
                file += 1
            }
        }
        // Side to move
        chess.white_turn = match iter.next() {
            Some("w") => true,
            Some("b") => false,
            _ => panic!("side to move")
        };
        // Castling ability
        let string = iter.next();
        if string != Some("-") {
            for ch in string.expect("castling ability").chars() {
                match ch {
                    'K' => chess.castle |= CASTLE_W_K,
                    'Q' => chess.castle |= CASTLE_W_Q,
                    'k' => chess.castle |= CASTLE_B_K,
                    'q' => chess.castle |= CASTLE_B_Q,
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
        let fen_half = iter.next().expect("FEN halfmove clock");
        chess.half_move = fen_half.parse().unwrap();
        // Fullmove counter
        let fen_full = iter.next().expect("FEN fullmove counter");
        chess.turn = fen_full.parse().unwrap();
        
        return chess
    }
}