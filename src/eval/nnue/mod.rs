mod load;

use crate::chess::{Move, Piece, Position, Side, Square, bitboard};
use crate::domain::IEvaluator;

const INPUT_SIZE: usize = 64 * 12;
const HIDDEN_SIZE: usize = 512;
const TOTAL_SIZE: usize = (1 + INPUT_SIZE) * HIDDEN_SIZE + (1 + HIDDEN_SIZE);

pub struct NnueEvaluationService {
    weights: Weights,
    hidden_outputs: Vec<[f32; HIDDEN_SIZE]>,
    current_hidden: usize,
}

impl NnueEvaluationService {
    pub fn new() -> Self {
        let name = "n-30-5268.nn";
        let filepath = load::find_file(name).expect(&format!("file not found {}", name));
        let raw_weights = load::load_weights(&filepath).expect("failed load nnue weight file");
        eprintln!("load nnue {}", filepath.display());
        return NnueEvaluationService {
            weights: Weights {
                weights: raw_weights,
            },
            hidden_outputs: (0..128).map(|_| [0_f32; HIDDEN_SIZE]).collect(),
            current_hidden: 0,
        };
    }
}

impl IEvaluator for NnueEvaluationService {
    fn init(&mut self, pos: &Position) {
        self.current_hidden = 0;
        let hidden_outputs = &mut self.hidden_outputs[self.current_hidden];
        hidden_outputs.copy_from_slice(self.weights.hidden_biases());
        let mut bb = pos.all_pieces();
        while bb != 0 {
            let sq = bitboard::first_one(bb);
            bb &= bb - 1;
            if let Some((side, piece)) = pos.side_piece_on_square(sq) {
                let input = calc_net_input_index(side, piece, sq);
                for (h, w) in hidden_outputs
                    .iter_mut()
                    .zip(self.weights.hidden_weights(input))
                {
                    *h += w;
                }
            }
        }
    }
    #[allow(invalid_value)]
    fn make_move(&mut self, pos: &Position, mv: Move) {
        let (a, b) = self.hidden_outputs.split_at_mut(self.current_hidden + 1);
        b[0].copy_from_slice(&a[self.current_hidden]);
        self.current_hidden += 1;
        if mv.is_null() {
            return;
        }

        // unpack move
        let side = pos.side_to_move;
        let from = mv.from();
        let to = mv.to();
        let moving_piece = mv.moving_piece();
        let captured_piece = mv.captured_piece();
        let promotion_piece = mv.promotion();

        // generate updates
        let mut updates: [Update; 4] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut updates_size = 0;

        updates[updates_size] = Update {
            input: calc_net_input_index(side, moving_piece, from),
            coeff: UPDATE_ACTION_REMOVE,
        };
        updates_size += 1;

        if captured_piece != Piece::NONE {
            let mut cap_sq = to;
            if moving_piece == Piece::PAWN && Some(to) == pos.ep_square {
                cap_sq = to.forward(side.opp());
            }

            updates[updates_size] = Update {
                input: calc_net_input_index(side.opp(), captured_piece, cap_sq),
                coeff: UPDATE_ACTION_REMOVE,
            };
            updates_size += 1;
        }

        let piece_after_move = if promotion_piece != Piece::NONE {
            promotion_piece
        } else {
            moving_piece
        };
        updates[updates_size] = Update {
            input: calc_net_input_index(side, piece_after_move, to),
            coeff: UPDATE_ACTION_ADD,
        };
        updates_size += 1;

        if moving_piece == Piece::KING {
            let mut is_catlingg = false;
            let mut rook_remove_sq = Square::A1;
            let mut rook_add_sq = Square::A1;
            if from == Square::E1 {
                if to == Square::G1 {
                    is_catlingg = true;
                    rook_remove_sq = Square::H1;
                    rook_add_sq = Square::F1;
                } else if to == Square::C1 {
                    is_catlingg = true;
                    rook_remove_sq = Square::A1;
                    rook_add_sq = Square::D1;
                }
            } else if from == Square::E8 {
                if to == Square::G8 {
                    is_catlingg = true;
                    rook_remove_sq = Square::H8;
                    rook_add_sq = Square::F8;
                } else if to == Square::C8 {
                    is_catlingg = true;
                    rook_remove_sq = Square::A8;
                    rook_add_sq = Square::D8;
                }
            }
            if is_catlingg {
                updates[updates_size] = Update {
                    input: calc_net_input_index(side, Piece::ROOK, rook_remove_sq),
                    coeff: UPDATE_ACTION_REMOVE,
                };
                updates_size += 1;
                updates[updates_size] = Update {
                    input: calc_net_input_index(side, Piece::ROOK, rook_add_sq),
                    coeff: UPDATE_ACTION_ADD,
                };
                updates_size += 1;
            }
        }

        // apply updates, SIMD
        let hidden_outputs = &mut b[0];
        for u in &updates[..updates_size] {
            if u.coeff == UPDATE_ACTION_ADD {
                hidden_outputs
                    .iter_mut()
                    .zip(self.weights.hidden_weights(u.input))
                    .for_each(|(h, &w)| *h += w);
            } else {
                hidden_outputs
                    .iter_mut()
                    .zip(self.weights.hidden_weights(u.input))
                    .for_each(|(h, &w)| *h -= w);
            }
        }
    }
    fn unmake_move(&mut self) {
        self.current_hidden -= 1;
    }

    fn quik_evaluate(&mut self, p: &Position) -> isize {
        let output: f32 = self.hidden_outputs[self.current_hidden]
            .iter()
            .zip(self.weights.output_weights())
            .fold(0_f32, |acc, (&x, &w)| {
                acc.algebraic_add(x.max(0_f32) * w) // SIMD
            });

        let mut eval = (output + self.weights.output_bias()).clamp(-15_000.0, 15_000.0) as isize;

        let np_material = (4 * (p.knights | p.bishops).count_ones()
            + 6 * p.rooks.count_ones()
            + 12 * p.queens.count_ones()) as isize;
        eval = eval * (160 + np_material) / 160;
        eval = eval * (200 - p.rule50) / 200;
        return eval;
    }
}

const UPDATE_ACTION_ADD: isize = 1;
const UPDATE_ACTION_REMOVE: isize = -1;

struct Update {
    input: usize,
    coeff: isize,
}

struct Weights {
    weights: Vec<f32>,
}

impl Weights {
    pub fn hidden_weights(&self, input: usize) -> &[f32] {
        let start = input * HIDDEN_SIZE;
        return &self.weights[start..start + HIDDEN_SIZE];
    }

    pub fn hidden_biases(&self) -> &[f32] {
        const START: usize = INPUT_SIZE * HIDDEN_SIZE;
        const END: usize = START + HIDDEN_SIZE;
        return &self.weights[START..END];
    }

    pub fn output_weights(&self) -> &[f32] {
        const START: usize = (1 + INPUT_SIZE) * HIDDEN_SIZE;
        const END: usize = START + HIDDEN_SIZE;
        return &self.weights[START..END];
    }

    pub fn output_bias(&self) -> f32 {
        const INDEX: usize = (1 + INPUT_SIZE) * HIDDEN_SIZE + HIDDEN_SIZE;
        return self.weights[INDEX];
    }
}

fn calc_net_input_index(side: Side, piece: Piece, square: Square) -> usize {
    let piece_index = if side == Side::WHITE {
        (piece as usize) - 1
    } else {
        (piece as usize) + 5
    };
    return square.index() ^ (piece_index << 6);
}
