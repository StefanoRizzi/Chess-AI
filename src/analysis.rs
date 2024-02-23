
use chess_rust::*;

use std::fs;
pub use std::io::{self, Write, prelude::*, BufReader};
pub use std::fs::File;
pub use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

struct TextMove { text: String, eval: Option<String> }
struct Game { id: i32, welo: i32, belo: i32, moves: Vec<TextMove> }

pub fn get_full_path(file_name: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    return PathBuf::from_str(&home).unwrap().join("Scaricati").join(file_name);
}

// #return: true se ha creato il file; false se già esistente
pub fn write_stockfish_eval(file_path: &PathBuf, to_file_path: &PathBuf, max_search: i32) -> bool {
    if fs::metadata(&to_file_path).is_ok() { return false }
    
    let games = read_games(file_path, true, max_search);
    save_games_evals(games, to_file_path);

    return true;
}

pub fn write_boss_eval(file_path: &PathBuf, to_file_path: &PathBuf, max_search: i32, time_per_move: Duration) -> bool {
    if fs::metadata(to_file_path).is_ok() { return false }
    let file_db = fs::File::open(file_path).unwrap();
    let mut lines: io::Lines<BufReader<File>> = BufReader::new(file_db).lines();
    let mut f_out = File::create(to_file_path).unwrap();
    writeln!(f_out, "-1").unwrap();

    let mut count = 0;
    let mut found = 0;
    loop {
        if found >= max_search { break }
        println!("reading games: {found} found {count} done");
        let res = read_game(&mut lines, count, true);
        //if count < 1218 { count += 1; continue }
        match res {
            None => break,
            Some(Some(mut game)) => {
                let mut chess = Chess::start_position();
                let mut boss = BossPlayer::new();
                boss.print_info = false;
                chess.display();
                for r#move in game.moves.iter_mut() {
                    let mv: Move = parse_move(&chess, r#move);
                    chess.make_move(mv);
                    boss.make_move(mv);
                    chess.update_display(mv);
                    let (_, eval) = boss.best_move(&mut chess, Some(time_per_move));
                    let eval = if chess.colour_to_move() == WHITE { eval } else { -eval };
                    let eval = (eval as f32 / 100.0).to_string();
                    print!("{} ", eval);
                    r#move.eval = Some(eval);
                }
                save_game_evals(game, &mut f_out);
                found += 1;
                println!()
            }
            Some(None) => (),
        }
        count += 1;
    }

    return true;
}

fn read_games(file_path: &PathBuf, check_eval: bool, max_search: i32) -> Vec<Game> {
    let file_db = fs::File::open(file_path).unwrap();
    let mut lines: io::Lines<BufReader<File>> = BufReader::new(file_db).lines();

    let mut games = Vec::new();
    let mut count = 0;
    let mut found = 0;
    loop {
        if found >= max_search { break }
        print!("\r\x1b[Jreading games: {found} found {count} done");
        io::stdout().flush().unwrap();

        let res: Option<Option<Game>> = read_game(&mut lines, count, check_eval);
        match res {
            None => break,
            Some(Some(game)) => {
                games.push(game);
                found += 1;
            }
            _ => ()
        }
        count += 1;
    }

    println!("\r\x1b[Jfound {} on {count} games", games.len());    
    return games
}

fn save_games_evals(games: Vec<Game>, to_file_path: &PathBuf) {
    println!("saving to {to_file_path:?}");

    let mut f_out = File::create(to_file_path).unwrap();
    writeln!(f_out, "{}", games.len()).unwrap();
    
    let len = games.len();
    for (count, game) in games.into_iter().enumerate() {
        print!("\r\x1b[J{count}/{}", len);
        io::stdout().flush().unwrap();

        save_game_evals(game, &mut f_out);
    } 
    println!("\r\x1b[JDone");
}

fn save_game_evals(game: Game, file_storage: &mut File) {
    let f_out = file_storage;
    writeln!(f_out, "id {} welo {} belo {}", game.id, game.welo, game.belo).unwrap();
    
    let mut last_eval = None;
    for r#move in game.moves {
        let eval = r#move.eval.unwrap_or_else(|| last_eval.unwrap());
        write!(f_out, "{eval} ").unwrap();
        last_eval = Some(eval);
    }
    write!(f_out, "\n").unwrap();
}

fn read_game(lines: &mut io::Lines<BufReader<File>>, game_id: i32, check_eval: bool) -> Option<Option<Game>> {
    
    fn read_game_notes(lines: &mut io::Lines<BufReader<File>>) -> Option<Option<(i32, i32)>> {
        let mut welo = None;
        let mut belo = None;
        let mut note;
        loop {
            let line = lines.next();
            if line.is_none() { return None }
            note = line.unwrap().unwrap();
            if note.starts_with("[") { break }
        }

        while note.starts_with("[") {
            let (name, value) = note[1..].split_once(" ").unwrap();
            match name {
                "WhiteElo" => {
                    let elo = value[1..(value.len() - 2)].parse::<i32>();
                    if let Ok(elo) = elo { welo = Some(elo) }
                }
                "BlackElo" => {
                    let elo = value[1..(value.len() - 2)].parse::<i32>();
                    if let Ok(elo) = elo { belo = Some(elo) }
                }
                _ => ()
            }
            note = lines.next().unwrap().unwrap();
        }
        return Some(welo.zip(belo));
    }
    fn read_move_note_eval(notes: &str) -> Option<String> {
        let eval = notes.split_once("[%eval ").map(|res| res.1.split_once("]").unwrap().0.to_string());
        return eval;
    }
    let notes = read_game_notes(lines);
    if notes.is_none() { return None }

    let tmp_str = lines.next().unwrap().unwrap();
    let mut str_moves = tmp_str.as_str().trim();
    let mut moves = Vec::new();
    
    if check_eval {
        if notes.unwrap().is_none() { return Some(None) }
        if !str_moves.split(" ").skip(2).next().map_or(false, |val| val.starts_with("{")) { return Some(None) }
    }

    loop {
        let white_turn;
        let black_turn;
        let white_move_text;
        let mut black_move_text;
        (white_turn, str_moves) = str_moves.split_once(" ").unwrap_or_default();
        if str_moves.is_empty() { break }
        
        (white_move_text, str_moves) = str_moves.split_once(" ").unwrap();
        (black_move_text, str_moves) = str_moves.split_once(" ").unwrap_or_default();
        
        let mut white_move_eval = None;
        let mut black_move_eval = None;
        if black_move_text == "{" {
            let str_notes;
            (str_notes, str_moves) = str_moves.split_once("} ").unwrap();
            white_move_eval = read_move_note_eval(str_notes);
            
            (black_turn, str_moves) = str_moves.split_once(" ").unwrap_or_default();
            if !str_moves.is_empty() {
                (black_move_text, str_moves) = str_moves.split_once(" ").unwrap()
            }
        }
        if str_moves.starts_with("{") {
            let str_notes;
            (str_notes, str_moves) = str_moves.split_once("} ").unwrap();
            black_move_eval = read_move_note_eval(str_notes);
        }
        moves.push(TextMove { text: white_move_text.to_string(), eval: white_move_eval });
        if str_moves.is_empty() { break }
        moves.push(TextMove { text: black_move_text.to_string(), eval: black_move_eval });
    }
    
    let notes = notes.unwrap().unwrap();
    return Some(Some(Game { id: game_id, welo: notes.0, belo: notes.1, moves }))
}

fn parse_move(chess: &Chess, r#move: &TextMove) -> Move {
    let text = r#move.text.as_str();
    let text = text.split_once(|ch| ['#', '+', '?', '!'].contains(&ch)).unwrap_or((text, "")).0;
    
    let gen_moves = chess.generate_legal_moves();
    //castling
    if text == "O-O" {
        return gen_moves.into_iter().find(|mv| mv.flag() == CASTLE_FLAG && mv.is_king_castling()).unwrap()
    }
    else if text == "O-O-O" {
        return gen_moves.into_iter().find(|mv| mv.flag() == CASTLE_FLAG && !mv.is_king_castling()).unwrap()
    }
    //promotion
    let split = text.split_once("=");
    let promotion = split.map(|p| PieceType::from_symbol(p.1.chars().next().unwrap()));
    let text = split.map_or(text, |val| val.0);
    let mut text_len = text.len();

    let mut chars = text.chars();
    let mut ch = chars.next().unwrap();
    //pice type
    let piece_type;
    if is_piece_type_symbol(ch) && ch.is_ascii_uppercase() {
        piece_type = PieceType::from_symbol(ch);
        ch = chars.next().unwrap();
        text_len -= 1;
    }
    else { piece_type = PAWN }
    //start
    let mut letter_start: Option<char> = None;
    let mut number_start: Option<char> = None;
    while text_len > 2 {
        if ch == 'x' { /*capture*/ }
        else if ch.is_ascii_digit() { number_start = Some(ch) }
        else { letter_start = Some(ch) }
        ch = chars.next().unwrap();
        text_len -= 1;
    }
    //target
    let target = square_from_text(ch, chars.next().unwrap());

    //println!("{} {} {} {:?}", text, square_to_text(target), piece_type.symbol(), letter_start);
    //find move
    for mv in gen_moves {
        //piece type filter 
        if chess.board(mv.start()).get_type() != piece_type { continue }
        //promotion filter
        if let Some(pt) = promotion {
            if !mv.is_promotion() { continue }
            if mv.promotion_type() != pt { continue }
        }
        //start filter
        let start = square_to_text(mv.start());
        let mut st_tx = start.chars();
        let (st_le, st_nu) = (st_tx.next().unwrap(), st_tx.next().unwrap());
        if letter_start.map_or(false, |le| le != st_le) { continue }
        if number_start.map_or(false, |nu| nu != st_nu) { continue }
        //target filter
        if mv.target() != target { continue }

        return mv 
    }
    //chess.display();
    //println!("{text}");
    unreachable!()
}

fn is_piece_type_symbol(symbol: char) -> bool {
    match symbol.to_ascii_lowercase() {
        'p' => true, 'n' => true, 'b' => true,
        'r' => true, 'q' => true, 'k' => true, _ => false,
    }
}