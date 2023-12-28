
use super::*;

impl BossPlayer {
    pub fn search(&mut self, chess: &mut Chess, depth: u16) -> (Move, Eval) {
        self.evaluated = 0;
        let hash = chess.hash();
        let mut moves = chess.generate_legal_moves();
        let mut best_move = moves.first().unwrap().clone();
        let mut best_eval = -Eval::MAX;
        
        for depth_iter in 1..=depth {
            
            self.order_moves(chess, &mut moves);
            let mut best_eval_this_iter = -Eval::MAX;
            'search: {
                for &r#move in &moves {
                    
                    chess.make_move(r#move);
                    let eval = -self.search_ab(chess, depth_iter - 1, -Eval::MAX, -best_eval_this_iter);
                    chess.unmake_move(r#move);

                    if self.search_canceled.load(Ordering::Relaxed) {break 'search}

                    if eval > best_eval_this_iter {
                        best_eval_this_iter = eval;
                        best_eval = eval;
                        best_move = r#move;
                    }
                }
                self.transposition_table.put_entry(TableEntry::new(hash, TypeNode::PV, best_move, depth_iter, best_eval));
            }
            write_to_log(&format!("[depth = {depth_iter}] Eval {best_eval}"));
            if best_eval == Eval::MAX || best_eval == -Eval::MAX {break}
            if self.search_canceled.load(Ordering::Relaxed) {break}
        }
        (best_move, best_eval)
    }

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
        if self.search_canceled.load(Ordering::Relaxed) {return 00}
        if depth == 0 {return self.search_all_captures(chess, alpha, beta)}

        let mut moves = chess.generate_legal_moves();
        if chess.is_finished(&moves) {
            self.evaluated += 1;
            match chess.get_outcome(&moves) {
                ChessOutcome::Draw => return 0,
                _ => return -Eval::MAX,
            }
        }
        
        if let Some(entry) = self.transposition_table.get_entry(chess.hash()) {
            if entry.depth >= depth {
                match entry.node {
                    TypeNode::PV => return entry.score,
                    TypeNode::All => if entry.score <= alpha {return alpha}
                    TypeNode::Cut => if entry.score >= beta {return beta}
                }
            }
        }
        self.order_moves(chess, &mut moves);

        let mut node = TypeNode::All;
        let mut best_move = moves.first().unwrap().clone();
        for r#move in moves {
            
            chess.make_move(r#move);
            let eval = -self.search_ab(chess, depth - 1, -beta, -alpha);
            chess.unmake_move(r#move);

            if self.search_canceled.load(Ordering::Relaxed) {return 00}

            if eval >= beta {
                self.transposition_table.put_entry(TableEntry::new(chess.hash(), TypeNode::Cut, r#move, depth, beta));
                return beta;
            }
            if eval > alpha {
                alpha = eval;
                best_move = r#move;
                node = TypeNode::PV;
            }
        }
        self.transposition_table.put_entry(TableEntry::new(chess.hash(), node, best_move, depth, alpha));
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
        let best_move = self.transposition_table.get_entry(chess.hash()).map_or(NONE_MOVE, |e|e.r#move);
        moves.sort_by_cached_key(|&r#move| {
            if r#move == best_move {return -Eval::MAX}
            
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