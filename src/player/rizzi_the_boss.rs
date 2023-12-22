
use super::*;
pub mod evaluation;
pub mod search;

pub struct BossPlayer {
    depth: u16,
}

impl BossPlayer {
    pub fn new() -> BossPlayer {BossPlayer { depth: 8 }}
}

impl ChessPlayer for BossPlayer {
    fn name(&self) -> &str {"RizziTheBoss"}
    fn notify_new_game(&self) {}
    fn set_position(&mut self, chess: &Chess) {}
    fn best_move(&mut self,  chess: &mut Chess, time: Option<(Duration, Duration)>) -> Move {
        let mut moves = chess.generate_legal_moves();
       
        self.order_moves(chess, &mut moves);
        let mut alpha = -Eval::MAX;
        let mut best_move = *moves.first().unwrap();
        for r#move in moves {
            chess.make_move(r#move);
            let eval = -self.search_ab(chess, self.depth - 1, -Eval::MAX, -alpha);
            write_to_log(&format!("Move {} Eval= {eval}", r#move.to_text()));
            chess.unmake_move(r#move);
            if eval > alpha {
                alpha = eval;
                best_move = r#move;
            }
        }
        write_to_log(&format!("--- Eval= {alpha} ---\n"));
        
        write_to_log("Hashes:");
        for s in &chess.irreversable_state {
            write_to_log(&format!("{:?}", s.4));
        }
        write_to_log(&format!("{}", chess.get_repetitions()));
        best_move
    }
    fn make_move(&mut self, r#move: Move) {}
    fn evaluate_infinite(&mut self, chess: &mut Chess) -> Eval {
        self.search_ab(chess, self.depth, -Eval::MAX, Eval::MAX)
    }
}


#[cfg(test)]
mod boss_tests {
    use std::time::*;

    use super::*;

    #[test]
    fn minimax() {
        let mut chess = Chess::position(2);
        let mut boss = BossPlayer::new();
        for depth in 1..=4 {
            let t_start = Instant::now();
            boss.search_minimax(&mut chess, depth);
            println!("Depth: {depth} Time: {:?}", Instant::now() - t_start);
        }
    }

    #[test]
    fn alpha_beta() {
        let mut chess = Chess::position(2);
        let mut boss = BossPlayer::new();
        for depth in 1..=5 {
            let t_start = Instant::now();
            boss.search_ab(&mut chess, depth, -i16::MAX, i16::MAX);
            println!("Depth: {depth} Time: {:?}", Instant::now() - t_start);
        }
    }

    fn boss_fight(fen: &str, outcome_reference: ChessOutcome) {
        let mut chess = Chess::build(fen);
        let outcome = play(&mut chess, &mut BossPlayer::new(), &mut BossPlayer::new());
        assert_eq!(outcome, outcome_reference);
    }
    #[test]
    fn checkmate_in_one() {
        boss_fight("k7/pp6/r7/8/8/8/PP6/K6R w - - 0 1", ChessOutcome::WhiteWinner);
    }
    #[test]
    fn one_rook_endgame() {
        boss_fight("RK6/8/8/8/3k4/8/8/8 w - - 0 1", ChessOutcome::WhiteWinner);
    }
    #[test]
    fn two_rooks_endgame() {
        boss_fight("RRK5/8/8/8/3k4/8/8/8 w - - 0 1", ChessOutcome::WhiteWinner);
    }
    #[test]
    fn queen_pawn_endgame() {
        boss_fight("8/2k5/3p4/8/8/8/8/6QK w - - 0 1", ChessOutcome::WhiteWinner);
    }
}