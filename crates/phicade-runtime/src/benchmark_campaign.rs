use serde::{Deserialize, Serialize};

pub const BENCHMARK_CAMPAIGN_SCHEMA: &str = "phicade.benchmark-campaign.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkTrialOutcome {
    pub score_1000: Option<u16>,
    pub task_success: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkCampaignStats {
    pub observed_trials: u16,
    pub scored_trials: u16,
    pub scoring_error_trials: u16,
    pub successful_trials: u16,
    pub success_rate: f64,
    pub mean_score_1000: Option<f64>,
    pub median_score_1000: Option<f64>,
    pub min_score_1000: Option<u16>,
    pub max_score_1000: Option<u16>,
    pub population_stddev_score_1000: Option<f64>,
}

pub fn summarize_benchmark_trials(
    outcomes: &[BenchmarkTrialOutcome],
) -> BenchmarkCampaignStats {
    let observed_trials = u16::try_from(outcomes.len()).unwrap_or(u16::MAX);
    let successful_trials = u16::try_from(
        outcomes.iter().filter(|outcome| outcome.task_success).count(),
    )
    .unwrap_or(u16::MAX);

    let mut scores: Vec<u16> = outcomes
        .iter()
        .filter_map(|outcome| outcome.score_1000)
        .collect();
    scores.sort_unstable();

    let scored_trials = u16::try_from(scores.len()).unwrap_or(u16::MAX);
    let scoring_error_trials = observed_trials.saturating_sub(scored_trials);
    let success_rate = if observed_trials == 0 {
        0.0
    } else {
        f64::from(successful_trials) / f64::from(observed_trials)
    };

    if scores.is_empty() {
        return BenchmarkCampaignStats {
            observed_trials,
            scored_trials,
            scoring_error_trials,
            successful_trials,
            success_rate,
            mean_score_1000: None,
            median_score_1000: None,
            min_score_1000: None,
            max_score_1000: None,
            population_stddev_score_1000: None,
        };
    }

    let count = scores.len() as f64;
    let sum: u64 = scores.iter().map(|score| u64::from(*score)).sum();
    let mean = sum as f64 / count;
    let median = if scores.len() % 2 == 1 {
        f64::from(scores[scores.len() / 2])
    } else {
        let right = scores.len() / 2;
        (f64::from(scores[right - 1]) + f64::from(scores[right])) / 2.0
    };
    let variance = scores
        .iter()
        .map(|score| {
            let delta = f64::from(*score) - mean;
            delta * delta
        })
        .sum::<f64>()
        / count;

    BenchmarkCampaignStats {
        observed_trials,
        scored_trials,
        scoring_error_trials,
        successful_trials,
        success_rate,
        mean_score_1000: Some(mean),
        median_score_1000: Some(median),
        min_score_1000: scores.first().copied(),
        max_score_1000: scores.last().copied(),
        population_stddev_score_1000: Some(variance.sqrt()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64) {
        assert!((left - right).abs() < 1e-9, "{left} != {right}");
    }

    #[test]
    fn summarizes_scored_trials_deterministically() {
        let stats = summarize_benchmark_trials(&[
            BenchmarkTrialOutcome {
                score_1000: Some(1000),
                task_success: true,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(500),
                task_success: false,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(750),
                task_success: false,
            },
        ]);

        assert_eq!(stats.observed_trials, 3);
        assert_eq!(stats.scored_trials, 3);
        assert_eq!(stats.successful_trials, 1);
        close(stats.success_rate, 1.0 / 3.0);
        close(stats.mean_score_1000.expect("mean"), 750.0);
        close(stats.median_score_1000.expect("median"), 750.0);
        assert_eq!(stats.min_score_1000, Some(500));
        assert_eq!(stats.max_score_1000, Some(1000));
        close(
            stats.population_stddev_score_1000.expect("stddev"),
            204.1241452319315,
        );
    }

    #[test]
    fn counts_scoring_errors_without_inventing_scores() {
        let stats = summarize_benchmark_trials(&[
            BenchmarkTrialOutcome {
                score_1000: None,
                task_success: false,
            },
            BenchmarkTrialOutcome {
                score_1000: Some(250),
                task_success: false,
            },
        ]);

        assert_eq!(stats.observed_trials, 2);
        assert_eq!(stats.scored_trials, 1);
        assert_eq!(stats.scoring_error_trials, 1);
        assert_eq!(stats.mean_score_1000, Some(250.0));
    }

    #[test]
    fn empty_campaign_has_no_score_statistics() {
        let stats = summarize_benchmark_trials(&[]);
        assert_eq!(stats.observed_trials, 0);
        assert_eq!(stats.success_rate, 0.0);
        assert_eq!(stats.mean_score_1000, None);
        assert_eq!(stats.population_stddev_score_1000, None);
    }
}
