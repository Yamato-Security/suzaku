use sigma_rust::Rule;
use sigma_rust::rule_from_yaml;
use std::fs;
use std::path::PathBuf;

pub fn load_rules_from_dir(path: &PathBuf) -> Vec<Rule> {
    let mut rules = Vec::new();
    if path.is_file() {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(rule) = rule_from_yaml(&contents) {
                rules.push(rule);
            }
        }
        return rules;
    }
    load_rules_recursive(path, &mut rules);
    rules
}

fn load_rules_recursive(directory: &PathBuf, rules: &mut Vec<Rule>) {
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yml") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(rule) = rule_from_yaml(&contents) {
                        rules.push(rule);
                    }
                }
            } else if path.is_dir() {
                load_rules_recursive(&path, rules);
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
