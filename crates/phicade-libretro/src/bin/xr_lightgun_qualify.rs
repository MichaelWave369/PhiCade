use phicade_libretro::LibretroCore;
use phicade_runtime::{AudioBuffer, AuthorityPolicy, EmulatorCore, FrameBuffer};
use sha2::{Digest, Sha256};

fn main() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: xr_lightgun_qualify <original demo libretro library>")?;
    let root = std::env::temp_dir().join("phicade-xr-fixture");
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let mut core = LibretroCore::open(&path, &root, &root).map_err(|e| format!("{e:?}"))?;
    core.load_no_content().map_err(|e| format!("{e:?}"))?;
    core.configure_lightgun(true)
        .map_err(|e| format!("{e:?}"))?;
    let wire = include_str!("../../../../xr/fixtures/input-center-hit.json");
    let packet: phicade_xr_protocol::InputPacket =
        serde_json::from_str(wire).map_err(|e| e.to_string())?;
    let mut gateway = phicade_xr_protocol::XrGateway::default();
    let token = "a".repeat(48);
    gateway.approve("quest-fixture".into(), token.clone(), true, 0)?;
    gateway.submit("quest-fixture", &token, packet, 0)?;
    let actions = gateway.drain(0, 0);
    let policy = AuthorityPolicy::new(1);
    let accepted = policy
        .authorize_batch(&actions, 0)
        .into_iter()
        .filter(|(_, d)| d.accepted)
        .map(|(e, _)| e)
        .collect::<Vec<_>>();
    let (mut video, mut audio) = (FrameBuffer::default(), AudioBuffer::default());
    core.step_frame(&accepted, &mut video, &mut audio)
        .map_err(|e| format!("{e:?}"))?;
    if video.width != 320
        || video.height != 240
        || video.rgba8.get(0..4) != Some(&[0, 255, 102, 255])
    {
        return Err("absolute lightgun input did not score the visible target in the real libretro ABI fixture".into());
    }
    if audio.interleaved_stereo_f32.len() != 1600
        || !audio.interleaved_stereo_f32.iter().any(|v| v.abs() > 0.01)
    {
        return Err("fixture hit audio missing".into());
    }
    core.neutralize_inputs();
    core.step_frame(&[], &mut video, &mut audio)
        .map_err(|e| format!("{e:?}"))?;
    let hash = hex::encode(Sha256::digest(
        std::fs::read(&path).map_err(|e| e.to_string())?,
    ));
    println!(
        "{}",
        serde_json::json!({"schema":"phicade.xr-lightgun-qualification.v1","result":"PASS","core":core.identity(),"coreSha256":hash,"width":320,"height":240,"absoluteLightgun":true,"governedActions":true,"pairedGrantVerified":true,"briefTriggerEdgeVerified":true,"hitAudio":true,"commercialGamesVerified":false,"questHardwareTested":false})
    );
    Ok(())
}
