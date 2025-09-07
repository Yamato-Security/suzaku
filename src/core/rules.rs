use crate::core::log_source::LogSource;
use sigma_rust::Rule;
use sigma_rust::rule_from_yaml;
use std::fs;
use std::path::PathBuf;

pub fn load_correlation_yamls_from_dir(path: &PathBuf) -> Vec<String> {
    let mut yaml_contents = Vec::new();
    if path.is_file() {
        if path.extension().and_then(|s| s.to_str()) == Some("yml")
            && let Ok(contents) = fs::read_to_string(path)
            && contains_correlation_key(&contents)
        {
            yaml_contents.push(contents);
        }
        return yaml_contents;
    }

    load_correlation_yamls_recursive(path, &mut yaml_contents);
    yaml_contents
}

fn load_correlation_yamls_recursive(directory: &PathBuf, yaml_contents: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yml") {
                if let Ok(contents) = fs::read_to_string(&path)
                    && contains_correlation_key(&contents)
                {
                    yaml_contents.push(contents);
                }
            } else if path.is_dir() {
                load_correlation_yamls_recursive(&path, yaml_contents);
            }
        }
    }
}

fn contains_correlation_key(yaml_content: &str) -> bool {
    yaml_content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("correlation:") || trimmed.contains("correlation:")
    })
}

pub fn load_rules_from_dir(path: &PathBuf, log: &LogSource) -> Vec<Rule> {
    let mut rules = Vec::new();
    if path.is_file() {
        if let Ok(contents) = fs::read_to_string(path)
            && let Ok(rule) = rule_from_yaml(&contents)
            && let Some(service) = &rule.logsource.service
            && log.supported_services().contains(&service.as_str())
        {
            rules.push(rule);
        }
        return rules;
    }
    load_rules_recursive(path, &mut rules, log);
    rules
}

fn load_rules_recursive(directory: &PathBuf, rules: &mut Vec<Rule>, log: &LogSource) {
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yml") {
                if let Ok(contents) = fs::read_to_string(&path)
                    && let Ok(rule) = rule_from_yaml(&contents)
                    && let Some(service) = &rule.logsource.service
                    && log.supported_services().contains(&service.as_str())
                {
                    rules.push(rule);
                }
            } else if path.is_dir() {
                load_rules_recursive(&path, rules, log);
            }
        }
    }
}

fn level_to_int(level: &str) -> u8 {
    match level.to_lowercase().as_str() {
        "info" | "informational" => 1,
        "low" => 2,
        "med" | "medium" => 3,
        "high" => 4,
        "crit" | "critical" => 5,
        _ => 0,
    }
}

pub fn filter_rules_by_level<'a>(rules: &'a [Rule], min_level: &'a str) -> Vec<&'a Rule> {
    let min = level_to_int(min_level);
    rules
        .iter()
        .filter(|rule| {
            rule.level
                .as_ref()
                .map(|lvl| level_to_int(&format!("{lvl:?}")) >= min)
                .unwrap_or(false)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sigma_rust::Rule;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_rule_with_level(level: Option<&str>) -> Rule {
        let yaml = format!(
            r#"
            title: Test Rule
            id: test_rule
            description: A test rule for filtering by level
            level: {}
            logsource:
              product: test
            detection:
              selection:
                field: value
              condition: selection
            "#,
            level.unwrap_or("informational")
        );
        rule_from_yaml(&yaml).unwrap()
    }

    fn create_test_rule_yaml() -> String {
        r#"
title: Test Rule
id: test-rule-001
description: A test rule for unit testing
level: medium
logsource:
  product: aws
  service: cloudtrail
detection:
  selection:
    EventID: 1
    Image: "*.exe"
  condition: selection
"#
        .to_string()
    }

    #[test]
    fn test_load_rules_from_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_rule.yml");

        fs::write(&file_path, create_test_rule_yaml()).unwrap();

        let rules = load_rules_from_dir(&file_path, &LogSource::All);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].title.as_str(), "Test Rule");
    }

    #[test]
    fn test_load_rules_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        // Create multiple rule files
        fs::write(dir_path.join("rule1.yml"), create_test_rule_yaml()).unwrap();
        fs::write(dir_path.join("rule2.yml"), create_test_rule_yaml()).unwrap();
        fs::write(dir_path.join("not_a_rule.txt"), "not a yaml file").unwrap();

        let rules = load_rules_from_dir(&dir_path, &LogSource::All);

        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_load_rules_from_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();
        let sub_dir = dir_path.join("subdir");

        fs::create_dir(&sub_dir).unwrap();
        fs::write(dir_path.join("rule1.yml"), create_test_rule_yaml()).unwrap();
        fs::write(sub_dir.join("rule2.yml"), create_test_rule_yaml()).unwrap();

        let rules = load_rules_from_dir(&dir_path, &LogSource::All);

        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_load_rules_from_nonexistent_path() {
        let nonexistent_path = PathBuf::from("/nonexistent/path");

        let rules = load_rules_from_dir(&nonexistent_path, &LogSource::All);

        assert_eq!(rules.len(), 0);
    }

    #[test]
    fn test_load_rules_ignores_non_yml_files() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_path_buf();

        fs::write(dir_path.join("rule.yml"), create_test_rule_yaml()).unwrap();
        fs::write(dir_path.join("rule.yaml"), create_test_rule_yaml()).unwrap(); // .yaml extension
        fs::write(dir_path.join("rule.txt"), create_test_rule_yaml()).unwrap();
        fs::write(dir_path.join("rule.json"), "{}").unwrap();

        let rules = load_rules_from_dir(&dir_path, &LogSource::All);

        assert_eq!(rules.len(), 1); // Only .yml file should be loaded
    }

    #[test]
    fn test_filter_rules_by_level() {
        let rules = vec![
            make_rule_with_level(Some("informational")),
            make_rule_with_level(Some("low")),
            make_rule_with_level(Some("medium")),
            make_rule_with_level(Some("high")),
            make_rule_with_level(Some("critical")),
        ];

        let filtered = filter_rules_by_level(&rules, "informational");
        assert_eq!(filtered.len(), 5);

        let filtered = filter_rules_by_level(&rules, "medium");
        assert_eq!(filtered.len(), 3);

        let filtered = filter_rules_by_level(&rules, "critical");
        assert_eq!(filtered.len(), 1);
    }
}
