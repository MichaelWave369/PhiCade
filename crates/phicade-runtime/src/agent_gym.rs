use crate::FrameBuffer;
use serde::{Deserialize, Serialize};

pub const AGENT_GYM_ID: &str = "move-block-to-x-v1";
pub const AGENT_GYM_ROM_SHA256: &str =
    "353e69e859f50f5ef14f0221e386b18b8194f771cc603696530a59617593c59e";
pub const AGENT_GYM_SOURCE_SHA256: &str =
    "0c82f65532030d64bf022539002f334716a5aae822672b3cdfca511906e51e6f";
pub const AGENT_GYM_TARGET_X: i32 = 136;
pub const AGENT_GYM_TARGET_Y: i32 = 112;
pub const AGENT_GYM_START_X: i32 = 16;
pub const AGENT_GYM_START_Y: i32 = 24;
pub const AGENT_GYM_INITIAL_DISTANCE: i32 = 208;
pub const AGENT_GYM_SUCCESS_DISTANCE: i32 = 4;
pub const AGENT_GYM_WARMUP_FRAMES: u64 = 120;
pub const AGENT_GYM_PLAYER_SIZE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}

pub const AGENT_GYM_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_TARGET_X,
    y: AGENT_GYM_TARGET_Y,
};

pub const AGENT_GYM_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_START_X,
    y: AGENT_GYM_START_Y,
};

fn dark(pixel: &[u8]) -> bool {
    if pixel.len() < 4 {
        return false;
    }
    let r = u16::from(pixel[0]);
    let g = u16::from(pixel[1]);
    let b = u16::from(pixel[2]);
    (r + g + b) / 3 < 112
}

pub fn locate_agent_gym_player(video: &FrameBuffer) -> Result<PixelPoint, String> {
    let width = usize::try_from(video.width).map_err(|_| "video width overflow")?;
    let height = usize::try_from(video.height).map_err(|_| "video height overflow")?;

    if width < AGENT_GYM_PLAYER_SIZE || height < AGENT_GYM_PLAYER_SIZE {
        return Err(format!(
            "unexpected gym framebuffer {}x{}",
            width, height
        ));
    }
    if video.rgba8.len() != width * height * 4 {
        return Err("gym framebuffer byte length mismatch".into());
    }

    let mut best: Option<(usize, usize, usize)> = None;
    for y in 0..=height - AGENT_GYM_PLAYER_SIZE {
        for x in 0..=width - AGENT_GYM_PLAYER_SIZE {
            let mut dark_count = 0usize;
            for py in y..y + AGENT_GYM_PLAYER_SIZE {
                let row = py * width * 4;
                for px in x..x + AGENT_GYM_PLAYER_SIZE {
                    let offset = row + px * 4;
                    if dark(&video.rgba8[offset..offset + 4]) {
                        dark_count += 1;
                    }
                }
            }

            if best.is_none_or(|(_, _, count)| dark_count > count) {
                best = Some((x, y, dark_count));
            }
        }
    }

    let (x, y, count) = best.ok_or_else(|| "no candidate player patch found".to_owned())?;
    if count < 52 {
        return Err(format!(
            "solid player patch not found: best 8x8 dark-pixel count was {count}"
        ));
    }

    Ok(PixelPoint {
        x: i32::try_from(x).map_err(|_| "player x overflow")?,
        y: i32::try_from(y).map_err(|_| "player y overflow")?,
    })
}

pub fn agent_gym_distance(point: PixelPoint) -> i32 {
    (point.x - AGENT_GYM_TARGET_X).abs() + (point.y - AGENT_GYM_TARGET_Y).abs()
}

pub fn agent_gym_score_1000(initial_distance: i32, final_distance: i32) -> u16 {
    if initial_distance <= 0 {
        return 1000;
    }

    let progress = (initial_distance - final_distance).clamp(0, initial_distance);
    u16::try_from((i64::from(progress) * 1000) / i64::from(initial_distance)).unwrap_or(0)
}

pub fn agent_gym_success(point: PixelPoint) -> bool {
    agent_gym_distance(point) <= AGENT_GYM_SUCCESS_DISTANCE
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGymScore {
    pub player: PixelPoint,
    pub target: PixelPoint,
    pub initial_distance: i32,
    pub final_distance: i32,
    pub progress: i32,
    pub score_1000: u16,
    pub success: bool,
}

pub fn score_agent_gym_frame(video: &FrameBuffer) -> Result<AgentGymScore, String> {
    let player = locate_agent_gym_player(video)?;
    let final_distance = agent_gym_distance(player);
    Ok(AgentGymScore {
        player,
        target: AGENT_GYM_TARGET,
        initial_distance: AGENT_GYM_INITIAL_DISTANCE,
        final_distance,
        progress: AGENT_GYM_INITIAL_DISTANCE - final_distance,
        score_1000: agent_gym_score_1000(AGENT_GYM_INITIAL_DISTANCE, final_distance),
        success: agent_gym_success(player),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_frame(player: PixelPoint) -> FrameBuffer {
        let width = 160usize;
        let height = 144usize;
        let mut rgba8 = vec![255u8; width * height * 4];

        for y in player.y as usize..player.y as usize + AGENT_GYM_PLAYER_SIZE {
            for x in player.x as usize..player.x as usize + AGENT_GYM_PLAYER_SIZE {
                let offset = (y * width + x) * 4;
                rgba8[offset..offset + 4].copy_from_slice(&[0, 0, 0, 255]);
            }
        }

        FrameBuffer {
            width: width as u32,
            height: height as u32,
            rgba8,
        }
    }

    #[test]
    fn locates_solid_player_from_pixels() {
        let point = PixelPoint { x: 33, y: 44 };
        assert_eq!(
            locate_agent_gym_player(&synthetic_frame(point)).expect("player"),
            point
        );
    }

    #[test]
    fn frozen_geometry_matches_qualification_receipt() {
        assert_eq!(agent_gym_distance(AGENT_GYM_START), AGENT_GYM_INITIAL_DISTANCE);
        assert_eq!(agent_gym_distance(AGENT_GYM_TARGET), 0);
        assert_eq!(agent_gym_score_1000(AGENT_GYM_INITIAL_DISTANCE, 0), 1000);
        assert!(agent_gym_success(AGENT_GYM_TARGET));
    }

    #[test]
    fn scores_target_frame_at_full_credit() {
        let score = score_agent_gym_frame(&synthetic_frame(AGENT_GYM_TARGET)).expect("score");
        assert_eq!(score.final_distance, 0);
        assert_eq!(score.progress, AGENT_GYM_INITIAL_DISTANCE);
        assert_eq!(score.score_1000, 1000);
        assert!(score.success);
    }

    #[test]
    fn scores_start_frame_at_zero_progress() {
        let score = score_agent_gym_frame(&synthetic_frame(AGENT_GYM_START)).expect("score");
        assert_eq!(score.final_distance, AGENT_GYM_INITIAL_DISTANCE);
        assert_eq!(score.progress, 0);
        assert_eq!(score.score_1000, 0);
        assert!(!score.success);
    }
}
