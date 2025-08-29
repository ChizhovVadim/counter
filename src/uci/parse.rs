use crate::domain::LimitsType;
use crate::uci::game::{Game, parse_game};
use std::time::Duration;

pub enum UciMessage {
    Uci,
    SetOption { name: String, value: String },
    IsReady,
    Position(Game),
    Go(LimitsType),
    NewGame,
    Stop,
    Quit,
}

pub fn parse_command(s: &str) -> Result<UciMessage, String> {
    let mut split = s.split_ascii_whitespace();
    let cmd_name = split.next().ok_or("empty command")?;
    match cmd_name {
        "uci" => {
            return Ok(UciMessage::Uci);
        }
        "isready" => {
            return Ok(UciMessage::IsReady);
        }
        "ucinewgame" => {
            return Ok(UciMessage::NewGame);
        }
        "stop" => {
            return Ok(UciMessage::Stop);
        }
        "quit" => {
            return Ok(UciMessage::Quit);
        }
        "setoption" => {
            let (name, value) = parse_option(&mut split).ok_or("parse_option failed")?;
            return Ok(UciMessage::SetOption {
                name: name,
                value: value,
            });
        }
        "position" => {
            let game = parse_game(&mut split)?;
            return Ok(UciMessage::Position(game));
        }
        "go" => {
            let limits = parse_limits(&mut split).ok_or("parse_limits failed")?;
            return Ok(UciMessage::Go(limits));
        }
        _ => {
            return Err("unknown command".into());
        }
    }
}

fn parse_option(split: &mut std::str::SplitAsciiWhitespace) -> Option<(String, String)> {
    split.next()?;
    let name = split.next()?;
    split.next()?;
    let value = split.next()?;
    return Some((name.into(), value.into()));
}

fn parse_limits(split: &mut std::str::SplitAsciiWhitespace) -> Option<LimitsType> {
    let mut result = LimitsType::default();
    while let Some(option) = split.next() {
        match option {
            "movetime" => {
                let millis: u64 = split.next()?.parse().ok()?;
                result.fixed_time = Some(Duration::from_millis(millis));
            }
            "depth" => {
                let depth = split.next()?.parse().ok()?;
                result.fixed_depth = Some(depth);
            }
            "nodes" => {
                let nodes = split.next()?.parse().ok()?;
                result.fixed_nodes = Some(nodes);
            }
            "infinite" => {
                result.infinite = true;
            }
            "wtime" => {
                let millis: u64 = split.next()?.parse().ok()?;
                result.tournament.white_time = Some(millis);
            }
            "btime" => {
                let millis: u64 = split.next()?.parse().ok()?;
                result.tournament.black_time = Some(millis);
            }
            "winc" => {
                let millis: u64 = split.next()?.parse().ok()?;
                result.tournament.white_increment = millis;
            }
            "binc" => {
                let millis: u64 = split.next()?.parse().ok()?;
                result.tournament.black_increment = millis;
            }
            "movestogo" => {
                let moves: u32 = split.next()?.parse().ok()?;
                result.tournament.moves = Some(moves);
            }
            _ => (),
        }
    }
    return Some(result);
}
