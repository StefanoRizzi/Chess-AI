
use std::thread;
use std::{time::*, mem::size_of};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use super::*;
pub mod transposition_table;
pub mod evaluation;
pub mod search;
pub use transposition_table::*;

pub struct BossPlayer {
    pub transposition_table: TranspositionTable,
    pub evaluated: u32,
    pub depth: u16,
    pub search_canceled: Arc<AtomicBool>,
    pub nodes: u32,
    pub print_info: bool,
}

impl BossPlayer {
    pub fn new() -> BossPlayer {BossPlayer { transposition_table: TranspositionTable::new(), evaluated: 0, depth: 4, search_canceled: Arc::new(AtomicBool::new(false)), nodes: 0, print_info: true }}
}

impl ChessPlayer for BossPlayer {
    fn name(&self) -> &str {"RizziTheBoss"}
    fn notify_new_game(&self) {}
    fn set_position(&mut self, chess: &Chess) {}
    fn get_stop(&self) -> Arc<AtomicBool> {self.search_canceled.clone()}

    fn best_move(&mut self, chess: &mut Chess, time: Option<Duration>) -> (Move, Eval) {
        let max_depth = if time.is_none() {self.depth} else {u16::MAX};
        let t_start = Instant::now();
        
        self.search_canceled.store(false, Ordering::Relaxed);
        let thread_search_canceled = self.search_canceled.clone();
        let thread_canceled = Arc::new(AtomicBool::new(false));
        if let Some(time) = time {
            let thread_canceled = thread_canceled.clone();
            thread::spawn(move || {
                thread::sleep(time);
                if !thread_canceled.load(Ordering::Relaxed) {
                    thread_search_canceled.store(true, Ordering::Relaxed);
                }
            });
        }

        clear_log();
        fn write_log_and_print(depth: u16, eval: Eval, time: u64, nodes: u32, nps: u32, pv: Move) {
            write_to_log(&format!("info depth {depth} score cp {eval} time {time} nodes {nodes} nps {nps} pv {}", pv.to_text()));
            println!("info depth {depth} score cp {eval} time {time} nodes {nodes} nps {nps} pv {}", pv.to_text());
        }
        fn write_log(depth: u16, eval: Eval, time: u64, nodes: u32, nps: u32, pv: Move) {
            write_to_log(&format!("info depth {depth} score cp {eval} time {time} nodes {nodes} nps {nps} pv {}", pv.to_text()));
        }
        let send_info = if self.print_info { write_log_and_print } else { write_log };
        let (mut best_move, eval) = self.search(chess, max_depth, send_info);
        
        if eval == Eval::MAX {
            let old_tt = std::mem::replace(&mut self.transposition_table, TranspositionTable::new());
            let (best_move_shorter_checkmate, eval_shorter_checkmate) = self.search(chess, max_depth-1, write_log);
            if eval_shorter_checkmate == Eval::MAX {
                best_move = best_move_shorter_checkmate;
            } else {
                self.transposition_table = old_tt;
            }
        }

        thread_canceled.store(true, Ordering::Relaxed);
        let time = Instant::now() - t_start;

        write_to_log(&format!("\nTime: {:.2} seconds\nEvaluated: {} positions", time.as_secs_f32(), self.evaluated));
        let tt = &self.transposition_table;
        write_to_log(&format!("\nTransposition Table\n Size: {} mb\n Transpositions: {}\n Occupancy: {:.1}%\n Overwrites: {:.1}%\n Collisions: {:.1}%\n",
            TRANSPOSITION_TABLE_SIZE_MB,
            self.transposition_table.occupancy,
            100.0 * tt.occupancy as f32 / NUM_ENTRIES as f32,
            100.0 * tt.overwrites as f32 / NUM_ENTRIES as f32,
            100.0 * tt.collisions as f32 / NUM_ENTRIES as f32,
            //tt.hash_collision,
        ));
        
        /*write_to_log("\nHashes {");
        for s in &chess.irreversable_state {
            write_to_log(&format!("  {:?}", s.4));
        }
        write_to_log(&format!("}}\nRepetitions: {}", chess.get_repetitions()));
        */
        return (best_move, eval);
    }
    fn make_move(&mut self, r#move: Move) {}
    fn evaluate_infinite(&mut self,
        chess: &mut Chess,
        send_info: fn(depth: u16, eval: Eval, time: u64, nodes: u32, nps: u32, pv: Move),
    ) {
        self.search_canceled.store(false, Ordering::Relaxed);

        clear_log();
        let write_log = |depth: u16, eval: Eval, time: u64, nodes: u32, nps: u32, pv: Move| {
            write_to_log(&format!("info depth {depth} score cp {eval} time {time} nodes {nodes} nps {nps} pv {}", pv.to_text()));
            send_info(depth, eval, time, nodes, nps, pv);
        };
        
        let (mut best_move, eval) = self.search(chess, u16::MAX, write_log);
        
        if eval == Eval::MAX {
            let old_tt = std::mem::replace(&mut self.transposition_table, TranspositionTable::new());
            let (best_move_shorter_checkmate, eval_shorter_checkmate) = self.search(chess, u16::MAX, write_log);
            if eval_shorter_checkmate == Eval::MAX {
                best_move = best_move_shorter_checkmate;
            } else {
                self.transposition_table = old_tt;
            }
        }
    }
}

mod time_tests {
    use std::time::*;

    use super::*;
    
    #[test]
    fn minimax_vs_alpha_beta() {
        for pos in [1, 3, 4, 5] {
            println!("[pos: {pos}]");
            //min-max
            let mut chess = Chess::position(pos);
            let mut boss = BossPlayer::new();
            /*for depth in 1..=5 {
                let t_start = Instant::now();
                boss.search_minimax(&mut chess, depth);
                println!("[Min-Max] Depth: {depth} Time: {:?}", t_start.elapsed());
            }*/
            /*for depth in 5..=6 {
                let t_start = Instant::now();
                boss.search_ab_no_table(&mut chess, depth, -i16::MAX, i16::MAX, false);
                println!("[Alpha-Beta] Depth: {depth} Time: {:?}", t_start.elapsed());
            }*/
            /*for depth in 7..=7 {
                let t_start = Instant::now();
                boss.search_ab_no_table(&mut chess, depth, -i16::MAX, i16::MAX, true);
                println!("[Alpha-Beta ordina] Depth: {depth} Time: {:?}", t_start.elapsed());
            }*/
            for depth in 8..=8 {
                let t_start = Instant::now();
                boss.search_ab(&mut chess, depth, -i16::MAX, i16::MAX);
                println!("[Alpha-Beta hash tale] Depth: {depth} Time: {:?}", t_start.elapsed());
            }
            for depth in 8..=8 {
                let mut chess = Chess::position(pos);
                let mut boss = BossPlayer::new();
                let t_start = Instant::now();
                for iter_depth in 1..=depth {
                    boss.search_ab(&mut chess, iter_depth, -i16::MAX, i16::MAX);
                }
                println!("[Alpha-Beta DFID] Depth: {depth} Time: {:?}", t_start.elapsed());
            }
        }
        panic!()
    }

    #[test]
    fn alpha_beta_vs_dfid() {
        let fen = "r1bq1rk1/ppppnppp/4pn2/1B6/4P3/2PP1N2/P1P2PPP/R1BQ1RK1 w - - 3 8";
        //for pos in [1, 3, 4, 5] {
            for depth in [8] {
                //println!("AB vs DFID [pos: {pos}] [Depth: {depth}]");
                //alpha-beta
                let mut chess = Chess::build(fen);
                let mut boss = BossPlayer::new();
                
                let t_start = Instant::now();
                boss.search_ab(&mut chess, depth, -i16::MAX, i16::MAX);
                println!("[AB] Time: {:?}", t_start.elapsed());
                
                //dfid
                let mut chess = Chess::build(fen);
                let mut boss = BossPlayer::new();

                let t_start = Instant::now();
                for iter_depth in 1..=depth {
                    boss.search_ab(&mut chess, iter_depth, -i16::MAX, i16::MAX);
                }
                println!("[DFID] Time: {:?}", t_start.elapsed());
            }
        //}
        panic!()
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

    fn boss_fight(fen: &str, outcome_reference: ChessOutcome, time: Option<Duration>) {
        let mut chess = Chess::build(fen);
        let outcome = play(&mut chess, &mut BossPlayer::new(), &mut BossPlayer::new(), time);
        assert_eq!(outcome, outcome_reference);
    }
    #[test]
    fn checkmate_in_one() {
        boss_fight("k7/pp6/r7/8/8/8/PP6/K6R w - - 0 1",
        ChessOutcome::WhiteWinner,
        None,
    );
    }
    #[test]
    fn one_rook_endgame() {
        boss_fight("RK6/8/8/8/3k4/8/8/8 w - - 0 1",
        ChessOutcome::WhiteWinner,
        None,
    );
    }
    #[test]
    fn two_rooks_endgame() {
        boss_fight("RRK5/8/8/8/3k4/8/8/8 w - - 0 1",
        ChessOutcome::WhiteWinner,
        None,
    );
    }
    #[test]
    fn queen_pawn_endgame() {
        boss_fight("8/2K5/3P4/8/8/8/8/6qk w - - 0 1",
        ChessOutcome::BlackWinner,
        Some(Duration::from_millis(50)),
    );
    }
    #[test]
    fn pawns_endgame() {
        boss_fight("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1",
        ChessOutcome::WhiteWinner,
        Some(Duration::from_millis(250)),
    );
    }
}