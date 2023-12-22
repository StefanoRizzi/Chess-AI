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


    pub fn update_display(&self, r#move: Move) {
        if unsafe {!DISPLAY} {return}
        let (start, target) = (r#move.start(), r#move.target());
        self.update_square(start);
        self.update_square(target);
        match r#move.flag() {
            EN_PASSANT_FLAG => {
                if !self.is_white_to_move // previous turn color
                {self.update_square(target-8)}
                else
                {self.update_square(target+8)}
            }
            CASTLE_FLAG => {
                let range = if r#move.is_king_castling()
                {start..start+4} else {start-4..start+1};
                for square in range {
                    self.update_square(square);
                }
            }
            _ => ()
        }
    }
    fn update_square(&self, square: Square) {
        let piece = self.board[square as usize];
        let look = if piece == NONE
        { if is_black_square(square) {':'} else {' '} }
        else
        { piece.symbol() };
        let up = 3 + square / 8;
        let right = 4 + (square % 8) * 3;
        println!("\x1b[{}A\r\x1b[{}C{}\x1b[{}B\r", up, right, look, up-1);
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