use std::io::{self, Write};
use chess_rust::chess::*;
use chess_rust::random_ai::BadPlayer;

fn play() {
    let mut chess = Chess::new();
    let mut moves = Vec::new();
    chess.display();

    loop {
        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();
        print!("\x1b[A\x1b[J");
        io::stdout().flush().unwrap();
        match cmd.as_str() {
            "deb\n" => println!("{chess:?}"),
            "\n" => {
                let movement = moves.pop().unwrap();
                chess.unmake_move(&movement);
                chess.update_display(&movement);
            }
            movement => {
                let mut chars = movement.chars();
                let start = utils::square_from_text(chars.next().unwrap(), chars.next().unwrap());
                let target = utils::square_from_text(chars.next().unwrap(), chars.next().unwrap());
                let pice = match chars.next() {
                    None | Some('\n') => piece::NONE,
                    Some(symbol) => piece::code(symbol),
                };
                let movement = Move { start_square: start, target_square: target, promotion: pice };
                if chess.generate_legal_moves().contains(&movement) {
                    chess.make_move(&movement);
                    chess.update_display(&movement);
                    moves.push(movement);
                } else {
                    //
                }
            }
        }
        //println!("{:?}", moves);
    }
}

fn main() {
    chess_rust::benchmark();
    //chess_rust::compete(&mut BadPlayer::new(), &mut BadPlayer::new(), 900);
}