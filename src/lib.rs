pub mod chess;
pub mod player;

use std::sync::Mutex;
use std::{thread::sleep, time::Duration};

pub use chess::*;
pub use player::*;

pub use std::io::Write;
pub use std::fs::File;
pub use std::path::PathBuf;
// SETTINGS
pub static mut DISPLAY: bool = true;
static ROOT_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub static  LOG: Mutex<Option<File>> = Mutex::new(None);
pub fn write_to_log(message: &str) {
    (*LOG.lock().unwrap()).as_mut().unwrap().write(message.as_bytes()).unwrap();
    (*LOG.lock().unwrap()).as_mut().unwrap().write("\n".as_bytes()).unwrap();
}
pub fn clear_log() {
    let path = ROOT_PATH.lock().unwrap().as_ref().unwrap().join("chess_log.txt");
    *LOG.lock().unwrap() = Some(File::create(path).unwrap());
}

pub fn benchmark(depth: u16) {
    let mut chess = Chess::start_position();
    chess.time_perft(depth);
}

pub fn compete(player_1: &mut dyn ChessPlayer, player_2: &mut dyn ChessPlayer, games: u32) {
    let (mut won, mut lost, mut draw) = (0, 0, 0);
    for n in 0..games {
        println!("Game {} of {games}", n+1);
        let mut chess = Chess::start_position();
        if n % 2 == 0 {
            let outcome = play(&mut chess, player_1, player_2, None);
            match outcome {
                ChessOutcome::Draw => draw += 1,
                ChessOutcome::WhiteWinner => won += 1,
                ChessOutcome::BlackWinner => lost += 1,
            }
        }
        else {
            let outcome = play(&mut chess, player_2, player_1, None);
            match outcome {
                ChessOutcome::Draw => draw += 1,
                ChessOutcome::WhiteWinner => lost += 1,
                ChessOutcome::BlackWinner => won += 1,
            }
        }
        println!("[P1] Won: {won} Draw: {draw} Lost: {lost}");
    }
}

pub fn play(chess: &mut Chess, player_1: &mut dyn ChessPlayer, player_2: &mut dyn ChessPlayer, time: Option<Duration>) -> ChessOutcome {
    player_1.notify_new_game();
    player_2.notify_new_game();
    player_1.set_position(&chess);
    player_2.set_position(&chess);
    chess.display();
    let mut moves;
    loop {
        moves = chess.generate_legal_moves();
        if chess.is_finished(&moves) {break}
        sleep(Duration::from_millis(1));
        let (r#move, _) = player_1.best_move(chess, time);
        chess.make_move(r#move);
        player_1.make_move(r#move);
        player_2.make_move(r#move);
        chess.update_display(r#move);

        moves = chess.generate_legal_moves();
        if chess.is_finished(&moves) {break}
        sleep(Duration::from_millis(1));
        let (r#move, _) = player_2.best_move(chess, time);
        chess.make_move(r#move);
        player_1.make_move(r#move);
        player_2.make_move(r#move);
        chess.update_display(r#move);
    }
    chess.get_outcome(&moves)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chess_new() {
        let chess = chess::Chess::start_position();
        assert_eq!(chess.board(3), Piece::new(QUEEN, WHITE));
        assert_eq!(chess.board(4), Piece::new(KING, WHITE));
    }
    #[test]
    fn chess_move() {
        let mut chess = chess::Chess::start_position();
        chess.make_move(Move::new(10, 18, NO_FLAG));
        assert_eq!(chess.board(10), NONE);
        assert_eq!(chess.board(18), Piece::new(PAWN, WHITE));
        chess.unmake_move(Move::new(10, 18, NO_FLAG));
        assert_eq!(chess.board(10), Piece::new(PAWN, WHITE));
        assert_eq!(chess.board(18), NONE);
    }
}
