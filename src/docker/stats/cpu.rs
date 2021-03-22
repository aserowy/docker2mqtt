use bollard::container::CPUStats;

pub fn calculate_cpu_usage(precpu_stats: &CPUStats, cpu_stats: &CPUStats) -> f64 {
    if let Some(system_cpu_delta) = calculate_system_cpu_delta(precpu_stats, cpu_stats) {
        calculate_relative_cpu_usage(precpu_stats, cpu_stats, system_cpu_delta)
    } else {
        0.0
    }
}

fn calculate_relative_cpu_usage(
    precpu_stats: &CPUStats,
    cpu_stats: &CPUStats,
    system_cpu_delta: u64,
) -> f64 {
    let delta_cpu_usage =
        (cpu_stats.cpu_usage.total_usage - precpu_stats.cpu_usage.total_usage) as f64;
    (delta_cpu_usage / system_cpu_delta as f64) * number_cpus(cpu_stats) as f64 * 100.0
}

fn calculate_system_cpu_delta(precpu_stats: &CPUStats, cpu_stats: &CPUStats) -> Option<u64> {
    if let (Some(cpu), Some(pre)) = (cpu_stats.system_cpu_usage, precpu_stats.system_cpu_usage) {
        Some(cpu - pre)
    } else {
        None
    }
}

fn number_cpus(cpu_stats: &CPUStats) -> u64 {
    if let Some(cpus) = cpu_stats.online_cpus {
        cpus
    } else {
        cpu_stats
            .cpu_usage
            .percpu_usage
            .as_deref()
            .unwrap_or(&[])
            .len() as u64
    }
}

#[cfg(test)]
mod must {
    use crate::docker::stats::cpu::calculate_cpu_usage;
    use bollard::container::{CPUStats, CPUUsage, ThrottlingData};

    fn create_cpu_stats(
        percpu_usage: Option<Vec<u64>>,
        total_usage: u64,
        system_cpu_usage: Option<u64>,
        online_cpus: Option<u64>,
    ) -> CPUStats {
        CPUStats {
            cpu_usage: CPUUsage {
                percpu_usage,
                usage_in_usermode: 0,
                total_usage,
                usage_in_kernelmode: 0,
            },
            system_cpu_usage,
            online_cpus,
            throttling_data: ThrottlingData {
                periods: 0,
                throttled_periods: 0,
                throttled_time: 0,
            },
        }
    }

    const FLOAT_ERROR_MARGIN: f64 = 0.0099;

    #[test]
    fn return_correct_cpu_usage_without_percpu_usage() {
        let precpu_stats = create_cpu_stats(None, 60, Some(70), Some(2));
        let cpu_stats = create_cpu_stats(None, 75, Some(80), Some(2));

        assert!(
            (calculate_cpu_usage(&precpu_stats, &cpu_stats) - 300.0).abs() < FLOAT_ERROR_MARGIN
        );
    }

    #[test]
    fn return_correct_cpu_usage_with_percpu_usage() {
        let precpu_stats = create_cpu_stats(Some(vec![25, 45]), 60, Some(70), None);
        let cpu_stats = create_cpu_stats(Some(vec![35, 45]), 75, Some(80), None);

        assert!(
            (calculate_cpu_usage(&precpu_stats, &cpu_stats) - 300.0).abs() < FLOAT_ERROR_MARGIN
        );
    }

    #[test]
    fn return_zero_cpu_usage_without_system_cpu_usage() {
        let precpu_stats = create_cpu_stats(None, 60, Some(70), Some(2));
        let precpu_stats_zero_system = create_cpu_stats(None, 60, None, Some(2));
        let cpu_stats = create_cpu_stats(None, 75, Some(80), Some(2));
        let cpu_stats_zero_system = create_cpu_stats(None, 75, None, Some(2));

        assert!(
            (calculate_cpu_usage(&precpu_stats_zero_system, &cpu_stats) - 0.0).abs()
                < FLOAT_ERROR_MARGIN
        );
        assert!(
            (calculate_cpu_usage(&precpu_stats, &cpu_stats_zero_system) - 0.0).abs()
                < FLOAT_ERROR_MARGIN
        );
    }
}
