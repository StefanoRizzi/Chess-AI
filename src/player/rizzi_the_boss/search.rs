
use super::*;

impl BossPlayer {
    pub fn search_minimax(&mut self, chess: &mut Chess, depth: u16) -> Eval {
        if depth == 0 {return self.evaluate(chess)}
        
        let moves = chess.generate_legal_moves();
        if chess.is_finished(&moves) {
            match chess.get_outcome(&moves) {
                ChessOutcome::Draw => return 0,
                _ => return -Eval::MAX,
            }
        }

        let mut best_evaluation = -Eval::MAX;
        for r#move in moves {
            chess.make_move(r#move);
            let eval = -self.search_minimax(chess, depth - 1);
            best_evaluation = best_evaluation.max(eval);
            chess.unmake_move(r#move);
        }
        best_evaluation
    }

    pub fn search_ab(&mut self, chess: &mut Chess, depth: u16, mut alpha: Eval, beta: Eval) -> Eval {
        if depth == 0 {return self.search_all_captures(chess, alpha, beta)}

        let mut moves = chess.generate_legal_moves();
        if chess.is_finished(&moves) {
            match chess.get_outcome(&moves) {
                ChessOutcome::Draw => return 0,
                _ => return -Eval::MAX,
            }
        }
        self.order_moves(chess, &mut moves);

        for r#move in moves {
            chess.make_move(r#move);
            let eval = -self.search_ab(chess, depth - 1, -beta, -alpha);
            chess.unmake_move(r#move);
            if eval >= beta {return beta}
            alpha = alpha.max(eval);
        }
        alpha
    }

    pub fn search_all_captures(&mut self, chess: &mut Chess, mut alpha: Eval, beta: Eval) -> Eval {
        let mut eval = self.evaluate(chess);
        if eval >= beta {return beta}
        alpha = alpha.max(eval);

        let mut capture_moves = chess.generate_legal_moves();
        capture_moves.retain(|&r#move| chess.board(r#move.target()) != NONE || r#move.flag() == EN_PASSANT_FLAG);
        self.order_moves(chess, &mut capture_moves);

        for r#move in capture_moves {
            chess.make_move(r#move);
            eval = -self.search_all_captures(chess, -beta, -alpha);
            chess.unmake_move(r#move);
            if eval >= beta {return beta}
            alpha = alpha.max(eval);
        }
        alpha
    }

    pub fn order_moves(&mut self, chess: &mut Chess, moves: &mut Vec<Move>) {
        moves.sort_by_cached_key(|r#move| {
            let mut move_score_guess = 0;
            let move_piece_type = chess.board(r#move.start()).get_type();
            let capture_piece_type = chess.board(r#move.target()).get_type();
        
            if capture_piece_type != NONE_TYPE {
                move_score_guess = 10 * capture_piece_type.get_piece_value() - move_piece_type.get_piece_value();
            }

            if r#move.is_promotion() {
                move_score_guess += r#move.promotion_type().get_piece_value();
            }

            if chess.side[chess.opponent_index()].piece_attacks[PAWN.piece_index()][r#move.target() as usize] > 0 {
                move_score_guess -= move_piece_type.get_piece_value();
            }
            -move_score_guess
        });
    }
}