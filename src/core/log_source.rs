use sigma_rust::Event;

pub enum LogSource {
    Aws,
    Azure,
    All,
}

impl LogSource {
    pub fn profile_path(&self) -> &str {
        match self {
            LogSource::Aws => "config/aws_profile.yaml",
            LogSource::Azure => "config/azure_profile.yaml",
            // The 'All' variant does not have a specific profile path.
            // Return an empty string to indicate no profile is available.
            LogSource::All => "",
        }
    }

    pub fn supported_services(&self) -> &[&str] {
        match self {
            LogSource::Aws => &["cloudtrail"],
            LogSource::Azure => &["activitylogs", "auditlogs", "signinlogs"],
            LogSource::All => &["cloudtrail", "activitylogs", "auditlogs", "signinlogs"],
        }
    }
}

pub fn is_match_service(service: &Option<String>, event: &Event) -> bool {
    if let Some(s) = service {
        match s.as_str() {
            "cloudtrail" => true,
            "activitylogs" => event
                .get("category")
                .is_some_and(|v| v.value_to_string() == "Administrative"),
            "auditlogs" => event
                .get("category")
                .is_some_and(|v| v.value_to_string() == "AuditLogs"),
            "signinlogs" => event
                .get("category")
                .is_some_and(|v| v.value_to_string() == "SignInLogs"),
            _ => false,
        }
    } else {
        false
    }
}
