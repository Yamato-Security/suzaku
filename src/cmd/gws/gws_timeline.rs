use crate::core::log_source::LogSource;
use crate::core::timeline::make_timeline;
use crate::core::util::error_msg;
use crate::option::cli::{CommonOptions, TimelineOptions};
use std::path::Path;

pub fn gws_timeline(options: &TimelineOptions, common_opt: &CommonOptions) {
    let log = LogSource::Gws;
    let profile_path = log.profile_path();
    if !Path::new(profile_path).exists() {
        error_msg(
            common_opt.no_color,
            &format!("Profile file does not exist: {profile_path:?}"),
        );
        return;
    }
    make_timeline(options, common_opt, log);
}
