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
            if piece == NONE {
                print!("\x1b[3C");
            } else {
                print!("{}\x1b[2C", piece::look(piece));
            }
            if (square+1) % 8 == 0 {
                print!("\x1b[1A\r\x1b[4C");
            }
        }
        println!("\x1b[10B");
    }


    pub fn update_display(&self, movement: &Move) {
        self.update(movement.start_square);
        self.update(movement.target_square);
        if piece::is_type(self.board[movement.target_square as usize], PAWN)
        || piece::is_type(self.board[movement.start_square as usize], PAWN) {
            if movement.target_square % 8 != movement.start_square % 8 {
                self.update(movement.start_square / 8 * 8 + movement.target_square % 8);
            }
        }
        if piece::is_type(self.board[movement.target_square as usize], KING)
        || piece::is_type(self.board[movement.start_square as usize], KING) {
            let dist = movement.target_square as i8 - movement.start_square as i8;
            if dist == 2 {
                self.update(movement.start_square + 1);
                self.update(movement.start_square + 3);
            }
            else if dist == -2 {
                self.update(movement.start_square - 1);
                self.update(movement.start_square - 4);
            }
        }
    }
    pub fn update(&self, square: u8) {
        if !DISPLAY {return}
        let piece = self.board[square as usize];
        let look = match piece {
            NONE => if (square + square / 8) % 2 == 0 {':'} else {' '}
            _ => piece::look(piece)
        };
        let up = 3 + square / 8;
        let right = 4 + (square % 8) * 3;
        print!("\x1b[{}A\r\x1b[{}C{}", up, right, look);
        println!("\x1b[{}B\r", up-1);
    }
    
}