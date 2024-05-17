use crate::chess::{self, Square};
use crate::types;

mod load;

const INPUT_SIZE: usize = 64 * 12;
const HIDDEN_SIZE: usize = 512;

pub struct NnueEvaluationService {
    weights: Weights, //TODO ref
    hidden_outputs: [[f32; HIDDEN_SIZE]; 128],
    current_hidden: usize,
}

pub struct Weights {
    hidden_weights: Vec<f32>, //[f32; INPUT_SIZE * HIDDEN_SIZE],
    hidden_biases: [f32; HIDDEN_SIZE],
    output_weights: [f32; HIDDEN_SIZE],
    output_bias: f32,
}

impl NnueEvaluationService {
    //TODO new(weights: ref Weights)->Box<Self>
    pub fn new() -> Box<Self> {
        let name = "n-30-5268.nn";
        let path = load::find_weights_file(name).expect(&format!("nnue file not found {}", name));
        eprintln!("load nnue {}", path.display());
        let weights = Weights::load(path.as_path()).unwrap();
        return NnueEvaluationService::with_weights(weights);
    }

    pub fn with_weights(weights: Weights) -> Box<Self> {
        let mut result = Box::new(NnueEvaluationService {
            weights: weights,
            hidden_outputs: unsafe { std::mem::MaybeUninit::uninit().assume_init() },
            current_hidden: 0,
        });
        return result;
    }
}

impl types::IEvaluator for NnueEvaluationService {
    fn init(&mut self, pos: &chess::Position) {
        self.current_hidden = 0;
        self.hidden_outputs[self.current_hidden].copy_from_slice(&self.weights.hidden_biases);

        let hidden_outputs = &mut self.hidden_outputs[self.current_hidden];
        let mut bb = pos.all_pieces();
        while bb != 0 {
            let sq = chess::bitboard::first_one(bb);
            bb &= bb - 1;
            let (side, piece) = pos.side_piece_on_square(sq);

            let input = calc_net_input_index(side, piece, sq);
            let index = input * HIDDEN_SIZE;

            for (h, w) in hidden_outputs
                .iter_mut()
                .zip(self.weights.hidden_weights[index..].iter())
            {
                *h += w;
            }
        }
    }

    //#[target_feature(enable = "avx2")]
    fn make_move(&mut self, history: &chess::History) {
        let (a, b) = self.hidden_outputs.split_at_mut(self.current_hidden + 1);
        b[0].copy_from_slice(&a[self.current_hidden]);
        self.current_hidden += 1;

        let hidden_outputs = &mut self.hidden_outputs[self.current_hidden];

        for u in &history.updates[..history.update_size] {
            let input = calc_net_input_index(u.side, u.piece, u.square);
            let index = input * HIDDEN_SIZE;
            if u.action == chess::UPDATE_ACTION_ADD {
                hidden_outputs
                    .iter_mut()
                    .zip(self.weights.hidden_weights[index..].iter())
                    .for_each(|(h, &w)| *h += w);
            } else {
                hidden_outputs
                    .iter_mut()
                    .zip(self.weights.hidden_weights[index..].iter())
                    .for_each(|(h, &w)| *h -= w);
            }
        }
    }

    fn unmake_move(&mut self) {
        self.current_hidden -= 1;
    }

    fn quik_evaluate(&mut self, _: &chess::Position) -> isize {
        let output: f32 = self.hidden_outputs[self.current_hidden]
            .iter()
            .zip(self.weights.output_weights.iter())
            //.map(|(&x, &w)| x.max(0_f32) * w)
            //.sum();
            .fold(0_f32, |acc, (&x, &w)| unsafe {
                std::intrinsics::fadd_fast(acc, x.max(0_f32) * w)
            });
        return (output + self.weights.output_bias) as isize;
    }
}

fn calc_net_input_index(side: usize, piece: usize, square: Square) -> usize {
    let mut piece12 = piece - chess::PIECE_PAWN;
    if side == chess::SIDE_BLACK {
        piece12 += 6
    }
    return square ^ piece12 << 6;
}
