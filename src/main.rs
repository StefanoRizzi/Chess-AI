
use std::{process::exit, sync::{atomic::Ordering, mpsc::{self, Receiver, SyncSender}}, time::Duration};
use chess_rust::*;

mod analysis;

fn analyse_db(db_file_name: &str, max_search: i32, time_per_move: Option<f32>) {
    use analysis::get_full_path;

    let db_file_path = get_full_path(&db_file_name);
    let to_file_path;
    let did;
    
    if let Some(time) = time_per_move {
        to_file_path = get_full_path(&(db_file_name.to_string() + ".boss.txt"));
        did = analysis::write_boss_eval(&db_file_path, &to_file_path, max_search, Duration::from_secs_f32(time));
    } else {
        to_file_path = get_full_path(&(db_file_name.to_string() + ".stock.txt"));
        did = analysis::write_stockfish_eval(&db_file_path, &to_file_path, max_search);
    }
    if !did { println!("file {to_file_path:?} already exists")}
}

fn main() {
    // -help
    if std::env::args().find(|arg| ["-h", "--help"].contains(&arg.as_str())).is_some() {
        println!("These are to export the move evaluations of games in a DB:\n\t-analyse <max_games_to_export> #this start exporting\n\t-db <filename.pgn> #the games to export\n\t-boss <sec_per_move> #to make and take boss player evaluations");
        exit(0);
    }

    // -analyse
    let mut args = std::env::args();
    if args.find(|arg| arg == "-analyse").is_some() {
        let mut db_file_name = "lichess_db_standard_rated_2016-01.pgn".to_string();
        let max_search = args.next().expect("expected: -analyse <num_search>").parse::<i32>().unwrap();
        let mut time_per_move = None;
        // -db
        args = std::env::args();
        if args.find(|arg| arg == "-db").is_some() { db_file_name = args.next().expect("expected: -db <filename.pgn>") }
        // -boss
        args = std::env::args();
        if args.find(|arg| arg == "-boss").is_some() { time_per_move = Some(args.next().expect("expected: -boss <sec_per_move>").parse::<f32>().unwrap()) }
        
        analyse_db(&db_file_name, max_search, time_per_move);
        exit(0);
    }

    //uci engine
    legal_moves::precompute();
    
    let mut engine = BossPlayer::new().into_engine_uci();
    let th_stop = engine.stop.clone();
    let (sender, receiver): (SyncSender<String>, Receiver<String>) = mpsc::sync_channel(1);
    
    std::thread::spawn(move || {
        loop {
            let mut message = String::new();
            std::io::stdin().read_line(&mut message).unwrap();

            match message.trim() {
                "stop" => th_stop.store(true, Ordering::Relaxed),
                "quit" => exit(0),
                _ => sender.send(message).unwrap(),
            }
        }
    });
        
    engine.greet();
    loop {
        engine.received_command(receiver.recv().unwrap().trim());
    }
}