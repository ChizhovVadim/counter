use super::TOTAL_SIZE;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn find_file(name: &str) -> Option<PathBuf> {
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
    if let Some(home_var) = std::env::home_dir() {
        let path = home_var.join("chess").join(name);
        if path.exists() {
            return Some(path);
        }
    }

    return None;
}

pub fn load_weights(path: &Path) -> Result<Vec<f32>, std::io::Error> {
    let mut file = File::open(path)?;

    //skip 24 bytes
    let mut buf = [0_u8; 24];
    file.read_exact(&mut buf)?;

    let mut buf = [0_u8; 4];
    let mut res = Vec::with_capacity(TOTAL_SIZE);
    for _ in 0..TOTAL_SIZE {
        file.read_exact(&mut buf)?;
        let w = f32::from_ne_bytes(buf);
        res.push(w);
    }

    return Ok(res);
}
