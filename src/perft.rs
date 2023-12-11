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
            let nodes = self.perft(depth);
            println!("generation moves={:?} make move={:?} unmake move={:?}", GEN_MOVE_TIME/nodes as u32, MAKE_MOVE_TIME/nodes as u32, UNMAKE_MOVE_TIME/nodes as u32);
        }
    }

    pub fn perft(&mut self, depth: u16 /* assuming >= 1 */) -> u64 {
        assert!(depth >= 1);
        let mut nodes: u64 = 0;

        let t_start = Instant::now();
        let legal_moves = self.generate_legal_moves();
        unsafe {GEN_MOVE_TIME += Instant::now() - t_start}

        if depth == 1 {return legal_moves.len() as u64}

        for movement in legal_moves {
            let t_start = Instant::now();
            self.make_move(&movement);
            unsafe {MAKE_MOVE_TIME += Instant::now() - t_start}
            nodes += self.perft(depth - 1);
            let t_start = Instant::now();
            self.unmake_move(&movement);
            unsafe {UNMAKE_MOVE_TIME += Instant::now() - t_start}
        }
        return nodes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_test() {
        let mut chess = Chess::new();
        for (mut depth, &num_positions) in [20, 400, 8902, 197281, 4865609/*, 119060324*/].iter().enumerate() {
            depth += 1;
            let t_start = Instant::now();
            let res = chess.perft(depth as u16);
            let time = Instant::now() - t_start;
            println!("Depth: {depth} Result: {res} Time: {time:?}");
            assert_eq!(res, num_positions);
        }
    }

    #[test]
    fn perft_test_position_5() {
        let mut chess = Chess::build("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        for (mut depth, &num_positions) in [44, 1486, 62379, 2103487, 89941194].iter().enumerate() {
            depth += 1;
            let t_start = Instant::now();
            let res = chess.perft(depth as u16);
            let time = Instant::now() - t_start;
            println!("Depth: {depth} Result: {res} Time: {time:?}");
            assert_eq!(res, num_positions);
        }
    }
}