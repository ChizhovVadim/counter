use crate::chess::{Move, Piece, Position};

#[derive(Clone)]
pub struct Game {
    pub position: Position,
    pub repeats: Vec<u64>, // НЕ включая position.key
}

impl Game {
    pub fn new() -> Self {
        return Game {
            position: Position::from_fen(Position::INITIAL_POSITION_FEN).unwrap(),
            repeats: Vec::new(),
        };
    }

    pub fn make_move(&mut self, lan: &str) -> Option<()> {
        let mv = Move::parse_lan(&self.position, lan)?;
        let mut child: Position = unsafe { std::mem::zeroed() };
        if !self.position.make_move(mv, &mut child) {
            return None;
        }
        if mv.moving_piece() == Piece::PAWN || mv.captured_piece() != Piece::NONE {
            self.repeats.clear();
        } else {
            self.repeats.push(self.position.key);
        }
        self.position = child;
        return Some(());
    }

    pub fn two_time_repeats(&self) -> Vec<u64> {
        let mut m = std::collections::HashMap::new();
        for key in self.repeats.iter() {
            if let Some(x) = m.get_mut(key) {
                *x += 1;
            } else {
                m.insert(*key, 1);
            }
        }
        let result: Vec<_> = m
            .iter()
            .filter(|&(_, &v)| v >= 2)
            .map(|(&k, _)| k)
            .collect();
        return result;
    }
}

pub fn parse_game(split: &mut std::str::SplitAsciiWhitespace) -> Result<Game, String> {
    let mut state = 0;
    let mut init_fen = String::new();
    let mut parse_moves = false;
    for token in split.by_ref() {
        if state == 0 {
            if token == "startpos" {
                state = 2;
                init_fen = String::from(Position::INITIAL_POSITION_FEN);
            } else if token == "fen" {
                state = 1;
                init_fen = String::new();
            } else {
                return Err(String::from("bad position format"));
            }
        } else if state == 1 {
            if token == "moves" {
                parse_moves = true;
                break;
            } else {
                init_fen.push_str(token);
                init_fen.push_str(" ");
            }
        } else if state == 2 {
            if token == "moves" {
                parse_moves = true;
                break;
            }
        }
    }
    let init_pos = Position::from_fen(&init_fen).ok_or("parse fen failed")?;
    let mut game = Game {
        position: init_pos,
        repeats: Vec::new(),
    };
    if parse_moves {
        for token in split.by_ref() {
            game.make_move(token).ok_or("parse move failed")?;
        }
    }
    return Ok(game);
}
