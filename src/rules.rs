use sigma_rust::Rule;
use sigma_rust::rule_from_yaml;
use std::fs;

pub fn load_rules_from_dir(directory: &str) -> Vec<Rule> {
    let mut rules = Vec::new();
    load_rules_recursive(directory, &mut rules);
    rules
}

fn load_rules_recursive(directory: &str, rules: &mut Vec<Rule>) {
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
                load_rules_recursive(path.to_str().unwrap(), rules);
            }
        }
    }
}
