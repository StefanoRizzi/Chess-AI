pub mod chess;
pub mod perft;
pub mod random_ai;
pub mod rizzi_the_boss;
use chess::*;

use rand::Rng;

pub trait ChessPlayer {
    fn best_move(&mut self, chess: &mut Chess) -> Move;
}

pub fn benchmark() {
    let mut chess = Chess::new();
    chess.time_perft(5);
}

pub fn compete(player_1: &mut dyn ChessPlayer, player_2: &mut dyn ChessPlayer, games: u32) {
    let (mut won, mut lost, mut draw) = (0, 0, 0);
    for _ in 0..games {
        let mut chess = Chess::new();
        play(&mut chess, player_1, player_2);
        if chess.half_move >= 100 {draw += 1} 
        else if chess.white_turn {lost += 1}
        else {won += 1}
    }
    println!("won: {won} draw: {draw} lost: {lost}");
}

pub fn play(mut chess: &mut Chess, player_1: &mut dyn ChessPlayer, player_2: &mut dyn ChessPlayer) {
    chess.display();
    loop {
        if chess.half_move >= 100 {break}
        if chess.generate_legal_moves().len() == 0 {break}
        let movement = player_1.best_move(&mut chess);
        chess.make_move(&movement);
        chess.update_display(&movement);

        if chess.half_move >= 100 {break}
        if chess.generate_legal_moves().len() == 0 {break}
        let movement = player_2.best_move(&mut chess);
        chess.make_move(&movement);
        chess.update_display(&movement);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chess_new() {
        let chess = chess::Chess::new();
        assert_eq!(chess.board[3], piece::QUEEN | piece::WHITE);
        assert_eq!(chess.board[4], piece::KING | piece::WHITE);
    }
    #[test]
    fn chess_move() {
        let mut chess = chess::Chess::new();
        chess.make_move(&Move { start_square: 10, target_square: 18, promotion: 0});
        assert_eq!(chess.board[10], piece::NONE);
        assert_eq!(chess.board[18], piece::PAWN | piece::WHITE);
        chess.unmake_move(&Move { start_square: 10, target_square: 18, promotion: 0});
        assert_eq!(chess.board[10], piece::PAWN | piece::WHITE);
        assert_eq!(chess.board[18], piece::NONE);
    }
}
