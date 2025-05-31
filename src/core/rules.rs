use sigma_rust::Rule;
use sigma_rust::rule_from_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn load_rules_from_dir(path: &PathBuf) -> HashMap<String, Rule> {
    let mut rules = HashMap::new();
    if path.is_file() {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(rule) = rule_from_yaml(&contents) {
                if let Some(id) = &rule.id {
                    rules.insert(id.clone(), rule);
                }
            }
        }
        return rules;
    }
    load_rules_recursive(path, &mut rules);
    rules
}

fn load_rules_recursive(directory: &PathBuf, rules: &mut HashMap<String, Rule>) {
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yml") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(rule) = rule_from_yaml(&contents) {
                        rules.insert(rule.id.clone().unwrap_or_default(), rule);
                    }
                }
            } else if path.is_dir() {
                load_rules_recursive(&path, rules);
            }
        }
    }
}
