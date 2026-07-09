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
            LogSource::All => "",
        }
    }

    pub fn supported_services(&self) -> &[&str] {
        match self {
            LogSource::Aws => &["cloudtrail"],
            LogSource::Azure => &["activitylogs", "auditlogs", "signinlogs", "m365"],
            LogSource::All => &[
                "cloudtrail",
                "activitylogs",
                "auditlogs",
                "signinlogs",
                "m365",
            ],
        }
    }
}

pub fn is_match_service(service: &Option<String>, event: &Event) -> bool {
    if let Some(s) = service {
        match s.as_str() {
            "cloudtrail" => true,
            "activitylogs" => {
                event
                    .get("category")
                    .is_some_and(|v| v.value_to_string() == "Administrative")
                    || event
                        .get("category.value")
                        .is_some_and(|v| v.value_to_string() == "Administrative")
            }
            "auditlogs" => {
                event
                    .get("category")
                    .is_some_and(|v| v.value_to_string() == "AuditLogs")
                    || event
                        .get("category.value")
                        .is_some_and(|v| v.value_to_string() == "AuditLogs")
            }
            "signinlogs" => {
                event
                    .get("category")
                    .is_some_and(|v| v.value_to_string() == "SignInLogs")
                    || event
                        .get("category.value")
                        .is_some_and(|v| v.value_to_string() == "SignInLogs")
            }
            // M365 Unified Audit Log records (Exchange/AzureActiveDirectory/etc.); these carry a
            // `Workload` (and numeric `RecordType`) instead of the Azure Monitor `category`.
            "m365" => event.get("Workload").is_some() || event.get("RecordType").is_some(),
            _ => false,
        }
    } else {
        false
    }
}
