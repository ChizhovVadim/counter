use super::Weights;
use crate::eval::nnue;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

impl Weights {
    pub fn load(path: &Path) -> Result<Weights, std::io::Error> {
        let mut result = Weights {
            hidden_weights: Vec::new(),
            hidden_biases: [0_f32; nnue::HIDDEN_SIZE],
            output_weights: [0_f32; nnue::HIDDEN_SIZE],
            output_bias: 0_f32,
        };
        result
            .hidden_weights
            .resize(nnue::INPUT_SIZE * nnue::HIDDEN_SIZE, 0_f32);
        load_nnue_weights(&mut result, path)?;
        return Ok(result);
    }
}

pub fn find_weights_file(name: &str) -> Option<PathBuf> {
    // find in folders: <working_dir>; <exe_dir>; <user_folder>/chess;

    if let Ok(working_dir) = std::env::current_dir() {
        let path = working_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(bin_folder) = current_exe.parent() {
            let path = bin_folder.join(name);
            if path.exists() {
                return Some(path);
            }
        }
    }
    if let Some(home_var) = std::env::var_os("HOME") {
        let path = PathBuf::from(home_var).join("chess").join(name);
        if path.exists() {
            return Some(path);
        }
    }

    return None;
}

fn load_nnue_weights(weights: &mut nnue::Weights, path: &Path) -> Result<(), std::io::Error> {
    let mut file = File::open(&path)?;

    //skip 24 bytes
    let mut buf: [u8; 24] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    file.read_exact(&mut buf)?;

    let mut buf: [u8; 4] = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

    for i in 0..nnue::INPUT_SIZE {
        for j in 0..nnue::HIDDEN_SIZE {
            file.read_exact(&mut buf)?;
            weights.hidden_weights[i * nnue::HIDDEN_SIZE + j] = f32::from_ne_bytes(buf);
        }
    }

    for i in 0..nnue::HIDDEN_SIZE {
        file.read_exact(&mut buf)?;
        weights.hidden_biases[i] = f32::from_ne_bytes(buf);
    }

    for i in 0..nnue::HIDDEN_SIZE {
        file.read_exact(&mut buf)?;
        weights.output_weights[i] = f32::from_ne_bytes(buf);
    }

    file.read_exact(&mut buf)?;
    weights.output_bias = f32::from_ne_bytes(buf);

    Ok(())
}
