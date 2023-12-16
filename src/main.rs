use std::io::{self, Write};
use chess_rust::chess::*;
use chess_rust::random_ai::BadPlayer;

struct HumanPlayer {

}
impl chess_rust::ChessPlayer for HumanPlayer {
    fn best_move(&mut self, chess: &mut Chess) -> Moves {
        loop {
            let mut cmd = String::new();
            print!("Human: "); io::stdout().flush().unwrap();
            io::stdin().read_line(&mut cmd).unwrap();
            print!("\x1b[A\x1b[J"); io::stdout().flush().unwrap();
            
            let movee = utils::gen_move(&chess, &cmd);
            if chess.generate_legal_moves().contains(&movee) {
                return movee;
            }
        }
    }
}

fn main() {
    let mut chess = Chess::build("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ");
    println!("{:?}", chess.perft(6));
    return;
    /*for j in (0..8).rev() {
        for i in 0..8 {
            print!("  {}", chess.white_side.control[i+j*8]);
        }
        println!()
    }*/
    /*for movee in chess.generate_legal_moves() {
        chess.make_move(movee);
        println!("{movee:?} num {}", chess.perft(2));
        
        let the_move = Moves::Move { start: 59, target: 32 };
        if !chess.generate_legal_moves().contains(&the_move) {
            println!("{movee:#?} {}", chess.generate_legal_moves().len());
            panic!();
            if movee != (Moves::Move { start: 2, target: 47 }) {
                panic!()
            }
        }
        chess.unmake_move(movee);
    }*/
    let movee = Moves::DoublePush { start: 11, target: 27 }; if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee); chess.unmake_move(movee); if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee);
    let movee = Moves::Move { start: 16, target: 20 }; if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee); chess.unmake_move(movee); if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee);
    chess.display();
    //let movee = Moves::Move { start: 13, target: 23 }; if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee); chess.unmake_move(movee); if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee);
    //let movee = Moves::Move { start: 6, target: 7 }; if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee); chess.unmake_move(movee); if !chess.generate_legal_moves().contains(&movee) {panic!()}; chess.make_move(movee);
    println!("{:#?}", chess.generate_legal_moves().len());
    //chess_rust::benchmark();
    //chess_rust::compete(&mut HumanPlayer{}, &mut HumanPlayer{}, 900);
    //chess_rust::compete(&mut BadPlayer::new(), &mut BadPlayer::new(), 900);
}