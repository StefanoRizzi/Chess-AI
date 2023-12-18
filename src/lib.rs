pub mod chess;
pub mod perft;
pub mod random_ai;
pub mod rizzi_the_boss;
use chess::*;

use rand::Rng;

pub trait ChessPlayer {
    fn best_move(&mut self, chess: &mut Chess) -> Moves;
}

pub fn benchmark() {
    let mut chess = Chess::new();
    chess.time_perft(5);
}

pub fn compete(player_1: &mut dyn ChessPlayer, player_2: &mut dyn ChessPlayer, games: u32) {
    let (mut won, mut lost, mut draw) = (0, 0, 0);
    for _ in 0..games {
        let mut chess = Chess::build("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        play(&mut chess, player_1, player_2);
        if chess.is_king_in_check() {
            if chess.is_white_to_move
            {lost += 1}
            else
            {won += 1}
        } 
        else
        {draw += 1} 
    }
    println!("won: {won} draw: {draw} lost: {lost}");
}

pub fn play(mut chess: &mut Chess, player_1: &mut dyn ChessPlayer, player_2: &mut dyn ChessPlayer) {
    chess.display();
    loop {
        if chess.generate_legal_moves().len() == 0 {break}
        let movee = player_1.best_move(&mut chess);
        chess.make_move(movee);
        chess.display();
        for j in (0..8).rev() {
            for i in 0..8 {
                print!("  {}", chess.side[0].attacks[i+j*8]);
            }
            println!()
        }
        
        if chess.generate_legal_moves().len() == 0 {break}
        let movee = player_2.best_move(&mut chess);
        chess.make_move(movee);
        chess.display();
        for j in (0..8).rev() {
            for i in 0..8 {
                print!("  {}", chess.side[0].attacks[i+j*8]);
            }
            println!()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chess_new() {
        let chess = chess::Chess::new();
        assert_eq!(chess.board[3], Piece::new(QUEEN, WHITE));
        assert_eq!(chess.board[4], Piece::new(KING, WHITE));
    }
    #[test]
    fn chess_move() {
        let mut chess = chess::Chess::new();
        chess.make_move(Moves::Move { start: 10, target: 18 });
        assert_eq!(chess.board[10], NONE);
        assert_eq!(chess.board[18], Piece::new(PAWN, WHITE));
        chess.unmake_move(Moves::Move { start: 10, target: 18 });
        assert_eq!(chess.board[10], Piece::new(PAWN, WHITE));
        assert_eq!(chess.board[18], NONE);
    }
}
