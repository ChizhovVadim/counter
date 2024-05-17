use crate::chess;
use crate::types;
pub use parse::print_uci_search_info;
use std::sync;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
pub use timemanager::TimeManager;

mod parse;
mod timemanager;

#[derive(Debug)]
pub enum LimitsType {
    Infinite,
    FixedNodes(u64),
    FixedTime(Duration),
    FixedDepth(usize),
    Tournament {
        time: Duration,
        inc: Option<Duration>,
        moves: Option<usize>,
    },
}

enum Message {
    UciGreeting,
    UciSetOption {
        name: String,
        value: String,
    },
    UciIsReady,
    UciGo {
        pos: chess::Position,
        repeats: Vec<u64>,
        limits: LimitsType,
        abort: sync::Arc<sync::atomic::AtomicBool>,
    },
    UciNewGame,
    UciQuit,
}

#[derive(Debug, Clone)]
struct Game {
    position: chess::Position,
    repeats: Vec<u64>,
}

pub fn run(engine: Box<dyn types::IEngine>) {
    thread::scope(|scope| {
        let (tx, tr) = mpsc::channel();
        scope.spawn(|| {
            read_uci_commands(tx);
        });
        handle_uci_commands(engine, tr);
    });
}

fn handle_uci_commands(mut engine: Box<dyn types::IEngine>, tr: mpsc::Receiver<Message>) {
    for received in tr {
        match received {
            Message::UciGreeting => {
                println!("id name {} {}", "Counter", "rust 0.1");
                println!("id author {}", "Vadim Chizhov");
                for opt in engine.get_options() {
                    match opt.value {
                        types::OptionValue::Bool(val) => {
                            println!("option name {} type check default {}", opt.name, val)
                        }
                        types::OptionValue::Int { min, max, value } => {
                            println!(
                                "option name {} type spin default {} min {} max {}",
                                opt.name, value, min, max
                            );
                        }
                    }
                }
                println!("uciok");
            }
            Message::UciSetOption { name, value } => {
                engine.set_option(&name, &value);
            }
            Message::UciIsReady => println!("readyok"),
            Message::UciGo {
                pos,
                repeats,
                limits,
                abort,
            } => {
                let res = engine.search(types::SearchParams {
                    position: pos,
                    repeats: repeats,
                    time_manager: Box::new(TimeManager::new(limits, abort)),
                });
                parse::print_uci_search_info(&res);
                if res.main_line.len() > 0 {
                    println!("bestmove {}", res.main_line[0]);
                }
            }
            Message::UciNewGame => engine.clear(),
            Message::UciQuit => return,
        }
    }
}

fn read_uci_commands(tx: mpsc::Sender<Message>) {
    let mut game = Game::new(chess::INITIAL_POSITION_FEN);
    let mut abort = sync::Arc::new(sync::atomic::AtomicBool::new(false));
    loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        if buffer.is_empty() {
            continue;
        }
        let mut split = buffer.split_ascii_whitespace();
        let token = match split.next() {
            None => continue,
            Some(token) => token,
        };
        match token {
            "uci" => {
                tx.send(Message::UciGreeting);
            }
            "setoption" => {
                split.next();
                let name = split.next().unwrap();
                split.next();
                let value = split.next().unwrap();
                tx.send(Message::UciSetOption {
                    name: String::from(name),
                    value: String::from(value),
                });
            }
            "isready" => {
                tx.send(Message::UciIsReady);
            }
            "ucinewgame" => {
                tx.send(Message::UciNewGame);
            }
            "position" => {
                game = parse::parse_game(split.collect());
            }
            "go" => {
                let limits = parse::parse_limits(&mut split);
                abort = sync::Arc::new(sync::atomic::AtomicBool::new(false));
                //TODO handle errors
                tx.send(Message::UciGo {
                    pos: game.as_ref().unwrap().position,
                    repeats: game.as_ref().unwrap().repeats.clone(),
                    limits: limits.unwrap(),
                    abort: abort.clone(),
                });
            }
            "stop" => {
                abort.store(true, atomic::Ordering::SeqCst);
            }
            "quit" => {
                abort.store(true, atomic::Ordering::SeqCst);
                tx.send(Message::UciQuit);
                return;
            }
            _ => eprintln!("command not found"),
        }
    }
}
