use super::*;


impl Chess {
    pub fn display(&self) {
        if !DISPLAY {return}
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
        if !DISPLAY {return}
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
                if self.colour_turn == BLACK // jet done
                {self.update_square(target-8)}
                else
                {self.update_square(target+8)}
            }
            Moves::KingCastling => {
                let start = if self.colour_turn == BLACK {4} else {60};
                for square in start..(start+4) {
                    self.update_square(square);
                }
            }
            Moves::QueenCastling => {
                let start = if self.colour_turn == BLACK {0} else {56};
                for square in start..(start+5) {
                    self.update_square(square);
                }
            }
        }
    }
    fn update_square(&self, square: u8) {
        let piece = self.board[square as usize];
        let look = if piece == NONE
        { if (square + square / 8) % 2 == 0 {':'} else {' '} }
        else
        { piece.symbol() };
        let up = 3 + square / 8;
        let right = 4 + (square % 8) * 3;
        print!("\x1b[{}A\r\x1b[{}C{}", up, right, look);
        println!("\x1b[{}B\r", up-1);
    }
    
}