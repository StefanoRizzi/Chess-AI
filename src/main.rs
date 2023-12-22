
use chess_rust::{*, legal_moves::precompute};

fn main() {
    precompute();
    //unsafe {DISPLAY = false}

    /*let mut chess = Chess::build("k7/pp6/r7/8/8/8/PP6/K6R w - - 0 1");
    play(&mut chess, &mut BossPlayer::new(), &mut BossPlayer::new());

    return;
    */

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