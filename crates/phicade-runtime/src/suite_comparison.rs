use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const BENCHMARK_SUITE_COMPARISON_SCHEMA: &str =
    "phicade.benchmark-suite-comparison.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiteTaskComparisonInput {
    pub task_id: String,
    pub mean_score_a_1000: f64,
    pub mean_score_b_1000: f64,
    pub success_rate_a: f64,
    pub success_rate_b: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiteTaskComparisonDelta {
    pub task_id: String,
    pub mean_score_difference_a_minus_b: f64,
    pub success_rate_difference_a_minus_b: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSuiteComparisonStats {
    pub task_count: u16,
    pub macro_mean_score_difference_a_minus_b: f64,
    pub overall_success_rate_difference_a_minus_b: f64,
    pub min_task_mean_difference_a_minus_b: f64,
    pub max_task_mean_difference_a_minus_b: f64,
    pub population_stddev_task_mean_difference: f64,
    pub tasks: Vec<SuiteTaskComparisonDelta>,
}

pub fn compare_benchmark_suites(
    tasks: &[SuiteTaskComparisonInput],
    overall_success_rate_a: f64,
    overall_success_rate_b: f64,
) -> Result<BenchmarkSuiteComparisonStats, String> {
    if tasks.is_empty() {
        return Err("benchmark suite comparison requires at least one task".into());
    }
    if !(0.0..=1.0).contains(&overall_success_rate_a)
        || !(0.0..=1.0).contains(&overall_success_rate_b)
    {
        return Err("suite overall success rates must be in 0..=1".into());
    }

    let mut seen = BTreeSet::new();
    let mut task_deltas = Vec::with_capacity(tasks.len());
    let mut mean_deltas = Vec::with_capacity(tasks.len());

    for task in tasks {
        if task.task_id.trim().is_empty() {
            return Err("suite task comparison requires a task ID".into());
        }
        if !seen.insert(task.task_id.clone()) {
            return Err(format!("duplicate suite task {}", task.task_id));
        }
        if !(0.0..=1000.0).contains(&task.mean_score_a_1000)
            || !(0.0..=1000.0).contains(&task.mean_score_b_1000)
        {
            return Err(format!(
                "suite task {} mean score is outside 0..=1000",
                task.task_id
            ));
        }
        if !(0.0..=1.0).contains(&task.success_rate_a)
            || !(0.0..=1.0).contains(&task.success_rate_b)
        {
            return Err(format!(
                "suite task {} success rate is outside 0..=1",
                task.task_id
            ));
        }

        let mean_delta = task.mean_score_a_1000 - task.mean_score_b_1000;
        mean_deltas.push(mean_delta);
        task_deltas.push(SuiteTaskComparisonDelta {
            task_id: task.task_id.clone(),
            mean_score_difference_a_minus_b: mean_delta,
            success_rate_difference_a_minus_b: task.success_rate_a - task.success_rate_b,
        });
    }

    let count = mean_deltas.len() as f64;
    let macro_delta = mean_deltas.iter().sum::<f64>() / count;
    let min_delta = mean_deltas.iter().copied().fold(f64::INFINITY, f64::min);
    let max_delta = mean_deltas
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let variance = mean_deltas
        .iter()
        .map(|value| {
            let delta = *value - macro_delta;
            delta * delta
        })
        .sum::<f64>()
        / count;

    Ok(BenchmarkSuiteComparisonStats {
        task_count: u16::try_from(tasks.len()).unwrap_or(u16::MAX),
        macro_mean_score_difference_a_minus_b: macro_delta,
        overall_success_rate_difference_a_minus_b:
            overall_success_rate_a - overall_success_rate_b,
        min_task_mean_difference_a_minus_b: min_delta,
        max_task_mean_difference_a_minus_b: max_delta,
        population_stddev_task_mean_difference: variance.sqrt(),
        tasks: task_deltas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-9, "{left} != {right}");
    }

    #[test]
    fn suite_comparison_reports_paired_task_deltas() {
        let stats = compare_benchmark_suites(
            &[
                SuiteTaskComparisonInput {
                    task_id: "a".into(),
                    mean_score_a_1000: 900.0,
                    mean_score_b_1000: 700.0,
                    success_rate_a: 0.8,
                    success_rate_b: 0.6,
                },
                SuiteTaskComparisonInput {
                    task_id: "b".into(),
                    mean_score_a_1000: 400.0,
                    mean_score_b_1000: 500.0,
                    success_rate_a: 0.2,
                    success_rate_b: 0.4,
                },
            ],
            0.5,
            0.4,
        )
        .expect("suite comparison");

        assert_eq!(stats.task_count, 2);
        close(stats.macro_mean_score_difference_a_minus_b, 50.0);
        close(stats.overall_success_rate_difference_a_minus_b, 0.1);
        close(stats.min_task_mean_difference_a_minus_b, -100.0);
        close(stats.max_task_mean_difference_a_minus_b, 200.0);
        close(stats.population_stddev_task_mean_difference, 150.0);
        close(stats.tasks[0].mean_score_difference_a_minus_b, 200.0);
        close(stats.tasks[1].success_rate_difference_a_minus_b, -0.2);
    }

    #[test]
    fn suite_comparison_refuses_duplicate_task_ids() {
        let result = compare_benchmark_suites(
            &[
                SuiteTaskComparisonInput {
                    task_id: "a".into(),
                    mean_score_a_1000: 500.0,
                    mean_score_b_1000: 500.0,
                    success_rate_a: 0.5,
                    success_rate_b: 0.5,
                },
                SuiteTaskComparisonInput {
                    task_id: "a".into(),
                    mean_score_a_1000: 500.0,
                    mean_score_b_1000: 500.0,
                    success_rate_a: 0.5,
                    success_rate_b: 0.5,
                },
            ],
            0.5,
            0.5,
        );
        assert!(result.is_err());
    }

    #[test]
    fn suite_comparison_refuses_invalid_rates_and_scores() {
        assert!(compare_benchmark_suites(&[], 0.0, 0.0).is_err());
        assert!(compare_benchmark_suites(
            &[SuiteTaskComparisonInput {
                task_id: "bad".into(),
                mean_score_a_1000: 1001.0,
                mean_score_b_1000: 500.0,
                success_rate_a: 0.5,
                success_rate_b: 0.5,
            }],
            0.5,
            0.5,
        )
        .is_err());
        assert!(compare_benchmark_suites(
            &[SuiteTaskComparisonInput {
                task_id: "bad-rate".into(),
                mean_score_a_1000: 500.0,
                mean_score_b_1000: 500.0,
                success_rate_a: 1.1,
                success_rate_b: 0.5,
            }],
            0.5,
            0.5,
        )
        .is_err());
    }
}
