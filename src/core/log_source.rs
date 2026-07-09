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
            LogSource::Azure => &[
                "activitylogs",
                "auditlogs",
                "signinlogs",
                "m365",
                "riskdetection",
                "pim",
            ],
            LogSource::All => &[
                "cloudtrail",
                "activitylogs",
                "auditlogs",
                "signinlogs",
                "m365",
                "riskdetection",
                "pim",
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
            // Entra ID Protection risk detections and Privileged Identity Management alert
            // incidents share the Microsoft Graph risk-event schema, identified by
            // `riskEventType`. The rule's specific `riskEventType` value selects the sub-type.
            "riskdetection" | "pim" => event.get("riskEventType").is_some(),
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigma_rust::event_from_json;

    fn ev(json: &str) -> Event {
        event_from_json(json).unwrap()
    }

    #[test]
    fn riskdetection_and_pim_match_risk_events() {
        // Entra ID Protection risk detections and PIM alert incidents both carry
        // `riskEventType`.
        let e = ev(r#"{"riskEventType":"anomalousToken","riskLevel":"high"}"#);
        assert!(is_match_service(&Some("riskdetection".to_string()), &e));
        assert!(is_match_service(&Some("pim".to_string()), &e));
    }

    #[test]
    fn risk_services_do_not_match_non_risk_events() {
        let e = ev(r#"{"category":"SignInLogs","properties":{}}"#);
        assert!(!is_match_service(&Some("riskdetection".to_string()), &e));
        assert!(!is_match_service(&Some("pim".to_string()), &e));
    }

    #[test]
    fn category_services_still_match() {
        let e = ev(r#"{"category":"SignInLogs"}"#);
        assert!(is_match_service(&Some("signinlogs".to_string()), &e));
        assert!(!is_match_service(&Some("auditlogs".to_string()), &e));
    }
}
