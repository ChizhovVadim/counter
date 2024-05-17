use crate::types;
use crate::chess;
use std::fmt::Write;
use crate::uci;
use crate::uci::Game;
use std::time::Duration;

pub fn print_uci_search_info(si: &types::SearchInfo) {
    let mut sb = String::new();
    write!(sb, "info depth {}", si.depth);
    match si.score {
        types::UciScore::Centipawns(x) => {
            write!(sb, " score cp {}", x);
        }
        types::UciScore::Mate(x) => {
            write!(sb, " score mate {}", x);
        }
    }
    let nps = si.nodes as f32 / si.duration.as_secs_f32();
    write!(
        sb,
        " nodes {} time {} nps {}",
        si.nodes,
        si.duration.as_millis(),
        nps as i32
    );
    if si.main_line.len() > 0 {
        sb.push_str(" pv");
        for m in si.main_line.iter() {
            write!(sb, " {}", m.name());
        }
    }
    println!("{}", sb);
}


impl Game {
    pub fn new(fen: &str) -> Option<Game> {
        return Some(Game {
            position: chess::Position::from_fen(fen)?,
            repeats: Vec::new(),
        });
    }
    pub fn make_move(&mut self, lan: &str) -> bool {
        let mv = chess::Move::parse_lan(&self.position, lan);
        if mv.is_none() {
            return false;
        }
        let mv = mv.unwrap();
        let mut child = self.position;
        let mut history = chess::History::new();
        if !child.make_move(mv, &mut history) {
            return false;
        }
        if mv.moving_piece() == chess::PIECE_PAWN || mv.captured_piece() != chess::PIECE_EMPTY {
            self.repeats.clear();
        } else {
            self.repeats.push(self.position.key);
        }
        self.position = child;
        return true;
    }
}

pub fn parse_game(fields: Vec<&str>) -> Option<Game> {
    let moves_index = fields.iter().position(|&token| token == "moves");
    let mut game: Game = if fields[0] == "startpos" {
        Game::new(chess::INITIAL_POSITION_FEN)?
    } else if fields[0] == "fen" {
        let end = moves_index.unwrap_or(fields.len());
        let mut fen = String::new();
        for &word in fields[1..end].iter() {
            fen.push_str(word);
            fen.push_str(" ");
        }
        Game::new(fen.as_str())?
    } else {
        return None;
    };
    if moves_index.is_none() {
        return Some(game);
    }
    for &smove in fields[moves_index.unwrap() + 1..].iter() {
        if !game.make_move(smove) {
            return None;
        }
    }
    return Some(game);
}

pub fn parse_limits(split: &mut core::str::SplitAsciiWhitespace) -> Option<uci::LimitsType> {
    let mut result: Option<uci::LimitsType> = None;
    while let Some(option) = split.next() {
        match option {
            "movetime" => {
                let millis = split.next().unwrap().parse().unwrap();
                result = Some(uci::LimitsType::FixedTime(Duration::from_millis(millis)));
            }
            "depth" => {
                let depth: u32 = split.next().unwrap().parse().unwrap();
                result = Some(uci::LimitsType::FixedDepth(depth as usize));
            }
            "nodes" => {
                let nodes = split.next().unwrap().parse().unwrap();
                result = Some(uci::LimitsType::FixedNodes(nodes));
            }
            "infinite" => {
                result = Some(uci::LimitsType::Infinite);
            }
            _ => (),
        }
    }
    return result;
}