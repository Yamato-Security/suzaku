use sigma_rust::Event;
use std::sync::LazyLock;

pub enum LogSource {
    Aws,
    Azure,
    Gws,
    All,
}

/// Declares the Google Workspace applications (`id.applicationName`) Suzaku knows about once,
/// and derives from them the `google_workspace.<app>` service names SigmaHQ's gworkspace rules
/// declare. A list is used rather than accepting any suffix so that a typo in a rule's `service:`
/// is dropped at load time instead of silently matching nothing during the scan.
///
/// The applications and the services have to agree exactly -- a service missing from either list
/// is a rule that never runs -- so both are generated from this single literal list.
macro_rules! gws_applications {
    ($($app:literal),+ $(,)?) => {
        /// The Google Workspace applications (`id.applicationName`) Suzaku knows about. Only
        /// the derived service names are needed at run time; the bare list is kept so a test can
        /// check that the two stay in step.
        #[cfg(test)]
        const GWS_APPLICATIONS: &[&str] = &[$($app),+];

        /// `google_workspace` plus one `google_workspace.<app>` entry per application above,
        /// built at compile time.
        const GWS_SERVICES: &[&str] = &[
            "google_workspace",
            $(concat!("google_workspace.", $app)),+
        ];
    };
}

gws_applications!(
    "admin",
    "login",
    "drive",
    "calendar",
    "token",
    "user_accounts",
    "saml",
    "groups",
    "mobile",
    "gmail",
    "chat",
    "meet",
    "chrome",
    "rules",
    "context_aware_access",
    "access_transparency",
    "keep",
    "vault",
);

const AWS_SERVICES: &[&str] = &["cloudtrail"];

const AZURE_SERVICES: &[&str] = &[
    "activitylogs",
    "auditlogs",
    "signinlogs",
    "m365",
    "audit",
    "exchange",
    "threat_detection",
    "threat_management",
    "riskdetection",
    "pim",
];

/// Every service any single-source subcommand supports. `LogSource::All` must be the exact union
/// of the others: a service it omits is a rule the `timeline` command silently drops, so it is
/// concatenated from them rather than being maintained as a third list.
static ALL_SERVICES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    AWS_SERVICES
        .iter()
        .chain(AZURE_SERVICES)
        .chain(GWS_SERVICES)
        .copied()
        .collect()
});

impl LogSource {
    pub fn profile_path(&self) -> &str {
        match self {
            LogSource::Aws => "config/aws_profile.yaml",
            LogSource::Azure => "config/azure_profile.yaml",
            LogSource::Gws => "config/gws_profile.yaml",
            LogSource::All => "",
        }
    }

    /// The timeline subcommand this log source belongs to, as the user typed it. Recorded in the
    /// DuckDB `suzaku_meta.command` column so a consumer can look up what produced a file rather
    /// than inferring it from which tables and columns happen to be present.
    pub fn command_name(&self) -> &'static str {
        match self {
            LogSource::Aws => "aws-ct-timeline",
            LogSource::Azure => "azure-timeline",
            LogSource::Gws => "gws-timeline",
            LogSource::All => "timeline",
        }
    }

    /// File name (under the rules directory's `config/`) listing rule UUIDs to skip loading.
    pub fn ignore_rule_list_filename(&self) -> &str {
        match self {
            LogSource::Aws => "aws_ignore_rule_list.txt",
            LogSource::Azure => "azure_ignore_rule_list.txt",
            LogSource::Gws => "gws_ignore_rule_list.txt",
            LogSource::All => "",
        }
    }

    pub fn supported_services(&self) -> &[&str] {
        match self {
            LogSource::Aws => AWS_SERVICES,
            LogSource::Azure => AZURE_SERVICES,
            LogSource::Gws => GWS_SERVICES,
            LogSource::All => &ALL_SERVICES,
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
            // SigmaHQ's upstream m365 rules split across several service names
            // (audit/exchange/threat_detection/threat_management); all of them target UAL records.
            "m365" | "audit" | "exchange" | "threat_detection" | "threat_management" => {
                event.get("Workload").is_some() || event.get("RecordType").is_some()
            }
            // Entra ID Protection risk detections and Privileged Identity Management alert
            // incidents share the Microsoft Graph risk-event schema, identified by
            // `riskEventType`. The rule's specific `riskEventType` value selects the sub-type.
            "riskdetection" | "pim" => event.get("riskEventType").is_some(),
            // Google Workspace Reports API activities, normalized by `normalize_gws_event` into
            // one flat record per `events[]` entry. SigmaHQ's gworkspace rules declare
            // `service: google_workspace.<app>`; a bare `google_workspace` matches any app.
            // The `kind` check keeps a rule from firing on an unrelated log that happens to
            // carry an `id.applicationName`.
            gws if gws == "google_workspace" || gws.starts_with("google_workspace.") => {
                event
                    .get("kind")
                    .is_some_and(|v| v.value_to_string() == "admin#reports#activity")
                    && gws.strip_prefix("google_workspace.").is_none_or(|app| {
                        event
                            .get("id.applicationName")
                            .is_some_and(|v| v.value_to_string() == app)
                    })
            }
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

    #[test]
    fn google_workspace_services_match_reports_api_activities() {
        // A normalized Google Workspace record: `kind` plus the nested `id.applicationName`.
        let admin = ev(
            r#"{"kind":"admin#reports#activity","id":{"applicationName":"admin"},"eventName":"ASSIGN_ROLE"}"#,
        );
        assert!(is_match_service(
            &Some("google_workspace".to_string()),
            &admin
        ));
        assert!(is_match_service(
            &Some("google_workspace.admin".to_string()),
            &admin
        ));
        // A different application must not match.
        assert!(!is_match_service(
            &Some("google_workspace.login".to_string()),
            &admin
        ));
        // A record from another cloud must not match either service form.
        let azure = ev(r#"{"category":"SignInLogs"}"#);
        assert!(!is_match_service(
            &Some("google_workspace".to_string()),
            &azure
        ));
        assert!(!is_match_service(
            &Some("google_workspace.admin".to_string()),
            &azure
        ));
    }

    #[test]
    fn google_workspace_services_are_all_declared_supported() {
        // Every service `is_match_service` accepts must also be loadable, otherwise a rule
        // declaring it is dropped before it is ever evaluated. `LogSource::All` is built by
        // concatenating the single-source lists, so this checks that concatenation: the union is
        // complete and carries no duplicate (a duplicate would load the same rule set twice).
        let all = LogSource::All.supported_services();
        let mut expected = 0;
        for log in [LogSource::Aws, LogSource::Azure, LogSource::Gws] {
            for svc in log.supported_services() {
                assert!(all.contains(svc), "{svc} missing from LogSource::All");
                expected += 1;
            }
        }
        assert_eq!(all.len(), expected, "LogSource::All carries extra services");
        let mut unique = all.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            all.len(),
            "duplicate service in LogSource::All"
        );
    }

    #[test]
    fn google_workspace_services_are_derived_from_the_application_list() {
        // The service names are generated from the application list and `is_match_service` reads
        // the `<app>` back out of them, so the two have to stay in step: a service whose suffix
        // is not an application Google reports is a rule that loads and then never matches.
        assert_eq!(
            GWS_APPLICATIONS.len(),
            18,
            "the documented closed set of Google Workspace applications"
        );
        assert_eq!(GWS_SERVICES.len(), GWS_APPLICATIONS.len() + 1);
        assert_eq!(GWS_SERVICES[0], "google_workspace");
        for app in GWS_APPLICATIONS {
            let service = format!("google_workspace.{app}");
            assert!(
                GWS_SERVICES.contains(&service.as_str()),
                "{service} missing from GWS_SERVICES"
            );
            let event = ev(&format!(
                r#"{{"kind":"admin#reports#activity","id":{{"applicationName":"{app}"}}}}"#
            ));
            assert!(
                is_match_service(&Some(service.clone()), &event),
                "{service} must match an activity from its own application"
            );
        }
    }

    #[test]
    fn m365_family_services_match_unified_audit_log_records() {
        // SigmaHQ's m365 rules use several service names; all target UAL records, which are
        // identified by `Workload`/`RecordType` rather than the Azure Monitor `category`.
        let ual = ev(r#"{"Workload":"Exchange","RecordType":1,"Operation":"Add-FederatedDomain"}"#);
        for svc in [
            "m365",
            "audit",
            "exchange",
            "threat_detection",
            "threat_management",
        ] {
            assert!(
                is_match_service(&Some(svc.to_string()), &ual),
                "service {svc} should match a UAL record"
            );
        }
        // An Azure Monitor record (only `category`) must not be treated as a UAL record.
        let azure = ev(r#"{"category":"SignInLogs"}"#);
        for svc in ["audit", "exchange", "threat_detection", "threat_management"] {
            assert!(
                !is_match_service(&Some(svc.to_string()), &azure),
                "service {svc} should not match an Azure Monitor record"
            );
        }
    }
}
