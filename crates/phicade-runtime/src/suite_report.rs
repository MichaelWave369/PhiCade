use serde::{Deserialize, Serialize};

pub const BENCHMARK_SUITE_REPORT_SCHEMA: &str = "phicade.benchmark-suite-report.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiteTaskAggregateInput {
    pub task_id: String,
    pub mean_score_1000: f64,
    pub observed_trials: u16,
    pub successful_trials: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkSuiteAggregateStats {
    pub task_count: u16,
    pub total_observed_trials: u32,
    pub total_successful_trials: u32,
    pub overall_success_rate: f64,
    pub macro_mean_score_1000: f64,
    pub min_task_mean_score_1000: f64,
    pub max_task_mean_score_1000: f64,
    pub population_stddev_task_mean_score_1000: f64,
}

pub fn summarize_benchmark_suite(
    tasks: &[SuiteTaskAggregateInput],
) -> Result<BenchmarkSuiteAggregateStats, String> {
    if tasks.is_empty() {
        return Err("benchmark suite report requires at least one task".into());
    }

    let mut total_observed = 0u32;
    let mut total_successful = 0u32;
    let mut means = Vec::with_capacity(tasks.len());

    for task in tasks {
        if task.observed_trials == 0 {
            return Err(format!(
                "suite task {} has zero observed trials",
                task.task_id
            ));
        }
        if task.successful_trials > task.observed_trials {
            return Err(format!(
                "suite task {} has more successes than observed trials",
                task.task_id
            ));
        }
        if !(0.0..=1000.0).contains(&task.mean_score_1000) {
            return Err(format!(
                "suite task {} mean score is outside 0..=1000",
                task.task_id
            ));
        }

        total_observed = total_observed.saturating_add(u32::from(task.observed_trials));
        total_successful =
            total_successful.saturating_add(u32::from(task.successful_trials));
        means.push(task.mean_score_1000);
    }

    let task_count = means.len() as f64;
    let macro_mean = means.iter().sum::<f64>() / task_count;
    let min_mean = means
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_mean = means
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let variance = means
        .iter()
        .map(|value| {
            let delta = *value - macro_mean;
            delta * delta
        })
        .sum::<f64>()
        / task_count;

    Ok(BenchmarkSuiteAggregateStats {
        task_count: u16::try_from(tasks.len()).unwrap_or(u16::MAX),
        total_observed_trials: total_observed,
        total_successful_trials: total_successful,
        overall_success_rate: f64::from(total_successful) / f64::from(total_observed),
        macro_mean_score_1000: macro_mean,
        min_task_mean_score_1000: min_mean,
        max_task_mean_score_1000: max_mean,
        population_stddev_task_mean_score_1000: variance.sqrt(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-9, "{left} != {right}");
    }

    #[test]
    fn suite_macro_mean_weights_tasks_equally() {
        let stats = summarize_benchmark_suite(&[
            SuiteTaskAggregateInput {
                task_id: "a".into(),
                mean_score_1000: 900.0,
                observed_trials: 5,
                successful_trials: 4,
            },
            SuiteTaskAggregateInput {
                task_id: "b".into(),
                mean_score_1000: 300.0,
                observed_trials: 5,
                successful_trials: 1,
            },
        ])
        .expect("suite stats");

        assert_eq!(stats.task_count, 2);
        assert_eq!(stats.total_observed_trials, 10);
        assert_eq!(stats.total_successful_trials, 5);
        close(stats.overall_success_rate, 0.5);
        close(stats.macro_mean_score_1000, 600.0);
        close(stats.min_task_mean_score_1000, 300.0);
        close(stats.max_task_mean_score_1000, 900.0);
        close(stats.population_stddev_task_mean_score_1000, 300.0);
    }

    #[test]
    fn suite_success_rate_uses_trials_not_task_average() {
        let stats = summarize_benchmark_suite(&[
            SuiteTaskAggregateInput {
                task_id: "a".into(),
                mean_score_1000: 500.0,
                observed_trials: 10,
                successful_trials: 10,
            },
            SuiteTaskAggregateInput {
                task_id: "b".into(),
                mean_score_1000: 500.0,
                observed_trials: 2,
                successful_trials: 0,
            },
        ])
        .expect("suite stats");

        close(stats.overall_success_rate, 10.0 / 12.0);
        close(stats.macro_mean_score_1000, 500.0);
    }

    #[test]
    fn suite_rejects_invalid_task_input() {
        assert!(summarize_benchmark_suite(&[]).is_err());
        assert!(summarize_benchmark_suite(&[SuiteTaskAggregateInput {
            task_id: "bad".into(),
            mean_score_1000: 1200.0,
            observed_trials: 5,
            successful_trials: 1,
        }])
        .is_err());
    }
}
