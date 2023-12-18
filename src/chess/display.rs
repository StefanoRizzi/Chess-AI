use crate::chess::utils::*;

use super::*;


impl Chess {
    pub fn display(&self) {
        if unsafe {!DISPLAY} {return}
        print!(r#"
  /========================\
8 |   :::   :::   :::   :::|
7 |:::   :::   :::   :::   |
6 |   :::   :::   :::   :::|
5 |:::   :::   :::   :::   |
4 |   :::   :::   :::   :::|
3 |:::   :::   :::   :::   |
2 |   :::   :::   :::   :::|
1 |:::   :::   :::   :::   |
  \========================/
    a  b  c  d  e  f  g  h
"#);
        print!("\x1b[3A\x1b[4C");
        for (square, &piece) in self.board.iter().enumerate() {
            if piece == NONE
            {print!("\x1b[3C")}
            else
            {print!("{}\x1b[2C", piece.symbol())}
            if (square+1) % 8 == 0
            {print!("\x1b[1A\r\x1b[4C")}
        }
        println!("\x1b[10B");
    }


    pub fn update_display(&self, movee: Moves) {
        if unsafe {!DISPLAY} {return}
        match movee {
            | Moves::Move { start, target }
            | Moves::DoublePush { start, target }
            | Moves::Promotion { start, target, .. } => {
                self.update_square(start);
                self.update_square(target);
            }
            Moves::EnPassant { start, target } => {
                self.update_square(start);
                self.update_square(target);
                if !self.is_white_to_move // previous turn color
                {self.update_square(target-8)}
                else
                {self.update_square(target+8)}
            }
            Moves::KingCastling => {
                let start = if !self.is_white_to_move {4} else {60};
                for square in start..(start+4) {
                    self.update_square(square);
                }
            }
            Moves::QueenCastling => {
                let start = if !self.is_white_to_move {0} else {56};
                for square in start..(start+5) {
                    self.update_square(square);
                }
            }
        }
    }
    fn update_square(&self, square: u8) {
        let piece = self.board[square as usize];
        let look = if piece == NONE
        { if is_black_square(square) {':'} else {' '} }
        else
        { piece.symbol() };
        let up = 3 + square / 8;
        let right = 4 + (square % 8) * 3;
        print!("\x1b[{}A\r\x1b[{}C{}", up, right, look);
        println!("\x1b[{}B\r", up-1);
    }
    pub fn display_attacks(&self, colour: Colour) {
        Chess::display_attacks_pieces(&self.side[colour.colour_index()].attacks);
    }
    pub fn display_attacks_pieces(attacks: &[i8; 64]) {
        println!(r#"  /========================\"#);
        for j in (0..8).rev() {
            print!("{j} |");
            for i in 0..8 {
                let square = i+j*8;
                if is_black_square(square) {
                    print!(":{}:", attacks[square as usize]);
                } else {
                    print!(" {} ", attacks[square as usize]);
                }
            }
            println!("|");
        }
        println!(r#"  \========================/"#);
    }
}