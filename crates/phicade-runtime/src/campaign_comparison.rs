use serde::{Deserialize, Serialize};

pub const CAMPAIGN_COMPARISON_SCHEMA: &str = "phicade.campaign-comparison.v1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignSampleSummary {
    pub scored_trials: u16,
    pub successful_trials: u16,
    pub observed_trials: u16,
    pub mean_score_1000: f64,
    pub sample_variance_score_1000: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignComparisonStats {
    pub sample_a: CampaignSampleSummary,
    pub sample_b: CampaignSampleSummary,
    pub mean_score_difference_a_minus_b: f64,
    pub welch_standard_error: f64,
    pub welch_degrees_of_freedom: Option<f64>,
    pub mean_difference_ci95_low: f64,
    pub mean_difference_ci95_high: f64,
    pub hedges_g_a_minus_b: Option<f64>,
    pub success_rate_difference_a_minus_b: f64,
}

fn mean(scores: &[u16]) -> f64 {
    scores.iter().map(|score| f64::from(*score)).sum::<f64>() / scores.len() as f64
}

fn sample_variance(scores: &[u16], sample_mean: f64) -> f64 {
    if scores.len() < 2 {
        return 0.0;
    }

    scores
        .iter()
        .map(|score| {
            let delta = f64::from(*score) - sample_mean;
            delta * delta
        })
        .sum::<f64>()
        / (scores.len() - 1) as f64
}

fn t_critical_975_conservative(df: f64) -> f64 {
    // Two-sided 95% Student-t critical values. Campaigns are capped at 20 trials
    // per side, so Welch df cannot exceed 38. Fractional df uses floor(df),
    // which is conservative because the critical value decreases as df rises.
    const TABLE: [f64; 40] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228,
        2.201, 2.179, 2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086,
        2.080, 2.074, 2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
        2.040, 2.037, 2.035, 2.032, 2.030, 2.028, 2.026, 2.024, 2.023, 2.021,
    ];

    let index = df.floor().clamp(1.0, 40.0) as usize - 1;
    TABLE[index]
}

fn sample_summary(
    scores: &[u16],
    successful_trials: u16,
    observed_trials: u16,
) -> Result<CampaignSampleSummary, String> {
    if scores.len() < 2 {
        return Err("campaign comparison requires at least two scored trials per campaign".into());
    }
    if observed_trials == 0 {
        return Err("campaign comparison requires observed trials".into());
    }
    if usize::from(successful_trials) > usize::from(observed_trials) {
        return Err("successful trial count exceeds observed trials".into());
    }

    let sample_mean = mean(scores);
    Ok(CampaignSampleSummary {
        scored_trials: u16::try_from(scores.len()).unwrap_or(u16::MAX),
        successful_trials,
        observed_trials,
        mean_score_1000: sample_mean,
        sample_variance_score_1000: sample_variance(scores, sample_mean),
        success_rate: f64::from(successful_trials) / f64::from(observed_trials),
    })
}

pub fn compare_campaign_samples(
    scores_a: &[u16],
    successful_trials_a: u16,
    observed_trials_a: u16,
    scores_b: &[u16],
    successful_trials_b: u16,
    observed_trials_b: u16,
) -> Result<CampaignComparisonStats, String> {
    let sample_a = sample_summary(scores_a, successful_trials_a, observed_trials_a)?;
    let sample_b = sample_summary(scores_b, successful_trials_b, observed_trials_b)?;

    let n_a = f64::from(sample_a.scored_trials);
    let n_b = f64::from(sample_b.scored_trials);
    let mean_difference = sample_a.mean_score_1000 - sample_b.mean_score_1000;
    let component_a = sample_a.sample_variance_score_1000 / n_a;
    let component_b = sample_b.sample_variance_score_1000 / n_b;
    let standard_error = (component_a + component_b).sqrt();

    let (welch_df, ci_low, ci_high) = if standard_error == 0.0 {
        (None, mean_difference, mean_difference)
    } else {
        let numerator = (component_a + component_b).powi(2);
        let denominator =
            component_a.powi(2) / (n_a - 1.0) + component_b.powi(2) / (n_b - 1.0);
        let df = numerator / denominator;
        let critical = t_critical_975_conservative(df);
        let margin = critical * standard_error;
        (
            Some(df),
            mean_difference - margin,
            mean_difference + margin,
        )
    };

    let pooled_degrees = n_a + n_b - 2.0;
    let pooled_variance = (
        (n_a - 1.0) * sample_a.sample_variance_score_1000
            + (n_b - 1.0) * sample_b.sample_variance_score_1000
    ) / pooled_degrees;
    let hedges_g = if pooled_variance > 0.0 {
        let cohen_d = mean_difference / pooled_variance.sqrt();
        let correction = 1.0 - 3.0 / (4.0 * pooled_degrees - 1.0);
        Some(correction * cohen_d)
    } else if mean_difference == 0.0 {
        Some(0.0)
    } else {
        None
    };

    Ok(CampaignComparisonStats {
        sample_a,
        sample_b,
        mean_score_difference_a_minus_b: mean_difference,
        welch_standard_error: standard_error,
        welch_degrees_of_freedom: welch_df,
        mean_difference_ci95_low: ci_low,
        mean_difference_ci95_high: ci_high,
        hedges_g_a_minus_b: hedges_g,
        success_rate_difference_a_minus_b: sample_a.success_rate - sample_b.success_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64, tolerance: f64) {
        assert!((left - right).abs() <= tolerance, "{left} != {right}");
    }

    #[test]
    fn identical_samples_have_zero_delta_and_zero_effect() {
        let stats = compare_campaign_samples(
            &[100, 200, 300, 400, 500],
            2,
            5,
            &[100, 200, 300, 400, 500],
            2,
            5,
        )
        .expect("comparison");

        close(stats.mean_score_difference_a_minus_b, 0.0, 1e-9);
        close(stats.mean_difference_ci95_low, -277.718, 0.01);
        close(stats.mean_difference_ci95_high, 277.718, 0.01);
        close(stats.hedges_g_a_minus_b.expect("g"), 0.0, 1e-9);
        close(stats.success_rate_difference_a_minus_b, 0.0, 1e-9);
    }

    #[test]
    fn separated_samples_report_positive_a_minus_b_effect() {
        let stats = compare_campaign_samples(
            &[800, 850, 900, 950, 1000],
            4,
            5,
            &[200, 250, 300, 350, 400],
            1,
            5,
        )
        .expect("comparison");

        close(stats.mean_score_difference_a_minus_b, 600.0, 1e-9);
        assert!(stats.mean_difference_ci95_low > 300.0);
        assert!(stats.mean_difference_ci95_high > stats.mean_difference_ci95_low);
        assert!(stats.hedges_g_a_minus_b.expect("g") > 3.0);
        close(stats.success_rate_difference_a_minus_b, 0.6, 1e-9);
    }

    #[test]
    fn zero_variance_equal_samples_have_exact_zero_interval() {
        let stats = compare_campaign_samples(&[500, 500, 500], 0, 3, &[500, 500, 500], 0, 3)
            .expect("comparison");
        assert_eq!(stats.welch_degrees_of_freedom, None);
        close(stats.mean_difference_ci95_low, 0.0, 1e-9);
        close(stats.mean_difference_ci95_high, 0.0, 1e-9);
        close(stats.hedges_g_a_minus_b.expect("g"), 0.0, 1e-9);
    }

    #[test]
    fn refuses_single_scored_trial() {
        let result = compare_campaign_samples(&[500], 0, 1, &[500, 600], 0, 2);
        assert!(result.is_err());
    }
}
