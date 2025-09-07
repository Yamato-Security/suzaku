use crate::core::log_source::LogSource;
use crate::core::timeline::make_timeline;
use crate::option::cli::{CommonOptions, TimelineOptions};

pub fn aws_timeline(options: &TimelineOptions, common_opt: &CommonOptions) {
    make_timeline(options, common_opt, LogSource::Aws);
}
