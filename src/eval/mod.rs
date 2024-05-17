use crate::chess;
use crate::types;

mod material;
mod nnue;
mod pesto;

pub fn make_eval(name: &str) -> Box<dyn types::IEvaluator> {
    match name {
        "material" => return Box::new(material::MaterialEvaluationService::new()),
        "pesto" => return Box::new(pesto::PestoEvaluationService::new()),
        "" | "nnue" => return nnue::NnueEvaluationService::new(),
        _ => panic!("make_eval {}", name),
    }
}
