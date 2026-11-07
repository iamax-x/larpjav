//! Rule evaluation for MoJson

use crate::value::Rule;
use std::collections::HashMap;

/// Evaluate rules for current OS/arch/features
pub fn evaluate_rules(rules: &[Rule], os: &str, arch: &str, features: &HashMap<String, bool>) -> bool {
    if rules.is_empty() {
        return true;
    }

    let mut allow = true; // Default to allow

    for rule in rules {
        let matches = rule_matches(rule, os, arch, features);
        if matches {
            allow = matches!(rule.action, crate::value::RuleAction::Allow);
        }
    }

    allow
}

/// Check if a single rule matches
fn rule_matches(rule: &Rule, os: &str, arch: &str, features: &HashMap<String, bool>) -> bool {
    // Check OS rule
    if let Some(os_rule) = &rule.os {
        if !os_rule_matches(os_rule, os, arch) {
            return false;
        }
    }

    // Check features
    if let Some(required_features) = &rule.features {
        for (feature, required) in required_features {
            if features.get(feature) != Some(required) {
                return false;
            }
        }
    }

    true
}

/// Check OS rule match
fn os_rule_matches(os_rule: &crate::value::OsRule, current_os: &str, current_arch: &str) -> bool {
    if let Some(name) = &os_rule.name {
        if !os_name_matches(name, current_os) {
            return false;
        }
    }
    if let Some(version) = &os_rule.version {
        // Version matching would go here
        // For now, just check if it's not empty
        if version.is_empty() {
            return false;
        }
    }
    if let Some(arch) = &os_rule.arch {
        if !arch_matches(arch, current_arch) {
            return false;
        }
    }
    true
}

/// Check OS name match
fn os_name_matches(pattern: &str, current: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let current = current.to_lowercase();
    pattern == current || pattern == "osx" && current == "macos"
}

/// Check architecture match
fn arch_matches(pattern: &str, current: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let current = current.to_lowercase();
    pattern == current ||
    (pattern == "x86" && (current == "x86" || current == "i386" || current == "i686")) ||
    (pattern == "x64" && (current == "x86_64" || current == "amd64"))
}

/// Get current OS name
pub fn current_os() -> &'static str {
    #[cfg(target_os = "windows")] return "windows";
    #[cfg(target_os = "macos")] return "osx";
    #[cfg(target_os = "linux")] return "linux";
    #[cfg(target_os = "android")] return "android";
    #[cfg(target_os = "ios")] return "ios";
    #[cfg(target_os = "freebsd")] return "freebsd";
    "unknown"
}

/// Get current architecture
pub fn current_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")] return "x64";
    #[cfg(target_arch = "x86")] return "x86";
    #[cfg(target_arch = "aarch64")] return "arm64";
    #[cfg(target_arch = "arm")] return "arm";
    "unknown"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_rules() {
        assert!(evaluate_rules(&[], "linux", "x64", &HashMap::new()));
    }

    #[test]
    fn test_os_rule() {
        let rule = Rule {
            action: crate::value::RuleAction::Allow,
            os: Some(crate::value::OsRule {
                name: Some("windows".to_string()),
                version: None,
                arch: None,
            }),
            features: None,
        };
        assert!(evaluate_rules(&[rule], "windows", "x64", &HashMap::new()));
        assert!(!evaluate_rules(&[rule], "linux", "x64", &HashMap::new()));
    }

    #[test]
    fn test_feature_rule() {
        let rule = Rule {
            action: crate::value::RuleAction::Allow,
            os: None,
            features: Some({
                let mut f = HashMap::new();
                f.insert("is_demo_user".to_string(), true);
                f
            }),
        };
        let mut features = HashMap::new();
        features.insert("is_demo_user".to_string(), true);
        assert!(evaluate_rules(&[rule], "linux", "x64", &features));

        features.insert("is_demo_user".to_string(), false);
        assert!(!evaluate_rules(&[rule], "linux", "x64", &features));
    }

    #[test]
    fn test_current_os_arch() {
        assert!(!current_os().is_empty());
        assert!(!current_arch().is_empty());
    }
}