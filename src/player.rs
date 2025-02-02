use super::*;

use std::io::Write;
use std::time::Duration;
use std::fs::File;
use std::sync::{Arc, atomic::AtomicBool};

pub mod rizzi_the_boss;
pub mod random_ai;
pub mod human;

pub use rizzi_the_boss::BossPlayer;
pub use random_ai::BadPlayer;
pub use human::HumanPlayer;

pub type Eval = i16;

pub trait ChessPlayer {
    
    fn name(&self) -> &str;
    fn into_engine_uci(self) -> EngineUCI<Self> where Self: Sized {
        EngineUCI::new(self)
    }

    fn notify_new_game(&self);
    fn set_position(&mut self, chess: &Chess);
    fn best_move(&mut self, chess: &mut Chess, time: Option<Duration>) -> (Move, Eval);
    fn make_move(&mut self, r#move: Move);
    fn evaluate_infinite(&mut self, chess: &mut Chess, send_info: fn(depth: u16, eval: Eval, time: u64, nodes: u32, nps: u32, pv: Move));
    fn get_stop(&self) -> Arc<AtomicBool> {Arc::new(AtomicBool::new(false))}
}

pub struct EngineUCI<PLAYER: ChessPlayer> {
    chess: Chess,
    player: PLAYER,
    pub stop: Arc<AtomicBool>,
    log: File,
}

impl<PLAYER: ChessPlayer> EngineUCI<PLAYER> {
    pub fn new(player: PLAYER) -> Self {
        let stop = player.get_stop();
        let path = ROOT_PATH.lock().unwrap().as_ref().unwrap().join("uci_log.txt");
        EngineUCI {
            chess: Chess::new(),
            player: player,
            stop,
            log:
            File::create(path).unwrap()
        }
    }

    pub fn greet(&mut self) {
        self.respond(&format!("Hi~ I'm {}!", self.player.name()));
    }

    pub fn received_command(&mut self, message: &str) {
        self.log.write_all("Scid: ".as_bytes()).unwrap();
        self.log.write_all(message.as_bytes()).unwrap();
        self.log.write_all("\n".as_bytes()).unwrap();

        let (msg, arg) = message.split_once(' ').unwrap_or((message, ""));
        match msg {
            "uci" => {
                self.respond(&format!("id name {}", self.player.name()));
                self.respond("id author Stefano R");
                self.respond("uciok")
            },
            "isready" => self.respond("readyok"),
            "ucinewgame" => self.process_new_game_command(),
            "position" => self.process_position_command(arg),
            "go" => self.process_go_command(arg),
            "stop" => (),
            "quit" => (),
            _ => println!("Huh? {msg}"),
        }
    }

    pub fn respond(&mut self, message: &str) {
        println!("{message}");
        self.log.write_all("Engine: ".as_bytes()).unwrap();
        self.log.write_all(message.as_bytes()).unwrap();
        self.log.write_all("\n".as_bytes()).unwrap();
    }

    fn process_new_game_command(&mut self) {
        self.chess.irreversable_state.clear();
        self.player.notify_new_game();
    }

    fn process_position_command(&mut self, message: &str) {
        let (msg, arg) = message.split_once(' ').unwrap_or((message, ""));
        let (fen, moves) = arg.split_once("moves").unwrap_or((arg, ""));
        let moves = moves.split_whitespace();
        match msg {
            "startpos" => {
                self.chess = Chess::start_position().into();
                self.player.set_position(&self.chess);
            },
            "fen" => {
                let mut new_chess = Chess::build(fen);
                std::mem::swap(&mut new_chess.irreversable_state, &mut self.chess.irreversable_state);
                new_chess.irreversable_state.push((NONE_TYPE, self.chess.en_passant, self.chess.castling, self.chess.half_move, self.chess.hash()));
                self.chess = new_chess;
                self.player.set_position(&self.chess);
            }
            _ => unreachable!()
        }

        for r#move in moves {
            let r#move = Move::from_text(
                &self.chess,
                r#move,
            );
            self.chess.make_move(r#move);
            self.player.make_move(r#move);
        }
    }

    fn process_go_command(&mut self, message: &str) {
        let mut msgs = message.split_whitespace();
        let mut move_time = 1_000;

        match msgs.next() {
            Some("infinite") => {
                fn send_depth_score (depth: u16, eval: Eval, time: u64, nodes: u32, nps: u32, pv: Move) {
                    //self.respond(&format!("info depth {depth} score cp {eval}"));
                    println!("info depth {depth} score cp {eval} time {time} nodes {nodes} nps {nps} pv {}", pv.to_text());
                }

                self.player.evaluate_infinite(&mut self.chess, send_depth_score);
                
                return;
            }
            Some("wtime") => {
                let (w_time, b_time) = (
                    {
                        msgs.next().unwrap().parse::<u64>().unwrap()
                    },
                    {
                        assert_eq!(msgs.next(), Some("btime"));
                        msgs.next().unwrap().parse::<u64>().unwrap()
                    },
                );
                move_time = if self.chess.is_white_to_move() { w_time } else { b_time };
            }
            Some("movetime") => move_time = msgs.next().unwrap().parse::<u64>().unwrap(),
            _ => (),
        }
        
        let (best_move, _) = self.player.best_move(&mut self.chess, Some(Duration::from_millis(move_time)));

        self.respond(&format!("bestmove {}", best_move.to_text()));
                    
        self.chess.make_move(best_move);
        self.player.make_move(best_move);
    }
    
}