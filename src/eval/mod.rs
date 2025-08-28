pub mod material;
pub mod nnue;

use crate::domain::IEvaluator;

pub fn make_eval(name: &str) -> Option<Box<dyn IEvaluator>> {
    match name {
        "material" => return Some(Box::new(material::MaterialEvaluationService::new())),
        "nnue" | "" => return Some(Box::new(nnue::NnueEvaluationService::new())),
        _ => None,
    }
}
