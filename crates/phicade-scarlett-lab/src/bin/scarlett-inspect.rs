//! No guest execution: read a user-specified file as bytes and emit metadata JSON.
use phicade_scarlett_lab::{inspect, MAX_IMAGE_BYTES};
use std::{env, fs, path::PathBuf, process};

fn main() {
    if let Err(message) = run() {
        eprintln!("scarlett-inspect: {message}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let first = args.next().ok_or_else(|| {
        "usage: scarlett-inspect <path-to-source-built-PE32+>\nRead-only, no execution".to_owned()
    })?;
    if first == "--help" || first == "-h" {
        println!("usage: scarlett-inspect <path-to-source-built-PE32+>");
        println!("Read-only generic PE32+ metadata inspection; NOT an Xbox emulator");
        return Ok(());
    }
    if args.next().is_some() {
        return Err("pass exactly one path".into());
    }
    let path = PathBuf::from(first);
    let meta = fs::metadata(&path).map_err(|e| format!("could not stat input: {e}"))?;
    if !meta.is_file() {
        return Err("path must be a regular file".into());
    }
    if meta.len() > MAX_IMAGE_BYTES as u64 {
        return Err("input exceeds the 64 MiB inspection cap".into());
    }
    let input = fs::read(&path).map_err(|e| format!("could not read input: {e}"))?;
    let report = inspect(&input).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    println!("{json}");
    Ok(())
}
