use phicade_libretro::{
    LibretroCore, SCUMMVM_LICENSE, SCUMMVM_QUALIFICATION_PROFILE_ID, SCUMMVM_SOURCE_REVISION,
    SCUMMVM_VERSION,
};
use phicade_runtime::{
    ActionEnvelope, ActionKind, ActionSource, AudioBuffer, CapabilityStatus, EmulatorCore,
    FrameBuffer, RuntimeCapabilityManifest,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScummVmQualificationReceipt {
    schema: &'static str,
    result: &'static str,
    core_name: String,
    core_version: String,
    core_source_revision: &'static str,
    core_declared_version: &'static str,
    core_license: &'static str,
    core_sha256: String,
    qualification_profile_id: &'static str,
    capability_manifest: RuntimeCapabilityManifest,
    no_content_supported: bool,
    frames_requested: u64,
    frames_with_video: u64,
    frames_with_audio_callback: u64,
    width: u32,
    height: u32,
    sample_rate_hz: u32,
    final_frame_sha256: String,
    governed_button_path_pass: bool,
    governed_analog_path_pass: bool,
    serialize_size_bytes: usize,
    serialize_attempt_refused: bool,
    save_ram_bytes: usize,
}

fn usage() -> ! {
    eprintln!(
        "usage: qualify_scummvm --core <scummvm_libretro> [--frames 180] [--receipt path]"
    );
    process::exit(2);
}

fn hash_file(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn button(sequence: u64, frame: u64, name: &str, pressed: bool) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source: ActionSource::Script {
            name: "scummvm-qualification".into(),
        },
        action: ActionKind::Button {
            button: name.into(),
            pressed,
        },
    }
}

fn axis(sequence: u64, frame: u64, name: &str, value: i16) -> ActionEnvelope {
    ActionEnvelope {
        sequence,
        frame,
        source: ActionSource::Script {
            name: "scummvm-qualification".into(),
        },
        action: ActionKind::Axis {
            axis: name.into(),
            value,
        },
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ScummVM qualification failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut core_path: Option<PathBuf> = None;
    let mut frames = 180u64;
    let mut receipt_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--core" => core_path = args.next().map(PathBuf::from),
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| usage())
            }
            "--receipt" => receipt_path = args.next().map(PathBuf::from),
            _ => usage(),
        }
    }

    let core_path = core_path.unwrap_or_else(|| usage());
    if frames < 60 {
        return Err("qualification requires at least 60 frames".into());
    }

    let temp = env::temp_dir().join("phicade-scummvm-qualify");
    let system_dir = temp.join("system");
    let save_dir = temp.join("save");
    fs::create_dir_all(&system_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&save_dir).map_err(|error| error.to_string())?;

    let mut core = LibretroCore::open(&core_path, &system_dir, &save_dir)
        .map_err(|error| format!("open core: {error:?}"))?;
    let identity = core.identity().clone();

    if identity.library_name != "ScummVM" {
        return Err(format!(
            "expected ScummVM core name, got {:?}",
            identity.library_name
        ));
    }
    if identity.library_version != SCUMMVM_VERSION {
        return Err(format!(
            "expected ScummVM version {SCUMMVM_VERSION}, got {:?}",
            identity.library_version
        ));
    }
    if !core.supports_no_game() {
        return Err("ScummVM core did not request no-content support".into());
    }

    core.load_no_content()
        .map_err(|error| format!("load no-content launcher: {error:?}"))?;

    let manifest = core.capability_manifest();
    let profile = manifest
        .qualification_profile
        .as_ref()
        .ok_or_else(|| "missing ScummVM qualification profile".to_string())?;
    if profile.profile_id != SCUMMVM_QUALIFICATION_PROFILE_ID {
        return Err(format!(
            "unexpected qualification profile {}",
            profile.profile_id
        ));
    }
    if manifest.capabilities.state_snapshots != CapabilityStatus::Unsupported {
        return Err("ScummVM unexpectedly advertises state snapshots".into());
    }
    if manifest.capabilities.exact_replay != CapabilityStatus::Unsupported {
        return Err("ScummVM unexpectedly advertises exact replay".into());
    }
    if manifest.capabilities.persistent_save_data != CapabilityStatus::Unsupported {
        return Err("ScummVM unexpectedly advertises libretro persistent save RAM".into());
    }

    let mut video = FrameBuffer::default();
    let mut audio = AudioBuffer::default();
    let mut frames_with_video = 0u64;
    let mut frames_with_audio_callback = 0u64;

    for frame_index in 0..frames {
        let actions = if frame_index == 30 {
            vec![button(0, frame_index, "RIGHT", true)]
        } else if frame_index == 40 {
            vec![button(1, frame_index, "RIGHT", false)]
        } else if frame_index == 50 {
            vec![axis(2, frame_index, "LEFT_X", 20_000)]
        } else if frame_index == 60 {
            vec![axis(3, frame_index, "LEFT_X", 0)]
        } else {
            Vec::new()
        };

        core.step_frame(&actions, &mut video, &mut audio)
            .map_err(|error| format!("run launcher frame {frame_index}: {error:?}"))?;

        if !video.rgba8.is_empty() {
            frames_with_video += 1;
        }
        if !audio.interleaved_stereo_f32.is_empty() {
            frames_with_audio_callback += 1;
        }

        if frame_index == 30 && core.input_mask_snapshot() == 0 {
            return Err("governed RIGHT button did not reach libretro input mask".into());
        }
        if frame_index == 40 && core.input_mask_snapshot() != 0 {
            return Err("governed RIGHT release did not clear libretro input mask".into());
        }
        if frame_index == 50 && core.analog_inputs_snapshot()[0] != 20_000 {
            return Err("governed LEFT_X axis did not reach libretro analog state".into());
        }
        if frame_index == 60 && core.analog_inputs_snapshot()[0] != 0 {
            return Err("governed LEFT_X neutral action did not clear analog state".into());
        }
    }

    if frames_with_video == 0 || video.rgba8.is_empty() {
        return Err("ScummVM launcher produced no software framebuffer".into());
    }
    if video.width == 0 || video.height == 0 {
        return Err(format!(
            "ScummVM launcher reported invalid geometry {}x{}",
            video.width, video.height
        ));
    }

    let serialize_size_bytes = core.serialize_size();
    if serialize_size_bytes != 0 {
        return Err(format!(
            "ScummVM unexpectedly reported {serialize_size_bytes} serializable state bytes"
        ));
    }
    let serialize_attempt_refused = core.serialize_state().is_err();
    if !serialize_attempt_refused {
        return Err("ScummVM unexpectedly serialized state".into());
    }

    let save_ram_bytes = core.save_ram_size();
    if save_ram_bytes != 0 {
        return Err(format!(
            "ScummVM unexpectedly exposed {save_ram_bytes} bytes of libretro save RAM"
        ));
    }

    let receipt = ScummVmQualificationReceipt {
        schema: "phicade.scummvm-qualification.v1",
        result: "PASS",
        core_name: identity.library_name,
        core_version: identity.library_version,
        core_source_revision: SCUMMVM_SOURCE_REVISION,
        core_declared_version: SCUMMVM_VERSION,
        core_license: SCUMMVM_LICENSE,
        core_sha256: hash_file(&core_path)?,
        qualification_profile_id: SCUMMVM_QUALIFICATION_PROFILE_ID,
        capability_manifest: manifest,
        no_content_supported: true,
        frames_requested: frames,
        frames_with_video,
        frames_with_audio_callback,
        width: video.width,
        height: video.height,
        sample_rate_hz: audio.sample_rate_hz,
        final_frame_sha256: hash_bytes(&video.rgba8),
        governed_button_path_pass: true,
        governed_analog_path_pass: true,
        serialize_size_bytes,
        serialize_attempt_refused,
        save_ram_bytes,
    };

    let json = serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?;
    println!("{json}");

    if let Some(path) = receipt_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, format!("{json}\n")).map_err(|error| error.to_string())?;
    }

    Ok(())
}
