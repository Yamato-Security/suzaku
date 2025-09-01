use crate::core::timeline::make_timeline;
use crate::option::cli::{CommonOptions, TimelineOptions};

pub fn aws_detect(options: &TimelineOptions, common_opt: &CommonOptions) {
    make_timeline(options, common_opt, "config/aws_profile.yaml");
}
