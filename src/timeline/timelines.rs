use compact_str::CompactString;
use super::metrics::EventMetrics;
use hashbrown::HashMap;

#[derive(Debug, Clone)]
pub struct Timeline {
    pub total_record_cnt: usize,
    pub stats: EventMetrics,
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Timeline {
    pub fn new() -> Timeline {
        let totalcnt = 0;
        let filepath = CompactString::default();
        let statslst = HashMap::new();
        let statsloginlst = HashMap::new();

        let statistic = EventMetrics::new(
            totalcnt,
            filepath.clone(),
            None,
            None,
            statslst,
            statsloginlst,
        );
        Timeline {
            total_record_cnt: 0,
            stats: statistic,
        }
    }
}

#[cfg(test)]
mod tests {}
