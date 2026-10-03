use phicade_libretro::{LibretroCore, SAMEBOY_LICENSE, SAMEBOY_SOURCE_REVISION, SAMEBOY_VERSION};
use phicade_runtime::{AudioBuffer, EmulatorCore, FrameBuffer, GameImage, SystemId};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{env, fs, path::{Path, PathBuf}, process};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QualificationReceipt {
    schema: &'static str,
    result: &'static str,
    core_name: String,
    core_version: String,
    core_source_revision: &'static str,
    core_declared_version: &'static str,
    core_license: &'static str,
    core_sha256: String,
    fixture_name: &'static str,
    fixture_sha256: String,
    frames_requested: u64,
    frames_with_video: u64,
    frames_with_audio_callback: u64,
    width: u32,
    height: u32,
    sample_rate_hz: u32,
    final_frame_sha256: String,
}

fn hash_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn usage() -> ! {
    eprintln!("usage: qualify --core <sameboy_libretro> --rom <dmg-acid2.gb> [--frames 180] [--receipt path]");
    process::exit(2);
}

fn main() {
    if let Err(error) = run() {
        eprintln!("qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut rom_path: Option<PathBuf> = None;
    let mut frames = 180u64;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--rom" => rom_path = args.next().map(PathBuf::from),
            "--frames" => frames = args.next().and_then(|value| value.parse().ok()).unwrap_or_else(|| usage()),
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    let rom_path = rom_path.unwrap_or_else(|| usage());
    if frames < 30 { return Err("qualification requires at least 30 frames".into()); }

    let temp = env::temp_dir().join("phicade-qualify");
    let system_dir = temp.join("system");
    let save_dir = temp.join("save");
    fs::create_dir_all(&system_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|error| error.to_string())?;

    let mut core = LibretroCore::open(&core_path, &system_dir, &save_dir)
        .map_err(|error| format!("open core: {error:?}"))?;
    let identity = core.identity().clone();
    if !identity.library_name.to_ascii_lowercase().contains("sameboy") {
        return Err(format!("expected SameBoy core, got {} {}", identity.library_name, identity.library_version));
    }

    let image = GameImage::new(&rom_path, SystemId::GameBoy, "dmg-acid2");
    core.load_game(&image).map_err(|error| format!("load fixture: {error:?}"))?;

    let mut frame = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    let mut video_frames = 0u64;
    let mut audio_frames = 0u64;

    for _ in 0..frames {
        core.step_frame(&[], &mut frame, &mut audio)
            .map_err(|error| format!("run frame: {error:?}"))?;
        if !frame.rgba8.is_empty() { video_frames += 1; }
        if !audio.interleaved_stereo_f32.is_empty() { audio_frames += 1; }
    }

    if video_frames == 0 { return Err("core produced no video frames".into()); }
    if frame.width != 160 || frame.height != 144 {
        return Err(format!("unexpected final geometry {}x{}", frame.width, frame.height));
    }
    if audio.sample_rate_hz == 0 || audio_frames == 0 {
        return Err("core produced no audio callback samples".into());
    }

    let receipt = QualificationReceipt {
        schema: "phicade.core-qualification.v1",
        result: "PASS",
        core_name: identity.library_name,
        core_version: identity.library_version,
        core_source_revision: SAMEBOY_SOURCE_REVISION,
        core_declared_version: SAMEBOY_VERSION,
        core_license: SAMEBOY_LICENSE,
        core_sha256: hash_file(&core_path)?,
        fixture_name: "dmg-acid2 v1.0 (MIT)",
        fixture_sha256: hash_file(&rom_path)?,
        frames_requested: frames,
        frames_with_video: video_frames,
        frames_with_audio_callback: audio_frames,
        width: frame.width,
        height: frame.height,
        sample_rate_hz: audio.sample_rate_hz,
        final_frame_sha256: hash_bytes(&frame.rgba8),
    };

    let json = serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?;
    println!("{json}");
    if let Some(path) = receipt_path {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
        fs::write(path, format!("{json}\n")).map_err(|error| error.to_string())?;
    }

    Ok(())
}
