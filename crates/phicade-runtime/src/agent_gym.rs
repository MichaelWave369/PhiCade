use crate::FrameBuffer;
use serde::{Deserialize, Serialize};

pub const BENCHMARK_SUITE_V1_ID: &str = "phicade-agent-gym-suite-v1";

pub const AGENT_GYM_ID: &str = "move-block-to-x-v1";
pub const AGENT_GYM_ROM_SHA256: &str =
    "353e69e859f50f5ef14f0221e386b18b8194f771cc603696530a59617593c59e";
pub const AGENT_GYM_SOURCE_SHA256: &str =
    "0c82f65532030d64bf022539002f334716a5aae822672b3cdfca511906e51e6f";

pub const AGENT_GYM_MIRROR_ID: &str = "move-block-to-x-mirror-v1";
pub const AGENT_GYM_MIRROR_ROM_SHA256: &str = "PENDING_MIRROR_ROM_SHA256";
pub const AGENT_GYM_MIRROR_SOURCE_SHA256: &str = "PENDING_MIRROR_SOURCE_SHA256";

pub const AGENT_GYM_TARGET_X: i32 = 136;
pub const AGENT_GYM_TARGET_Y: i32 = 112;
pub const AGENT_GYM_START_X: i32 = 16;
pub const AGENT_GYM_START_Y: i32 = 24;
pub const AGENT_GYM_INITIAL_DISTANCE: i32 = 208;
pub const AGENT_GYM_SUCCESS_DISTANCE: i32 = 4;
pub const AGENT_GYM_WARMUP_FRAMES: u64 = 120;
pub const AGENT_GYM_PLAYER_SIZE: usize = 8;

pub const AGENT_GYM_MIRROR_START_X: i32 = 136;
pub const AGENT_GYM_MIRROR_START_Y: i32 = 112;
pub const AGENT_GYM_MIRROR_TARGET_X: i32 = 16;
pub const AGENT_GYM_MIRROR_TARGET_Y: i32 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleLeg {
    pub button: &'static str,
    pub frames: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkTaskSpec {
    pub suite_id: &'static str,
    pub id: &'static str,
    pub title: &'static str,
    pub rom_sha256: &'static str,
    pub source_sha256: &'static str,
    pub start: PixelPoint,
    pub target: PixelPoint,
    pub initial_distance: i32,
    pub success_distance: i32,
    pub warmup_frames: u64,
    pub prompt: &'static str,
    pub oracle: [OracleLeg; 2],
}

pub const AGENT_GYM_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_TARGET_X,
    y: AGENT_GYM_TARGET_Y,
};

pub const AGENT_GYM_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_START_X,
    y: AGENT_GYM_START_Y,
};

pub const AGENT_GYM_MIRROR_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_MIRROR_START_X,
    y: AGENT_GYM_MIRROR_START_Y,
};

pub const AGENT_GYM_MIRROR_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_MIRROR_TARGET_X,
    y: AGENT_GYM_MIRROR_TARGET_Y,
};

pub const AGENT_GYM_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V1_ID,
    id: AGENT_GYM_ID,
    title: "Move the Block to the X",
    rom_sha256: AGENT_GYM_ROM_SHA256,
    source_sha256: AGENT_GYM_SOURCE_SHA256,
    start: AGENT_GYM_START,
    target: AGENT_GYM_TARGET,
    initial_distance: AGENT_GYM_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad.",
    oracle: [
        OracleLeg {
            button: "RIGHT",
            frames: 60,
        },
        OracleLeg {
            button: "DOWN",
            frames: 44,
        },
    ],
};

pub const AGENT_GYM_MIRROR_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V1_ID,
    id: AGENT_GYM_MIRROR_ID,
    title: "Mirror Dash",
    rom_sha256: AGENT_GYM_MIRROR_ROM_SHA256,
    source_sha256: AGENT_GYM_MIRROR_SOURCE_SHA256,
    start: AGENT_GYM_MIRROR_START,
    target: AGENT_GYM_MIRROR_TARGET,
    initial_distance: AGENT_GYM_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad.",
    oracle: [
        OracleLeg {
            button: "LEFT",
            frames: 60,
        },
        OracleLeg {
            button: "UP",
            frames: 44,
        },
    ],
};

pub static BENCHMARK_SUITE_V1_TASKS: [BenchmarkTaskSpec; 2] =
    [AGENT_GYM_TASK, AGENT_GYM_MIRROR_TASK];

pub fn benchmark_suite_v1_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V1_TASKS
}

pub fn benchmark_task_by_id(id: &str) -> Option<&'static BenchmarkTaskSpec> {
    BENCHMARK_SUITE_V1_TASKS
        .iter()
        .find(|task| task.id == id)
}

pub fn benchmark_task_by_rom_sha256(rom_sha256: &str) -> Option<&'static BenchmarkTaskSpec> {
    BENCHMARK_SUITE_V1_TASKS
        .iter()
        .find(|task| task.rom_sha256 == rom_sha256)
}

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

pub fn benchmark_task_distance(task: &BenchmarkTaskSpec, point: PixelPoint) -> i32 {
    (point.x - task.target.x).abs() + (point.y - task.target.y).abs()
}

pub fn agent_gym_distance(point: PixelPoint) -> i32 {
    benchmark_task_distance(&AGENT_GYM_TASK, point)
}

pub fn agent_gym_score_1000(initial_distance: i32, final_distance: i32) -> u16 {
    if initial_distance <= 0 {
        return 1000;
    }

    let progress = (initial_distance - final_distance).clamp(0, initial_distance);
    u16::try_from((i64::from(progress) * 1000) / i64::from(initial_distance)).unwrap_or(0)
}

pub fn benchmark_task_success(task: &BenchmarkTaskSpec, point: PixelPoint) -> bool {
    benchmark_task_distance(task, point) <= task.success_distance
}

pub fn agent_gym_success(point: PixelPoint) -> bool {
    benchmark_task_success(&AGENT_GYM_TASK, point)
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

pub fn score_benchmark_task_frame(
    video: &FrameBuffer,
    task: &BenchmarkTaskSpec,
) -> Result<AgentGymScore, String> {
    let player = locate_agent_gym_player(video)?;
    let final_distance = benchmark_task_distance(task, player);
    Ok(AgentGymScore {
        player,
        target: task.target,
        initial_distance: task.initial_distance,
        final_distance,
        progress: task.initial_distance - final_distance,
        score_1000: agent_gym_score_1000(task.initial_distance, final_distance),
        success: benchmark_task_success(task, player),
    })
}

pub fn score_agent_gym_frame(video: &FrameBuffer) -> Result<AgentGymScore, String> {
    score_benchmark_task_frame(video, &AGENT_GYM_TASK)
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
    fn suite_v1_contains_two_distinct_tasks() {
        assert_eq!(benchmark_suite_v1_tasks().len(), 2);
        assert_ne!(AGENT_GYM_TASK.id, AGENT_GYM_MIRROR_TASK.id);
        assert_ne!(AGENT_GYM_TASK.start, AGENT_GYM_MIRROR_TASK.start);
        assert_ne!(AGENT_GYM_TASK.target, AGENT_GYM_MIRROR_TASK.target);
    }

    #[test]
    fn registry_resolves_original_task_by_id_and_hash() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_ID).map(|task| task.id),
            Some(AGENT_GYM_ID)
        );
        assert_eq!(
            benchmark_task_by_rom_sha256(AGENT_GYM_ROM_SHA256).map(|task| task.id),
            Some(AGENT_GYM_ID)
        );
    }

    #[test]
    fn both_task_geometries_have_same_frozen_distance() {
        assert_eq!(
            benchmark_task_distance(&AGENT_GYM_TASK, AGENT_GYM_TASK.start),
            AGENT_GYM_INITIAL_DISTANCE
        );
        assert_eq!(
            benchmark_task_distance(&AGENT_GYM_MIRROR_TASK, AGENT_GYM_MIRROR_TASK.start),
            AGENT_GYM_INITIAL_DISTANCE
        );
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
    fn original_compatibility_wrappers_match_task_registry() {
        assert_eq!(agent_gym_distance(AGENT_GYM_START), AGENT_GYM_INITIAL_DISTANCE);
        assert_eq!(agent_gym_distance(AGENT_GYM_TARGET), 0);
        assert_eq!(agent_gym_score_1000(AGENT_GYM_INITIAL_DISTANCE, 0), 1000);
        assert!(agent_gym_success(AGENT_GYM_TARGET));
    }

    #[test]
    fn scores_both_targets_at_full_credit() {
        for task in benchmark_suite_v1_tasks() {
            let score = score_benchmark_task_frame(&synthetic_frame(task.target), task)
                .expect("score target");
            assert_eq!(score.final_distance, 0);
            assert_eq!(score.progress, task.initial_distance);
            assert_eq!(score.score_1000, 1000);
            assert!(score.success);
        }
    }

    #[test]
    fn scores_both_starts_at_zero_progress() {
        for task in benchmark_suite_v1_tasks() {
            let score = score_benchmark_task_frame(&synthetic_frame(task.start), task)
                .expect("score start");
            assert_eq!(score.final_distance, task.initial_distance);
            assert_eq!(score.progress, 0);
            assert_eq!(score.score_1000, 0);
            assert!(!score.success);
        }
    }
}
