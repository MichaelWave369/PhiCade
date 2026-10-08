use crate::SparkSemanticObservation;
use serde::{Deserialize, Serialize};

pub const SPARK_TASK_QUALITY_SCHEMA: &str = "phicade.spark-task-quality.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SparkMicrotask {
    MoveEast,
    UseDash,
    UsePulse,
}

impl SparkMicrotask {
    pub fn id(self) -> &'static str {
        match self {
            Self::MoveEast => "move-east",
            Self::UseDash => "use-dash",
            Self::UsePulse => "use-pulse",
        }
    }

    pub fn objective(self) -> &'static str {
        match self {
            Self::MoveEast => {
                "Move east as far as you can using only the granted gameplay controls."
            }
            Self::UseDash => {
                "Use a dash successfully at least once using only the granted gameplay controls."
            }
            Self::UsePulse => {
                "Use Lumen Pulse successfully at least once using only the granted gameplay controls."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SparkTaskQualityEvidence {
    pub schema: String,
    pub task_id: String,
    pub objective: String,
    pub success: bool,
    pub metric_name: String,
    pub initial_metric: f64,
    pub final_metric: f64,
    pub progress: f64,
}

pub fn score_spark_microtask(
    task: SparkMicrotask,
    initial: &SparkSemanticObservation,
    final_observation: &SparkSemanticObservation,
) -> Result<SparkTaskQualityEvidence, String> {
    if final_observation.tick < initial.tick {
        return Err("SPARK task evidence cannot move backward in tick".into());
    }
    if initial.room != final_observation.room {
        return Err(
            "SPARK microtask scorer requires the initial and final observations to stay in one room"
                .into(),
        );
    }

    let (metric_name, initial_metric, final_metric, success) = match task {
        SparkMicrotask::MoveEast => {
            let initial_value = initial.player.x;
            let final_value = final_observation.player.x;
            ("playerX", initial_value, final_value, final_value > initial_value)
        }
        SparkMicrotask::UseDash => {
            let initial_value = initial.dashes as f64;
            let final_value = final_observation.dashes as f64;
            (
                "dashCount",
                initial_value,
                final_value,
                final_observation.dashes > initial.dashes,
            )
        }
        SparkMicrotask::UsePulse => {
            let initial_value = initial.powers as f64;
            let final_value = final_observation.powers as f64;
            (
                "powerUseCount",
                initial_value,
                final_value,
                final_observation.powers > initial.powers,
            )
        }
    };

    Ok(SparkTaskQualityEvidence {
        schema: SPARK_TASK_QUALITY_SCHEMA.into(),
        task_id: task.id().into(),
        objective: task.objective().into(),
        success,
        metric_name: metric_name.into(),
        initial_metric,
        final_metric,
        progress: final_metric - initial_metric,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SparkSemanticPlayer, SPARK_SEMANTIC_OBSERVATION_SCHEMA,
    };

    fn observation(x: f64, dashes: u64, powers: u64, tick: u64) -> SparkSemanticObservation {
        SparkSemanticObservation {
            schema: SPARK_SEMANTIC_OBSERVATION_SCHEMA.into(),
            tick,
            room: "threshold".into(),
            world: "threshold".into(),
            phase: "playing".into(),
            form: "spark".into(),
            player: SparkSemanticPlayer {
                x,
                y: 390.0,
                hp: 112.0,
                max_hp: 112.0,
                dash_ready: dashes == 0,
            },
            power_name: "Lumen pulse".into(),
            power_cooldown: if powers == 0 { 0.0 } else { 7.9 },
            enemy_count: 0,
            nearest_enemy: None,
            fragment_count: 0,
            exit_directions: vec!["north".into()],
            allowed_actions: vec!["MOVE".into(), "DASH".into(), "PULSE".into()],
            dashes,
            powers,
        }
    }

    #[test]
    fn scores_eastward_progress_without_inventing_a_threshold() {
        let score = score_spark_microtask(
            SparkMicrotask::MoveEast,
            &observation(480.0, 0, 0, 0),
            &observation(492.0, 0, 0, 3),
        )
        .unwrap();
        assert!(score.success);
        assert_eq!(score.metric_name, "playerX");
        assert_eq!(score.progress, 12.0);
    }

    #[test]
    fn dash_and_pulse_are_scored_from_canonical_counters() {
        let initial = observation(480.0, 0, 0, 0);
        let dashed = observation(480.0, 1, 0, 2);
        let pulsed = observation(480.0, 0, 1, 2);

        assert!(
            score_spark_microtask(SparkMicrotask::UseDash, &initial, &dashed)
                .unwrap()
                .success
        );
        assert!(
            score_spark_microtask(SparkMicrotask::UsePulse, &initial, &pulsed)
                .unwrap()
                .success
        );
    }

    #[test]
    fn refuses_cross_room_scoring() {
        let initial = observation(480.0, 0, 0, 0);
        let mut final_observation = observation(490.0, 0, 0, 2);
        final_observation.room = "other".into();
        assert!(
            score_spark_microtask(
                SparkMicrotask::MoveEast,
                &initial,
                &final_observation
            )
            .is_err()
        );
    }
}
