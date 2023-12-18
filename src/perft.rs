
use super::chess::*;
use std::time::*;


static mut GEN_MOVE_TIME: Duration = Duration::ZERO;
static mut MAKE_MOVE_TIME: Duration = Duration::ZERO;
static mut UNMAKE_MOVE_TIME: Duration = Duration::ZERO;

impl Chess {
    pub fn time_perft(&mut self, depth: u16 /* assuming >= 1 */) {
        unsafe {
            GEN_MOVE_TIME = Duration::ZERO;
            MAKE_MOVE_TIME = Duration::ZERO;
            UNMAKE_MOVE_TIME = Duration::ZERO;
            let _ = self.perft(depth);
            let tot = GEN_MOVE_TIME + MAKE_MOVE_TIME + UNMAKE_MOVE_TIME;
            println!("tot= {tot:?}\ngeneration moves= {:.2?}%\nmake move= {:.2?}%\nunmake move= {:.2?}%",
                100.0*GEN_MOVE_TIME.as_secs_f64()/tot.as_secs_f64(),
                100.0*MAKE_MOVE_TIME.as_secs_f64()/tot.as_secs_f64(),
                100.0*UNMAKE_MOVE_TIME.as_secs_f64()/tot.as_secs_f64(),
            );
        }
    }

    pub fn perft(&mut self, depth: u16 /* assuming >= 1 */) -> u64 {
        assert!(depth >= 1);
        let mut nodes: u64 = 0;

        let t_start = Instant::now();
        let legal_moves = self.generate_legal_moves();
        unsafe {GEN_MOVE_TIME += Instant::now() - t_start}

        if depth == 0 {return 1}
        
        for movee in legal_moves {
            
            let t_start = Instant::now();
            self.make_move(movee);
            unsafe {MAKE_MOVE_TIME += Instant::now() - t_start}

            nodes += self.perft(depth - 1);
            
            let t_start = Instant::now();
            self.unmake_move(movee);
            unsafe {UNMAKE_MOVE_TIME += Instant::now() - t_start}
        }
        return nodes;
    }

    pub fn perft_bulck(&mut self, depth: u16) -> u64 {
        assert!(depth >= 1);
        let mut nodes: u64 = 0;

        let legal_moves = self.generate_legal_moves();

        if depth == 1 {return legal_moves.len() as u64}
            
        for movee in legal_moves {
            self.make_move(movee);
            nodes += self.perft_bulck(depth - 1);
            self.unmake_move(movee);
        }
        return nodes;
    }
}

#[cfg(test)]
mod perft_tests {
    use super::*;

    fn perft_test(mut chess: Chess, nodes: &[u64]) {
        for (mut depth, &num_positions) in nodes.iter().enumerate() {
            if num_positions > 150000000 {break}
            depth += 1;
            let t_start = Instant::now();
            let res = chess.perft_bulck(depth as u16);
            let time = Instant::now() - t_start;

            println!("Depth: {depth} Result: {res} Time: {time:?}");
            if res != num_positions {
                chess.display();
                chess.display_attacks(BLACK);
                chess.display_attacks(WHITE);
                for movee in chess.generate_legal_moves() {
                    println!("{movee:?}");
                }
            }
            assert_eq!(res, num_positions, "my - real");
        }
    }

    #[test]
    fn position_1() {
        perft_test(
            Chess::new(),
            &[20, 400, 8902, 197281, 4865609, 119060324],
        );
    }

    #[test]
    fn position_2() {
        perft_test(
            Chess::build("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - "),
            &[48, 2039, 97862/*, 4085603, 193690690, 8031647685*/],
        );
    }

    #[test]
    fn position_3() {
        perft_test(
            Chess::build("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - "),
            &[14, 191, 2812, 43238, 674624, 11030083, 178633661, 3009794393],
        );
    }

    #[test]
    fn position_4() {
        perft_test(
            Chess::build("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
            &[6, 264, 9467, 422333, 15833292, 706045033],
        );
    }

    #[test]
    fn position_5() {
        perft_test(
            Chess::build("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"),
            &[44, 1486, 62379, 2103487, 89941194],
        );
    }

    #[test]
    fn position_6() {
        perft_test(
            Chess::build("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"),
            &[46, 2079, 89890, 3894594, 164075551],
        );
    }
}