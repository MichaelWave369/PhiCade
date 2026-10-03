use crate::FrameBuffer;
use serde::{Deserialize, Serialize};

pub const BENCHMARK_SUITE_V1_ID: &str = "phicade-agent-gym-suite-v1";
pub const BENCHMARK_SUITE_V2_ID: &str = "phicade-agent-gym-suite-v2";
pub const BENCHMARK_SUITE_V3_ID: &str = "phicade-agent-gym-suite-v3";

pub const AGENT_GYM_ID: &str = "move-block-to-x-v1";
pub const AGENT_GYM_ROM_SHA256: &str =
    "353e69e859f50f5ef14f0221e386b18b8194f771cc603696530a59617593c59e";
pub const AGENT_GYM_SOURCE_SHA256: &str =
    "0c82f65532030d64bf022539002f334716a5aae822672b3cdfca511906e51e6f";

pub const AGENT_GYM_MIRROR_ID: &str = "move-block-to-x-mirror-v1";
pub const AGENT_GYM_MIRROR_ROM_SHA256: &str =
    "278a8106343fe1688a1370c0575578417744c96ae52568ab1e97f446dc222bfb";
pub const AGENT_GYM_MIRROR_SOURCE_SHA256: &str =
    "fc10866cf7166f74f1ae41f8957f3b6057d8bf710731e42a02cbdb9087feb658";

pub const AGENT_GYM_WALL_ID: &str = "wall-detour-v1";
pub const AGENT_GYM_WALL_ROM_SHA256: &str =
    "c8bfbe95b368635370a61abf004d0efd4135b96c8fa9e4fb510582e25534c4f8";
pub const AGENT_GYM_WALL_SOURCE_SHA256: &str =
    "02fc444e7ffc6f9de819f7462685448268b41a641592332918f972e17fd0fc96";

pub const AGENT_GYM_TEMPORAL_LEFT_ID: &str = "temporal-cue-left-v1";
pub const AGENT_GYM_TEMPORAL_LEFT_ROM_SHA256: &str =
    "da16bdda571bd2f3d5581097e1643ce7496ae3ea586f3b1756330a2b3fc171f5";
pub const AGENT_GYM_TEMPORAL_LEFT_SOURCE_SHA256: &str =
    "450ecadadb3ed51e1e613e4edd06007880309c4841fbae313bb45f0a75932a4c";

pub const AGENT_GYM_TEMPORAL_RIGHT_ID: &str = "temporal-cue-right-v1";
pub const AGENT_GYM_TEMPORAL_RIGHT_ROM_SHA256: &str =
    "eefb364a47b68267138d35b402d2d6f5de1d70940318cac947fb400d6e210337";
pub const AGENT_GYM_TEMPORAL_RIGHT_SOURCE_SHA256: &str =
    "8c814246cb6948d50d5242ef2e01369c90a5965cb2151da5b9e4f27fff5e282f";

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

pub const AGENT_GYM_WALL_START_X: i32 = 16;
pub const AGENT_GYM_WALL_START_Y: i32 = 24;
pub const AGENT_GYM_WALL_TARGET_X: i32 = 136;
pub const AGENT_GYM_WALL_TARGET_Y: i32 = 24;
pub const AGENT_GYM_WALL_INITIAL_DISTANCE: i32 = 120;

pub const AGENT_GYM_TEMPORAL_START_X: i32 = 72;
pub const AGENT_GYM_TEMPORAL_START_Y: i32 = 96;
pub const AGENT_GYM_TEMPORAL_LEFT_TARGET_X: i32 = 24;
pub const AGENT_GYM_TEMPORAL_RIGHT_TARGET_X: i32 = 120;
pub const AGENT_GYM_TEMPORAL_TARGET_Y: i32 = 96;
pub const AGENT_GYM_TEMPORAL_INITIAL_DISTANCE: i32 = 48;

pub static AGENT_GYM_DPAD_BUTTONS: [&str; 4] = ["UP", "DOWN", "LEFT", "RIGHT"];
pub static AGENT_GYM_TEMPORAL_BUTTONS: [&str; 3] = ["A", "LEFT", "RIGHT"];

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
    /// Suite where this task was first introduced. Suite membership is owned by
    /// BenchmarkSuiteSpec so one frozen task may belong to multiple suite versions.
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
    pub allowed_buttons: &'static [&'static str],
    pub prompt: &'static str,
    pub oracle: &'static [OracleLeg],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkSuiteSpec {
    pub id: &'static str,
    pub title: &'static str,
    pub version: u16,
    pub tasks: &'static [BenchmarkTaskSpec],
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

pub const AGENT_GYM_WALL_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_WALL_START_X,
    y: AGENT_GYM_WALL_START_Y,
};

pub const AGENT_GYM_WALL_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_WALL_TARGET_X,
    y: AGENT_GYM_WALL_TARGET_Y,
};

pub const AGENT_GYM_TEMPORAL_START: PixelPoint = PixelPoint {
    x: AGENT_GYM_TEMPORAL_START_X,
    y: AGENT_GYM_TEMPORAL_START_Y,
};

pub const AGENT_GYM_TEMPORAL_LEFT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_TEMPORAL_LEFT_TARGET_X,
    y: AGENT_GYM_TEMPORAL_TARGET_Y,
};

pub const AGENT_GYM_TEMPORAL_RIGHT_TARGET: PixelPoint = PixelPoint {
    x: AGENT_GYM_TEMPORAL_RIGHT_TARGET_X,
    y: AGENT_GYM_TEMPORAL_TARGET_Y,
};

pub static AGENT_GYM_ORACLE: [OracleLeg; 2] = [
    OracleLeg {
        button: "RIGHT",
        frames: 60,
    },
    OracleLeg {
        button: "DOWN",
        frames: 44,
    },
];

pub static AGENT_GYM_MIRROR_ORACLE: [OracleLeg; 2] = [
    OracleLeg {
        button: "LEFT",
        frames: 60,
    },
    OracleLeg {
        button: "UP",
        frames: 44,
    },
];

pub static AGENT_GYM_WALL_ORACLE: [OracleLeg; 3] = [
    OracleLeg {
        button: "DOWN",
        frames: 40,
    },
    OracleLeg {
        button: "RIGHT",
        frames: 60,
    },
    OracleLeg {
        button: "UP",
        frames: 40,
    },
];

pub static AGENT_GYM_TEMPORAL_LEFT_ORACLE: [OracleLeg; 3] = [
    OracleLeg {
        button: "A",
        frames: 1,
    },
    OracleLeg {
        button: "WAIT",
        frames: 101,
    },
    OracleLeg {
        button: "LEFT",
        frames: 24,
    },
];

pub static AGENT_GYM_TEMPORAL_RIGHT_ORACLE: [OracleLeg; 3] = [
    OracleLeg {
        button: "A",
        frames: 1,
    },
    OracleLeg {
        button: "WAIT",
        frames: 101,
    },
    OracleLeg {
        button: "RIGHT",
        frames: 24,
    },
];

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
    allowed_buttons: &AGENT_GYM_DPAD_BUTTONS,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad.",
    oracle: &AGENT_GYM_ORACLE,
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
    allowed_buttons: &AGENT_GYM_DPAD_BUTTONS,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad.",
    oracle: &AGENT_GYM_MIRROR_ORACLE,
};

pub const AGENT_GYM_WALL_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V2_ID,
    id: AGENT_GYM_WALL_ID,
    title: "Wall Detour",
    rom_sha256: AGENT_GYM_WALL_ROM_SHA256,
    source_sha256: AGENT_GYM_WALL_SOURCE_SHA256,
    start: AGENT_GYM_WALL_START,
    target: AGENT_GYM_WALL_TARGET,
    initial_distance: AGENT_GYM_WALL_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_DPAD_BUTTONS,
    prompt: "Benchmark task: move the solid square block onto the visible X target using the D-pad. Navigate around visible obstacles.",
    oracle: &AGENT_GYM_WALL_ORACLE,
};

pub const AGENT_GYM_TEMPORAL_LEFT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V3_ID,
    id: AGENT_GYM_TEMPORAL_LEFT_ID,
    title: "Temporal Cue: Left",
    rom_sha256: AGENT_GYM_TEMPORAL_LEFT_ROM_SHA256,
    source_sha256: AGENT_GYM_TEMPORAL_LEFT_SOURCE_SHA256,
    start: AGENT_GYM_TEMPORAL_START,
    target: AGENT_GYM_TEMPORAL_LEFT_TARGET,
    initial_distance: AGENT_GYM_TEMPORAL_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_TEMPORAL_BUTTONS,
    prompt: "Benchmark task: memorize the visible arrow cue, press A to dismiss it, wait for the choice chamber to unlock, then move the solid block through the side indicated by the earlier cue. The choice screen intentionally does not repeat the cue.",
    oracle: &AGENT_GYM_TEMPORAL_LEFT_ORACLE,
};

pub const AGENT_GYM_TEMPORAL_RIGHT_TASK: BenchmarkTaskSpec = BenchmarkTaskSpec {
    suite_id: BENCHMARK_SUITE_V3_ID,
    id: AGENT_GYM_TEMPORAL_RIGHT_ID,
    title: "Temporal Cue: Right",
    rom_sha256: AGENT_GYM_TEMPORAL_RIGHT_ROM_SHA256,
    source_sha256: AGENT_GYM_TEMPORAL_RIGHT_SOURCE_SHA256,
    start: AGENT_GYM_TEMPORAL_START,
    target: AGENT_GYM_TEMPORAL_RIGHT_TARGET,
    initial_distance: AGENT_GYM_TEMPORAL_INITIAL_DISTANCE,
    success_distance: AGENT_GYM_SUCCESS_DISTANCE,
    warmup_frames: AGENT_GYM_WARMUP_FRAMES,
    allowed_buttons: &AGENT_GYM_TEMPORAL_BUTTONS,
    prompt: "Benchmark task: memorize the visible arrow cue, press A to dismiss it, wait for the choice chamber to unlock, then move the solid block through the side indicated by the earlier cue. The choice screen intentionally does not repeat the cue.",
    oracle: &AGENT_GYM_TEMPORAL_RIGHT_ORACLE,
};

pub static BENCHMARK_TASKS: [BenchmarkTaskSpec; 5] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
];

pub static BENCHMARK_SUITE_V1_TASKS: [BenchmarkTaskSpec; 2] =
    [AGENT_GYM_TASK, AGENT_GYM_MIRROR_TASK];

pub static BENCHMARK_SUITE_V2_TASKS: [BenchmarkTaskSpec; 3] =
    [AGENT_GYM_TASK, AGENT_GYM_MIRROR_TASK, AGENT_GYM_WALL_TASK];

pub static BENCHMARK_SUITE_V3_TASKS: [BenchmarkTaskSpec; 5] = [
    AGENT_GYM_TASK,
    AGENT_GYM_MIRROR_TASK,
    AGENT_GYM_WALL_TASK,
    AGENT_GYM_TEMPORAL_LEFT_TASK,
    AGENT_GYM_TEMPORAL_RIGHT_TASK,
];

pub static BENCHMARK_SUITE_V1: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V1_ID,
    title: "Phi-Agent Gym Suite v1",
    version: 1,
    tasks: &BENCHMARK_SUITE_V1_TASKS,
};

pub static BENCHMARK_SUITE_V2: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V2_ID,
    title: "Phi-Agent Gym Suite v2",
    version: 2,
    tasks: &BENCHMARK_SUITE_V2_TASKS,
};

pub static BENCHMARK_SUITE_V3: BenchmarkSuiteSpec = BenchmarkSuiteSpec {
    id: BENCHMARK_SUITE_V3_ID,
    title: "Phi-Agent Gym Suite v3",
    version: 3,
    tasks: &BENCHMARK_SUITE_V3_TASKS,
};

pub static BENCHMARK_SUITES: [&BenchmarkSuiteSpec; 3] =
    [&BENCHMARK_SUITE_V1, &BENCHMARK_SUITE_V2, &BENCHMARK_SUITE_V3];

pub fn benchmark_suites() -> &'static [&'static BenchmarkSuiteSpec] {
    &BENCHMARK_SUITES
}

pub fn benchmark_suite_by_id(id: &str) -> Option<&'static BenchmarkSuiteSpec> {
    BENCHMARK_SUITES.iter().copied().find(|suite| suite.id == id)
}

pub fn benchmark_suite_v1_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V1_TASKS
}

pub fn benchmark_suite_v2_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V2_TASKS
}

pub fn benchmark_suite_v3_tasks() -> &'static [BenchmarkTaskSpec] {
    &BENCHMARK_SUITE_V3_TASKS
}

pub fn benchmark_suites_for_task(task_id: &str) -> Vec<&'static BenchmarkSuiteSpec> {
    BENCHMARK_SUITES
        .iter()
        .copied()
        .filter(|suite| suite.tasks.iter().any(|task| task.id == task_id))
        .collect()
}

pub fn benchmark_task_by_id(id: &str) -> Option<&'static BenchmarkTaskSpec> {
    BENCHMARK_TASKS.iter().find(|task| task.id == id)
}

pub fn benchmark_task_by_rom_sha256(rom_sha256: &str) -> Option<&'static BenchmarkTaskSpec> {
    BENCHMARK_TASKS
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
    fn suite_v2_preserves_v1_and_adds_wall_detour() {
        assert_eq!(benchmark_suite_v2_tasks().len(), 3);
        assert_eq!(benchmark_suite_v2_tasks()[0].id, AGENT_GYM_ID);
        assert_eq!(benchmark_suite_v2_tasks()[1].id, AGENT_GYM_MIRROR_ID);
        assert_eq!(benchmark_suite_v2_tasks()[2].id, AGENT_GYM_WALL_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V1_ID).unwrap().version, 1);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V2_ID).unwrap().version, 2);
    }

    #[test]
    fn suite_v3_preserves_v2_and_adds_balanced_temporal_cues() {
        let tasks = benchmark_suite_v3_tasks();
        assert_eq!(tasks.len(), 5);
        assert_eq!(tasks[0].id, AGENT_GYM_ID);
        assert_eq!(tasks[1].id, AGENT_GYM_MIRROR_ID);
        assert_eq!(tasks[2].id, AGENT_GYM_WALL_ID);
        assert_eq!(tasks[3].id, AGENT_GYM_TEMPORAL_LEFT_ID);
        assert_eq!(tasks[4].id, AGENT_GYM_TEMPORAL_RIGHT_ID);
        assert_eq!(benchmark_suite_by_id(BENCHMARK_SUITE_V3_ID).unwrap().version, 3);
        assert_eq!(
            AGENT_GYM_TEMPORAL_LEFT_TASK.prompt,
            AGENT_GYM_TEMPORAL_RIGHT_TASK.prompt
        );
        assert_eq!(
            AGENT_GYM_TEMPORAL_LEFT_TASK.allowed_buttons,
            AGENT_GYM_TEMPORAL_RIGHT_TASK.allowed_buttons
        );
        assert_ne!(
            AGENT_GYM_TEMPORAL_LEFT_TASK.target,
            AGENT_GYM_TEMPORAL_RIGHT_TASK.target
        );
    }

    #[test]
    fn suite_membership_is_separate_from_task_origin() {
        let memberships = benchmark_suites_for_task(AGENT_GYM_ID);
        assert_eq!(memberships.len(), 3);
        assert_eq!(memberships[0].id, BENCHMARK_SUITE_V1_ID);
        assert_eq!(memberships[1].id, BENCHMARK_SUITE_V2_ID);
        assert_eq!(memberships[2].id, BENCHMARK_SUITE_V3_ID);

        let wall_memberships = benchmark_suites_for_task(AGENT_GYM_WALL_ID);
        assert_eq!(wall_memberships.len(), 2);
        assert_eq!(wall_memberships[0].id, BENCHMARK_SUITE_V2_ID);
        assert_eq!(wall_memberships[1].id, BENCHMARK_SUITE_V3_ID);

        let temporal_memberships = benchmark_suites_for_task(AGENT_GYM_TEMPORAL_LEFT_ID);
        assert_eq!(temporal_memberships.len(), 1);
        assert_eq!(temporal_memberships[0].id, BENCHMARK_SUITE_V3_ID);
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
    fn registry_resolves_wall_task_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_WALL_ID).map(|task| task.id),
            Some(AGENT_GYM_WALL_ID)
        );
        assert_eq!(AGENT_GYM_WALL_SOURCE_SHA256.len(), 64);
        assert_eq!(AGENT_GYM_WALL_ROM_SHA256.len(), 64);
        assert_eq!(AGENT_GYM_WALL_TASK.oracle.len(), 3);
    }

    #[test]
    fn registry_resolves_temporal_tasks_by_id() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_TEMPORAL_LEFT_ID).map(|task| task.id),
            Some(AGENT_GYM_TEMPORAL_LEFT_ID)
        );
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_TEMPORAL_RIGHT_ID).map(|task| task.id),
            Some(AGENT_GYM_TEMPORAL_RIGHT_ID)
        );
        assert_eq!(AGENT_GYM_TEMPORAL_LEFT_TASK.oracle[0].button, "A");
        assert_eq!(AGENT_GYM_TEMPORAL_LEFT_TASK.oracle[1].button, "WAIT");
        assert_eq!(AGENT_GYM_TEMPORAL_RIGHT_TASK.oracle[2].button, "RIGHT");
    }

    #[test]
    fn registry_resolves_mirror_task_by_exact_hash() {
        assert_eq!(
            benchmark_task_by_id(AGENT_GYM_MIRROR_ID).map(|task| task.id),
            Some(AGENT_GYM_MIRROR_ID)
        );
        assert_eq!(
            benchmark_task_by_rom_sha256(AGENT_GYM_MIRROR_ROM_SHA256).map(|task| task.id),
            Some(AGENT_GYM_MIRROR_ID)
        );
        assert_eq!(AGENT_GYM_MIRROR_ROM_SHA256.len(), 64);
        assert_eq!(AGENT_GYM_MIRROR_SOURCE_SHA256.len(), 64);
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
    fn scores_registered_targets_at_full_credit() {
        for task in BENCHMARK_TASKS.iter() {
            let score = score_benchmark_task_frame(&synthetic_frame(task.target), task)
                .expect("score target");
            assert_eq!(score.final_distance, 0);
            assert_eq!(score.progress, task.initial_distance);
            assert_eq!(score.score_1000, 1000);
            assert!(score.success);
        }
    }

    #[test]
    fn scores_registered_starts_at_zero_progress() {
        for task in BENCHMARK_TASKS.iter() {
            let score = score_benchmark_task_frame(&synthetic_frame(task.start), task)
                .expect("score start");
            assert_eq!(score.final_distance, task.initial_distance);
            assert_eq!(score.progress, 0);
            assert_eq!(score.score_1000, 0);
            assert!(!score.success);
        }
    }
}
