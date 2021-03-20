use bollard::container::{Stats, CPUStats};

pub fn calculate_cpu_usage(stats: &Stats) -> f64 {
    let precpu_stats = &stats.precpu_stats;
    let cpu_stats = &stats.cpu_stats;
    if let Some(system_cpu_delta) = calculate_system_cpu_delta(precpu_stats, cpu_stats) {
        (calculate_cpu_delta(precpu_stats, cpu_stats) as f64 / system_cpu_delta as f64)
            * number_cpus(cpu_stats) as f64
            * 100.0
    } else {
        0.0
    }
}

fn calculate_cpu_delta(precpu_stats: &CPUStats, cpu_stats: &CPUStats) -> u64 {
    cpu_stats.cpu_usage.total_usage - precpu_stats.cpu_usage.total_usage
}

fn calculate_system_cpu_delta(precpu_stats: &CPUStats, cpu_stats: &CPUStats) -> Option<u64> {
    if let (Some(cpu), Some(pre)) = (
        cpu_stats.system_cpu_usage,
        precpu_stats.system_cpu_usage,
    ) {
        Some(cpu - pre)
    } else {
        None
    }
}

fn number_cpus(cpu_stats: &CPUStats) -> u64 {
    if let Some(cpus) = cpu_stats.online_cpus {
        cpus
    } else {
        let empty = &[];
        cpu_stats
            .cpu_usage
            .percpu_usage
            .as_deref()
            .unwrap_or(empty)
            .len() as u64
    }
}
