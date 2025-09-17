use crate::core::log_source::LogSource;
use crate::core::timeline::make_timeline;
use crate::core::util::p;
use crate::option::cli::{CommonOptions, TimelineOptions};
use std::path::Path;

pub fn azure_timeline(options: &TimelineOptions, common_opt: &CommonOptions) {
    let log = LogSource::Azure;
    let profile_path = log.profile_path();
    if !Path::new(profile_path).exists() {
        p(
            None,
            &format!("Profile file does not exist: {:?}", profile_path),
            true,
        );
        return;
    }
    make_timeline(options, common_opt, log);
}
