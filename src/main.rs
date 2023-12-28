
use std::time::Duration;

use chess_rust::{*, legal_moves::precompute, rizzi_the_boss::evaluation};

fn main() {
    precompute();
    //unsafe {DISPLAY = false}

    
    /*let mut chess = Chess::build("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1");
    let mut boss = BossPlayer::new();
    //boss.depth = 20;
    boss.best_move(&mut chess, Some(Duration::from_millis(1000)));
    let chess = Chess::build("8/7k/3p4/p2P1p2/P2P1P2/8/8/7K w - - 0 1");
    println!("{:?}", boss.transposition_table.get_entry(chess.hash()));
    return;*/
    
    //let mut chess = Chess::build("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1");
    //BossPlayer::new().best_move(&mut chess, Some(Duration::from_millis(500)));
    //play(&mut chess, &mut BossPlayer::new(), &mut BossPlayer::new(), Some(Duration::from_millis(150)));
    //return;

    //benchmark(4);
    //play(&mut Chess::position(1), &mut HumanPlayer::new("H1".to_string()), &mut BossPlayer::new());
    //compete(&mut BossPlayer::new(), &mut BadPlayer::new(), 1000);
    //return;
    let mut engine = BossPlayer::new().into_engine_uci();
    let mut message = String::new();
    engine.greet();

    loop {
        std::io::stdin().read_line(&mut message).unwrap();
        engine.received_command(&message.trim());
        if message == "quit\n" {break}
        message.clear();
    }
}