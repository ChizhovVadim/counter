mod game;
mod parse;

use crate::domain::{
    CancelToken, IEngine, LimitsType, OptionValue, SearchInfo, SearchParams, UciScore,
};
use game::Game;
use parse::UciMessage;
use std::fmt;

enum EngineMessage {
    Uci,
    SetOption { name: String, value: String },
    IsReady,
    Go(Game, LimitsType, CancelToken),
    NewGame,
    Quit,
}

pub fn run(eng: &mut dyn IEngine) {
    std::thread::scope(|scope| {
        let (sender, receiver) = std::sync::mpsc::channel();
        scope.spawn(|| {
            let _ = cli_commands_cycle(sender);
        });
        engine_commands_cycle(eng, receiver);
    })
}

fn cli_commands_cycle(
    sender: std::sync::mpsc::Sender<EngineMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cancel = CancelToken::new();
    let mut game = Game::new();
    let mut buffer = String::new();
    loop {
        buffer.clear();
        std::io::stdin().read_line(&mut buffer)?;
        let user_cmd = parse::parse_command(buffer.trim_end());
        match user_cmd {
            Err(msg) => {
                eprintln!("{}", msg);
            }
            Ok(UciMessage::Uci) => {
                sender.send(EngineMessage::Uci)?;
            }
            Ok(UciMessage::SetOption { name, value }) => {
                sender.send(EngineMessage::SetOption {
                    name: name,
                    value: value,
                })?;
            }
            Ok(UciMessage::IsReady) => {
                sender.send(EngineMessage::IsReady)?;
            }
            Ok(UciMessage::NewGame) => {
                game = Game::new();
                sender.send(EngineMessage::NewGame)?;
            }
            Ok(UciMessage::Stop) => {
                cancel.cancel();
            }
            Ok(UciMessage::Quit) => {
                cancel.cancel();
                sender.send(EngineMessage::Quit)?;
                return Ok(());
            }
            Ok(UciMessage::Position(g)) => {
                game = g;
            }
            Ok(UciMessage::Go(limits)) => {
                cancel = CancelToken::new();
                sender.send(EngineMessage::Go(game.clone(), limits, cancel.clone()))?;
            }
        }
    }
}

fn engine_commands_cycle(
    eng: &mut dyn IEngine,
    receiver: std::sync::mpsc::Receiver<EngineMessage>,
) {
    let name = "Counter";
    let version = "rust 0.1";
    let author = "Vadim Chizhov";

    for received in receiver {
        match received {
            EngineMessage::Uci => {
                println!("id name {} {}", name, version);
                println!("id author {}", author);
                for opt in eng.get_options() {
                    match opt.value {
                        OptionValue::Bool(val) => {
                            println!("option name {} type check default {}", opt.name, val);
                        }
                        OptionValue::Int { min, max, value } => {
                            println!(
                                "option name {} type spin default {} min {} max {}",
                                opt.name, value, min, max
                            );
                        }
                        OptionValue::String(val) => {
                            let val = if val.is_empty() {
                                String::from("<empty>")
                            } else {
                                val
                            };
                            println!("option name {} type string default {}", opt.name, val);
                        }
                    }
                }
                println!("uciok");
            }
            EngineMessage::SetOption { name, value } => {
                eng.set_option(&name, &value);
            }
            EngineMessage::IsReady => {
                println!("readyok");
            }
            EngineMessage::NewGame => {
                eng.clear();
            }
            EngineMessage::Go(game, limits, cancel) => {
                let repeats = game.two_time_repeats();
                let search_result = eng.search(SearchParams {
                    position: game.position,
                    repeats: repeats,
                    limits: limits,
                    cancel: cancel,
                    progress: Box::new(uci_search_progress()),
                });
                println!("{}", &search_result);
                if !search_result.main_line.is_empty() {
                    println!("bestmove {:?}", search_result.main_line[0]);
                }
            }
            EngineMessage::Quit => {
                return;
            }
        }
    }
}

impl fmt::Display for SearchInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "info")?;
        write!(f, " depth {}", self.depth)?;
        match self.score {
            UciScore::Centipawns(x) => {
                write!(f, " score cp {}", x)?;
            }
            UciScore::Mate(x) => {
                write!(f, " score mate {}", x)?;
            }
        }
        write!(f, " nodes {}", self.nodes)?;
        write!(f, " time {}", self.duration.as_millis())?;
        let nps = (self.nodes as f32 / self.duration.as_secs_f32()) as i32;
        write!(f, " nps {}", nps)?;
        if !self.main_line.is_empty() {
            write!(f, " pv")?;
            for m in self.main_line.iter() {
                write!(f, " {:?}", m)?;
            }
        }
        Ok(())
    }
}

fn uci_search_progress() -> impl Fn(&SearchInfo) {
    const MIN_DURATION: std::time::Duration = std::time::Duration::from_millis(500);
    move |si| {
        if si.duration >= MIN_DURATION {
            println!("{}", si);
        }
    }
}
