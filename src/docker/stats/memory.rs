use bollard::container::{MemoryStats, MemoryStatsStats};
use tracing::warn;

pub fn usage(stats: &MemoryStats) -> f64 {
    match (stats.usage, stats.stats, stats.limit) {
        (Some(usage), Some(stats), Some(limit)) => match stats {
            MemoryStatsStats::V1(stats_v1) => {
                let used_memory = usage - stats_v1.cache;
                (used_memory as f64 / limit as f64) * 100.0
            }
            MemoryStatsStats::V2(_) => {
                warn!("v2 cgroups are not supported for memory stats.");
                0.0
            }
        },
        _ => 0.0,
    }
}

#[cfg(test)]
mod must {
    use crate::docker::stats::memory::usage;
    use bollard::container::{MemoryStats, MemoryStatsStats, MemoryStatsStatsV1};

    fn create_memory_stats(
        stats: Option<MemoryStatsStats>,
        usage: Option<u64>,
        limit: Option<u64>,
    ) -> MemoryStats {
        MemoryStats {
            stats,
            max_usage: None,
            usage,
            failcnt: None,
            limit,
            commit: None,
            commit_peak: None,
            commitbytes: None,
            commitpeakbytes: None,
            privateworkingset: None,
        }
    }

    fn create_memory_stats_stats(cache: u64) -> MemoryStatsStats {
        MemoryStatsStats::V1(MemoryStatsStatsV1 {
            cache,
            dirty: 0,
            mapped_file: 0,
            total_inactive_file: 0,
            pgpgout: 0,
            rss: 0,
            total_mapped_file: 0,
            writeback: 0,
            unevictable: 0,
            pgpgin: 0,
            total_unevictable: 0,
            pgmajfault: 0,
            total_rss: 0,
            total_rss_huge: 0,
            total_writeback: 0,
            total_inactive_anon: 0,
            rss_huge: 0,
            hierarchical_memory_limit: 0,
            total_pgfault: 0,
            total_active_file: 0,
            active_anon: 0,
            total_active_anon: 0,
            total_pgpgout: 0,
            total_cache: 0,
            total_dirty: 0,
            inactive_anon: 0,
            active_file: 0,
            pgfault: 0,
            inactive_file: 0,
            total_pgmajfault: 0,
            total_pgpgin: 0,
            hierarchical_memsw_limit: None,
        })
    }

    const FLOAT_ERROR_MARGIN: f64 = 0.0099;

    #[test]
    fn return_correct_memory_usage() {
        let stats_stats = create_memory_stats_stats(3);
        let memory_stats = create_memory_stats(Option::Some(stats_stats), Some(5), Some(10));
        let actual = usage(&memory_stats);

        assert!((actual - 20.0).abs() < FLOAT_ERROR_MARGIN);
    }

    #[test]
    fn return_zero_usage_if_no_limit_defined() {
        let stats_stats = create_memory_stats_stats(3);
        let memory_stats = create_memory_stats(Option::Some(stats_stats), Some(5), None);
        let actual = usage(&memory_stats);

        assert!((actual - 0.0).abs() < FLOAT_ERROR_MARGIN);
    }

    #[test]
    fn return_zero_usage_if_no_stats_defined() {
        let memory_stats = create_memory_stats(None, Some(5), Some(10));
        let actual = usage(&memory_stats);

        assert!((actual - 0.0).abs() < FLOAT_ERROR_MARGIN);
    }

    #[test]
    fn return_zero_usage_if_no_usage_defined() {
        let stats_stats = create_memory_stats_stats(3);
        let memory_stats = create_memory_stats(Option::Some(stats_stats), None, Some(10));
        let actual = usage(&memory_stats);

        assert!((actual - 0.0).abs() < FLOAT_ERROR_MARGIN);
    }
}
